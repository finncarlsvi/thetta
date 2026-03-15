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
    target_tabs: f64,
}

/// Generate holding tabs for a placed (world-space) outer contour.
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
    for i in 0..n {
        let a = contour.outer[i];
        let b = contour.outer[(i + 1) % n];
        let length = edge_length(a, b);
        let score = score_edge(vertex_scores[i], vertex_scores[(i + 1) % n], length, min_length);
        edges.push(EdgeInfo {
            index: i,
            length,
            score,
            weight: length * score,
            target_tabs: 0.0,
        });
    }

    let total_weight: f64 = edges.iter().map(|e| e.weight).sum();
    if total_weight <= 0.0 || !total_weight.is_finite() {
        return Vec::new();
    }

    let mut edge_tabs = vec![0usize; n];
    for edge in &mut edges {
        edge.target_tabs = target_count as f64 * (edge.weight / total_weight);
        edge_tabs[edge.index] = edge.target_tabs.round() as usize;
    }

    let mut allocated = edge_tabs.iter().sum::<usize>();
    if allocated < target_count {
        let mut order: Vec<usize> = (0..n).collect();
        order.sort_by(|&a, &b| {
            let ra = edges[a].target_tabs - edge_tabs[a] as f64;
            let rb = edges[b].target_tabs - edge_tabs[b] as f64;
            rb.total_cmp(&ra)
                .then_with(|| edges[b].weight.total_cmp(&edges[a].weight))
                .then_with(|| a.cmp(&b))
        });
        let mut idx = 0usize;
        while allocated < target_count {
            let edge_idx = order[idx % order.len()];
            if edges[edge_idx].score > 0.0 {
                edge_tabs[edge_idx] += 1;
                allocated += 1;
            }
            idx += 1;
            if idx > n * (target_count + 1) {
                break;
            }
        }
    } else if allocated > target_count {
        let mut order: Vec<usize> = (0..n).collect();
        order.sort_by(|&a, &b| {
            edges[a]
                .weight
                .total_cmp(&edges[b].weight)
                .then_with(|| a.cmp(&b))
        });
        let mut idx = 0usize;
        while allocated > target_count {
            let edge_idx = order[idx % order.len()];
            if edge_tabs[edge_idx] > 0 {
                edge_tabs[edge_idx] -= 1;
                allocated -= 1;
            }
            idx += 1;
            if idx > n * (allocated + 1) {
                break;
            }
        }
    }

    let mut tabs = Vec::new();
    for edge in &edges {
        let tabs_on_edge = edge_tabs[edge.index];
        if tabs_on_edge == 0 || edge.length <= EPS {
            continue;
        }

        let usable_start = config.corner_exclusion / edge.length;
        let usable_end = 1.0 - usable_start;
        let usable_range = usable_end - usable_start;
        if usable_range <= 0.0 {
            continue;
        }

        let a = contour.outer[edge.index];
        let b = contour.outer[(edge.index + 1) % n];
        let spacing = usable_range / (tabs_on_edge + 1) as f64;
        for j in 1..=tabs_on_edge {
            let t = usable_start + spacing * j as f64;
            tabs.push(Tab {
                position: point_on_edge(a, b, t),
                edge_index: edge.index,
                t,
                width: tab_width,
                height: tab_height,
            });
        }
    }

    if tabs.is_empty() {
        return tabs;
    }

    remove_spacing_conflicts(
        &mut tabs,
        &edges,
        contour_perimeter,
        2.0 * tab_width,
    );

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
                    target_tabs: 0.0,
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
