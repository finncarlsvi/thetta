use crate::polygon_ops::{edge_length, perimeter, point_on_edge};
use crate::types::{Point2D, Polygon, Tab, TabConfig};

const COLLINEAR_THRESHOLD: f64 = 1e-6;
const EPS: f64 = 1e-9;

#[derive(Debug, Clone)]
struct EdgeInfo {
    index: usize,
    length: f64,
    score: f64,
    weight: f64,
}

/// Generate holding tabs for a placed (world-space) outer contour.
///
/// Uses a perimeter-centric approach: places tabs at even arc-length intervals
/// around the full contour, then snaps each to the nearest valid edge position
/// (respecting corner exclusions and edge scores).
pub fn generate_tabs(contour: &Polygon, config: &TabConfig) -> Vec<Tab> {
    if contour.outer.len() < 3 {
        return Vec::new();
    }
    debug_assert!(signed_ring_area(&contour.outer) > 0.0);

    let contour_perimeter = perimeter(contour);
    if contour_perimeter < config.size_threshold || !contour_perimeter.is_finite() {
        return Vec::new();
    }

    let tab_distance = config.effective_tab_distance();
    let base_count = (contour_perimeter / tab_distance).round() as usize;
    let target_count = base_count.clamp(config.min_tabs, config.max_tabs);
    if target_count == 0 {
        return Vec::new();
    }

    let tab_width = config.effective_tab_width();
    let tab_height = config.effective_tab_height();
    if !tab_width.is_finite() || !tab_height.is_finite() || tab_width <= 0.0 || tab_height <= 0.0 {
        return Vec::new();
    }

    let n = contour.outer.len();
    let mut vertex_scores = Vec::with_capacity(n);
    for i in 0..n {
        let prev = contour.outer[(i + n - 1) % n];
        let curr = contour.outer[i];
        let next = contour.outer[(i + 1) % n];
        vertex_scores.push(classify_vertex(prev, curr, next));
    }

    let min_length = tab_width + 2.0 * config.corner_exclusion;
    let mut edges = Vec::with_capacity(n);
    // Cumulative arc-length at start of each edge
    let mut edge_starts = Vec::with_capacity(n);
    let mut cumulative = 0.0;
    for i in 0..n {
        let a = contour.outer[i];
        let b = contour.outer[(i + 1) % n];
        let length = edge_length(a, b);
        let score = score_edge(vertex_scores[i], vertex_scores[(i + 1) % n], length, min_length);
        edge_starts.push(cumulative);
        edges.push(EdgeInfo {
            index: i,
            length,
            score,
            weight: length * score,
        });
        cumulative += length;
    }

    // Place tabs at even arc-length intervals, then snap to nearest valid position
    let spacing = contour_perimeter / target_count as f64;
    let mut tabs = Vec::new();

    for i in 0..target_count {
        let ideal_arc = spacing * i as f64 + spacing * 0.5; // offset by half-spacing for centering
        if let Some(tab) = snap_to_nearest_valid(
            ideal_arc,
            contour,
            &edges,
            &edge_starts,
            contour_perimeter,
            config.corner_exclusion,
            tab_width,
            tab_height,
        ) {
            tabs.push(tab);
        }
    }

    if tabs.is_empty() {
        return tabs;
    }

    // Remove tabs that ended up too close after snapping
    remove_spacing_conflicts(
        &mut tabs,
        &edges,
        contour_perimeter,
        2.0 * tab_width,
    );

    // Ensure minimum tab count by adding tabs on unused high-weight edges
    if tabs.len() < config.min_tabs {
        let mut candidates: Vec<usize> = edges
            .iter()
            .filter(|edge| edge.weight > 0.0 && !tabs.iter().any(|t| t.edge_index == edge.index))
            .map(|edge| edge.index)
            .collect();
        candidates.sort_by(|&a, &b| {
            edges[b]
                .weight
                .total_cmp(&edges[a].weight)
                .then_with(|| a.cmp(&b))
        });

        for edge_index in candidates {
            if tabs.len() >= config.min_tabs {
                break;
            }
            let candidate = Tab {
                position: point_on_edge(
                    contour.outer[edge_index],
                    contour.outer[(edge_index + 1) % n],
                    0.5,
                ),
                edge_index,
                t: 0.5,
                width: tab_width,
                height: tab_height,
            };
            if has_min_spacing(
                &tabs,
                &candidate,
                &edges,
                contour_perimeter,
                2.0 * tab_width,
            ) {
                tabs.push(candidate);
            }
        }
    }

    tabs.sort_by(|a, b| {
        a.edge_index
            .cmp(&b.edge_index)
            .then_with(|| a.t.total_cmp(&b.t))
    });
    tabs
}

/// Snap an ideal arc-length position to the nearest valid tab placement.
/// Searches outward from the ideal position, preferring higher-scored edges.
fn snap_to_nearest_valid(
    ideal_arc: f64,
    contour: &Polygon,
    edges: &[EdgeInfo],
    edge_starts: &[f64],
    contour_perimeter: f64,
    corner_exclusion: f64,
    tab_width: f64,
    tab_height: f64,
) -> Option<Tab> {
    let n = edges.len();

    // Find which edge the ideal position falls on
    let edge_idx = find_edge_at_arc(ideal_arc, edge_starts, contour_perimeter);

    // Try the ideal edge first, then search outward in both directions
    let max_search = (n + 1) / 2;
    for offset in 0..=max_search {
        // Try edges at +offset and -offset from the ideal
        let indices: Vec<usize> = if offset == 0 {
            vec![edge_idx]
        } else {
            vec![
                (edge_idx + offset) % n,
                (edge_idx + n - offset) % n,
            ]
        };

        for &ei in &indices {
            if edges[ei].score <= 0.0 || edges[ei].length <= EPS {
                continue;
            }

            // Compute the valid t-range on this edge (excluding corners)
            let excl_t = corner_exclusion / edges[ei].length;
            let t_min = excl_t;
            let t_max = 1.0 - excl_t;
            if t_min >= t_max {
                continue;
            }

            // Find the t-parameter closest to the ideal arc position
            let t = if ei == edge_idx && offset == 0 {
                // Ideal position is on this edge — compute exact t
                let local_arc = ideal_arc - edge_starts[ei];
                (local_arc / edges[ei].length).clamp(t_min, t_max)
            } else {
                // For neighboring edges, place at midpoint of usable range
                (t_min + t_max) * 0.5
            };

            let a = contour.outer[ei];
            let b = contour.outer[(ei + 1) % n];
            return Some(Tab {
                position: point_on_edge(a, b, t),
                edge_index: ei,
                t,
                width: tab_width,
                height: tab_height,
            });
        }
    }

    None
}

/// Find which edge an arc-length position falls on via binary search.
fn find_edge_at_arc(arc: f64, edge_starts: &[f64], _contour_perimeter: f64) -> usize {
    let n = edge_starts.len();
    if n == 0 {
        return 0;
    }
    // Binary search: find the last edge whose start <= arc
    match edge_starts.binary_search_by(|s| s.total_cmp(&arc)) {
        Ok(i) => i,
        Err(i) => {
            if i == 0 {
                n - 1 // wraps around
            } else {
                i - 1
            }
        }
    }
}

fn classify_vertex(prev: Point2D, curr: Point2D, next: Point2D) -> f64 {
    let cross =
        (curr.x - prev.x) * (next.y - curr.y) - (curr.y - prev.y) * (next.x - curr.x);
    if cross > COLLINEAR_THRESHOLD {
        1.0
    } else if cross < -COLLINEAR_THRESHOLD {
        0.3
    } else {
        0.8
    }
}

fn score_edge(start_score: f64, end_score: f64, length: f64, min_length: f64) -> f64 {
    if length < min_length {
        0.0
    } else {
        (start_score + end_score) * 0.5
    }
}

fn signed_ring_area(points: &[Point2D]) -> f64 {
    if points.len() < 3 {
        return 0.0;
    }
    let mut sum = 0.0;
    for i in 0..points.len() {
        let j = (i + 1) % points.len();
        sum += points[i].x * points[j].y - points[j].x * points[i].y;
    }
    0.5 * sum
}

fn arc_position(tab: &Tab, edges: &[EdgeInfo]) -> f64 {
    let prefix: f64 = edges
        .iter()
        .take(tab.edge_index)
        .map(|edge| edge.length)
        .sum();
    prefix + tab.t * edges[tab.edge_index].length
}

fn remove_spacing_conflicts(
    tabs: &mut Vec<Tab>,
    edges: &[EdgeInfo],
    contour_perimeter: f64,
    min_spacing: f64,
) {
    if tabs.len() < 2 || contour_perimeter <= 0.0 {
        return;
    }

    loop {
        if tabs.len() < 2 {
            break;
        }

        let mut ordered: Vec<(usize, f64)> = tabs
            .iter()
            .enumerate()
            .map(|(idx, tab)| (idx, arc_position(tab, edges)))
            .collect();
        ordered.sort_by(|a, b| a.1.total_cmp(&b.1));

        let mut remove_idx = None;
        for i in 0..ordered.len() {
            let j = (i + 1) % ordered.len();
            let s_i = ordered[i].1;
            let s_j = ordered[j].1;
            let forward = if j == 0 {
                contour_perimeter - s_i + s_j
            } else {
                s_j - s_i
            };
            if forward + EPS < min_spacing {
                let idx_i = ordered[i].0;
                let idx_j = ordered[j].0;
                let score_i = edges[tabs[idx_i].edge_index].score;
                let score_j = edges[tabs[idx_j].edge_index].score;
                let chosen = if score_i < score_j {
                    idx_i
                } else if score_j < score_i {
                    idx_j
                } else if tabs[idx_i].edge_index > tabs[idx_j].edge_index {
                    idx_i
                } else if tabs[idx_j].edge_index > tabs[idx_i].edge_index {
                    idx_j
                } else if tabs[idx_i].t >= tabs[idx_j].t {
                    idx_i
                } else {
                    idx_j
                };
                remove_idx = Some(chosen);
                break;
            }
        }

        if let Some(idx) = remove_idx {
            tabs.remove(idx);
        } else {
            break;
        }
    }
}

fn has_min_spacing(
    tabs: &[Tab],
    candidate: &Tab,
    edges: &[EdgeInfo],
    contour_perimeter: f64,
    min_spacing: f64,
) -> bool {
    if tabs.is_empty() {
        return true;
    }
    let c = arc_position(candidate, edges);
    tabs.iter().all(|tab| {
        let s = arc_position(tab, edges);
        let d = (c - s).abs();
        d.min(contour_perimeter - d) + EPS >= min_spacing
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_shapes;
    use crate::types::{MaterialType, TabConfig};

    fn default_cfg() -> TabConfig {
        TabConfig::default()
    }

    fn arc_distances(tabs: &[Tab], poly: &Polygon) -> Vec<f64> {
        if tabs.len() < 2 {
            return Vec::new();
        }
        let n = poly.outer.len();
        let edges = (0..n)
            .map(|i| {
                let len = edge_length(poly.outer[i], poly.outer[(i + 1) % n]);
                EdgeInfo {
                    index: i,
                    length: len,
                    score: 1.0,
                    weight: len,
                        }
            })
            .collect::<Vec<_>>();
        let mut ordered = tabs
            .iter()
            .map(|t| arc_position(t, &edges))
            .collect::<Vec<_>>();
        ordered.sort_by(|a, b| a.total_cmp(b));
        let perim = perimeter(poly);
        let mut out = Vec::new();
        for i in 0..ordered.len() {
            let j = (i + 1) % ordered.len();
            let d = if j == 0 {
                perim - ordered[i] + ordered[j]
            } else {
                ordered[j] - ordered[i]
            };
            out.push(d);
        }
        out
    }

    #[test]
    fn test_rectangle_tabs() {
        let poly = test_shapes::rectangle(100.0, 50.0);
        let tabs = generate_tabs(&poly, &default_cfg());
        assert_eq!(tabs.len(), 6);
        let long_a = tabs.iter().filter(|t| t.edge_index == 0).count();
        let long_b = tabs.iter().filter(|t| t.edge_index == 2).count();
        assert!(long_a >= 1);
        assert!(long_b >= 1);
    }

    #[test]
    fn test_circle_tabs() {
        let poly = test_shapes::circle(50.0, 32);
        let cfg = TabConfig {
            auto_size: false,
            tab_width: 4.0,
            tab_height: 2.0,
            corner_exclusion: 1.0,
            ..default_cfg()
        };
        let tabs = generate_tabs(&poly, &cfg);
        assert_eq!(tabs.len(), 6);
        let unique_edges = tabs
            .iter()
            .map(|t| t.edge_index)
            .collect::<std::collections::BTreeSet<_>>();
        assert!(unique_edges.len() >= 6);
    }

    #[test]
    fn test_l_shape_tabs() {
        let poly = test_shapes::l_shape(100.0, 80.0, 35.0);
        let tabs = generate_tabs(&poly, &default_cfg());
        let concave_edge_count = tabs
            .iter()
            .filter(|t| t.edge_index == 2 || t.edge_index == 3)
            .count();
        let straight_edge_count = tabs
            .iter()
            .filter(|t| t.edge_index == 0 || t.edge_index == 5)
            .count();
        assert!(straight_edge_count >= concave_edge_count);
    }

    #[test]
    fn test_tiny_part_no_tabs() {
        let poly = test_shapes::rectangle(5.0, 5.0);
        let tabs = generate_tabs(&poly, &default_cfg());
        assert!(tabs.is_empty());
    }

    #[test]
    fn test_large_part_max_tabs() {
        let poly = test_shapes::rectangle(2000.0, 2000.0);
        let tabs = generate_tabs(&poly, &default_cfg());
        assert_eq!(tabs.len(), default_cfg().max_tabs);
    }

    #[test]
    fn test_min_tabs_enforced() {
        let cfg = TabConfig {
            tab_distance: 10_000.0,
            min_tabs: 3,
            ..default_cfg()
        };
        let poly = test_shapes::l_shape(120.0, 80.0, 20.0);
        let tabs = generate_tabs(&poly, &cfg);
        assert!(tabs.len() >= cfg.min_tabs);
    }

    #[test]
    fn test_deterministic() {
        let poly = test_shapes::t_shape(120.0, 20.0, 30.0, 80.0);
        let cfg = default_cfg();
        let a = generate_tabs(&poly, &cfg);
        let b = generate_tabs(&poly, &cfg);
        assert_eq!(a, b);
    }

    #[test]
    fn test_no_tabs_at_corners() {
        let poly = test_shapes::rectangle(200.0, 100.0);
        let cfg = default_cfg();
        let tabs = generate_tabs(&poly, &cfg);
        for tab in tabs {
            let a = poly.outer[tab.edge_index];
            let b = poly.outer[(tab.edge_index + 1) % poly.outer.len()];
            let len = edge_length(a, b);
            let dist_to_corner = (tab.t * len).min((1.0 - tab.t) * len);
            assert!(dist_to_corner + EPS >= cfg.corner_exclusion);
        }
    }

    #[test]
    fn test_tab_spacing() {
        let poly = test_shapes::rectangle(300.0, 200.0);
        let cfg = TabConfig {
            tab_distance: 30.0,
            ..default_cfg()
        };
        let tabs = generate_tabs(&poly, &cfg);
        let distances = arc_distances(&tabs, &poly);
        let min_spacing = 2.0 * cfg.effective_tab_width();
        assert!(distances.iter().all(|d| *d + EPS >= min_spacing));
    }

    #[test]
    fn test_all_edges_too_short() {
        let poly = Polygon {
            outer: vec![
                Point2D { x: 0.0, y: 0.0 },
                Point2D { x: 4.0, y: 0.0 },
                Point2D { x: 4.0, y: 4.0 },
                Point2D { x: 0.0, y: 4.0 },
            ],
            holes: Vec::new(),
        };
        let cfg = TabConfig {
            size_threshold: 0.0,
            corner_exclusion: 5.0,
            tab_width: 10.0,
            auto_size: false,
            ..default_cfg()
        };
        let tabs = generate_tabs(&poly, &cfg);
        assert!(tabs.is_empty());
    }

    #[test]
    fn test_material_sizing() {
        let cfg = TabConfig {
            material_type: MaterialType::WoodSoft,
            material_thickness: 18.0,
            auto_size: true,
            ..default_cfg()
        };
        assert!((cfg.effective_tab_height() - 7.2).abs() < 1e-9);
        assert!((cfg.effective_tab_width() - 10.8).abs() < 1e-9);
    }

    #[test]
    fn test_material_sizing_hard_wood() {
        let cfg = TabConfig {
            material_type: MaterialType::WoodHard,
            material_thickness: 18.0,
            auto_size: true,
            ..default_cfg()
        };
        assert!((cfg.effective_tab_height() - 5.94).abs() < 1e-9);
        assert!((cfg.effective_tab_width() - 8.91).abs() < 1e-9);
    }

    #[test]
    fn test_world_space_transform_integration() {
        let base = test_shapes::rectangle(100.0, 40.0);
        let rotated = crate::polygon_ops::rotate(&base, std::f64::consts::FRAC_PI_2, Point2D { x: 0.0, y: 0.0 });
        let placed = crate::polygon_ops::translate(&rotated, 250.0, 300.0);
        let tabs = generate_tabs(&placed, &default_cfg());
        assert!(!tabs.is_empty());
        assert!(tabs.iter().all(|t| t.position.x > 200.0 && t.position.y > 250.0));
    }
}
