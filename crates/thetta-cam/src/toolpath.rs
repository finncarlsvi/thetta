use geo_buffer::buffer_polygon;
use geo_types::{LineString, MultiPolygon, Polygon as GeoPolygon};
use thetta_geometry::polygon_ops;
use thetta_geometry::types::{
    GcodeConfig, Point2D, Point3D, Polygon, ProfileCutConfig, Tab, Tool, Toolpath, ToolpathMove,
};

const EPS: f64 = 1e-9;

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectedTab {
    pub position: Point2D,
    pub edge_index: usize,
    pub t: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntryPoint {
    pub position: Point2D,
    pub edge_index: usize,
    pub t: f64,
    pub outward_normal: Point2D,
}

#[derive(Debug, Clone, Copy)]
struct TabInterval {
    start: f64,
    end: f64,
    height: f64,
    ramp: f64,
}

pub fn offset_contour(contour: &Polygon, distance: f64) -> Polygon {
    if contour.outer.len() < 3 || distance.abs() <= EPS {
        return contour.clone();
    }

    let geo_polygon = to_geo_polygon(contour);
    let buffered = buffer_polygon(&geo_polygon, distance);
    from_largest_multipolygon(&buffered).unwrap_or_else(|| contour.clone())
}

pub fn project_tabs(tabs: &[Tab], offset_contour: &Polygon, tool_radius: f64) -> Vec<ProjectedTab> {
    let mut projected = Vec::with_capacity(tabs.len());
    let n = offset_contour.outer.len();
    if n < 2 {
        return projected;
    }

    for tab in tabs {
        let mut best_dist2 = f64::INFINITY;
        let mut best_idx = 0usize;
        let mut best_t = 0.0;
        let mut best_point = offset_contour.outer[0];

        for i in 0..n {
            let a = offset_contour.outer[i];
            let b = offset_contour.outer[(i + 1) % n];
            let (pt, t) = nearest_point_on_segment(tab.position, a, b);
            let dx = tab.position.x - pt.x;
            let dy = tab.position.y - pt.y;
            let d2 = dx * dx + dy * dy;
            if d2 < best_dist2 {
                best_dist2 = d2;
                best_idx = i;
                best_t = t;
                best_point = pt;
            }
        }

        let distance = best_dist2.sqrt();
        if tool_radius > 0.0 && distance > tool_radius * 1.5 {
            eprintln!(
                "warning: tab projection distance {:.3} exceeds 1.5 * tool radius {:.3}",
                distance, tool_radius
            );
        }

        projected.push(ProjectedTab {
            position: best_point,
            edge_index: best_idx,
            t: best_t,
            width: tab.width,
            height: tab.height,
        });
    }

    projected
}

pub fn select_entry_point(
    offset_contour: &Polygon,
    projected_tabs: &[ProjectedTab],
) -> Option<EntryPoint> {
    let n = offset_contour.outer.len();
    if n < 2 {
        return None;
    }

    let centroid = ring_centroid(&offset_contour.outer);
    let mut best_score = f64::NEG_INFINITY;
    let mut best_idx = 0usize;

    for i in 0..n {
        let a = offset_contour.outer[i];
        let b = offset_contour.outer[(i + 1) % n];
        let dx = b.x - a.x;
        let dy = b.y - a.y;
        let len = (dx * dx + dy * dy).sqrt();
        if len <= EPS {
            continue;
        }

        let mid = Point2D {
            x: (a.x + b.x) * 0.5,
            y: (a.y + b.y) * 0.5,
        };
        let cx = mid.x - centroid.x;
        let cy = mid.y - centroid.y;
        let dist = (cx * cx + cy * cy).sqrt();
        let mut score = dist * len;
        if projected_tabs.iter().any(|t| t.edge_index == i) {
            score -= 1_000_000.0;
        }

        if score > best_score {
            best_score = score;
            best_idx = i;
        }
    }

    let a = offset_contour.outer[best_idx];
    let b = offset_contour.outer[(best_idx + 1) % n];
    let dir = unit_dir(a, b);
    let outward = Point2D {
        x: dir.y,
        y: -dir.x,
    };

    Some(EntryPoint {
        position: Point2D {
            x: (a.x + b.x) * 0.5,
            y: (a.y + b.y) * 0.5,
        },
        edge_index: best_idx,
        t: 0.5,
        outward_normal: outward,
    })
}

pub fn plan_profile_toolpath(
    contour: &Polygon,
    tabs: &[Tab],
    tool: &Tool,
    gcode_config: &GcodeConfig,
    profile_config: &ProfileCutConfig,
) -> Toolpath {
    if contour.outer.len() < 3 {
        return Toolpath {
            moves: Vec::new(),
            config: gcode_config.clone(),
        };
    }

    if tool.diameter <= 0.0 {
        return Toolpath {
            moves: Vec::new(),
            config: gcode_config.clone(),
        };
    }

    let mut working = contour.clone();
    ensure_ccw(&mut working);

    let tool_radius = tool.diameter * 0.5;
    let mut offset = offset_contour(&working, tool_radius);
    ensure_ccw(&mut offset);

    let projected_tabs = project_tabs(tabs, &offset, tool_radius);
    let Some(entry) = select_entry_point(&offset, &projected_tabs) else {
        return Toolpath {
            moves: Vec::new(),
            config: gcode_config.clone(),
        };
    };

    let n = offset.outer.len();
    let perimeter = polygon_ops::perimeter(&offset);
    if perimeter <= EPS {
        return Toolpath {
            moves: Vec::new(),
            config: gcode_config.clone(),
        };
    }

    let entry_dist = edge_prefix_length(&offset.outer, entry.edge_index)
        + polygon_ops::edge_length(
            offset.outer[entry.edge_index],
            offset.outer[(entry.edge_index + 1) % n],
        ) * entry.t;

    let mut intervals = build_tab_intervals(&projected_tabs, &offset, entry_dist, perimeter);
    intervals.sort_by(|a, b| a.start.total_cmp(&b.start));

    let path_points = build_path_points(&offset, &entry);
    let segment_lengths = build_segment_lengths(&path_points);
    let total_len: f64 = segment_lengths.iter().sum();
    if total_len <= EPS {
        return Toolpath {
            moves: Vec::new(),
            config: gcode_config.clone(),
        };
    }

    let mut breakpoints = vec![0.0, total_len];
    let mut acc = 0.0;
    for len in &segment_lengths {
        acc += *len;
        breakpoints.push(acc);
    }
    for interval in &intervals {
        let r = interval.ramp;
        breakpoints.push(interval.start);
        breakpoints.push((interval.start + r).min(interval.end));
        breakpoints.push((interval.end - r).max(interval.start));
        breakpoints.push(interval.end);
    }
    breakpoints.sort_by(|a, b| a.total_cmp(b));
    breakpoints.dedup_by(|a, b| (*a - *b).abs() < 1e-7);

    let mut moves = Vec::new();
    append_lead_in(&mut moves, &entry, tool, gcode_config, profile_config);

    let mut current_z = -gcode_config.cut_depth;
    for window in breakpoints.windows(2) {
        let d0 = window[0];
        let d1 = window[1];
        if d1 - d0 <= EPS {
            continue;
        }

        let p1 = point_along_path(&path_points, &segment_lengths, d1);
        let z1 = z_at_distance(d1, gcode_config.cut_depth, &intervals);
        let to = Point3D {
            x: p1.x,
            y: p1.y,
            z: z1,
        };

        if (z1 - current_z).abs() > 1e-9 && z1 > -gcode_config.cut_depth + 1e-9 {
            moves.push(ToolpathMove::TabLift {
                to,
                tab_height: (gcode_config.cut_depth + z1).max(0.0),
            });
        } else {
            moves.push(ToolpathMove::LinearCut {
                to,
                feed: tool.feed_rate,
            });
        }
        current_z = z1;
    }

    append_lead_out(&mut moves, &entry, tool, gcode_config, profile_config);
    moves.push(ToolpathMove::Retract {
        to_z: gcode_config.safe_z,
    });

    Toolpath {
        moves,
        config: gcode_config.clone(),
    }
}

fn append_lead_in(
    moves: &mut Vec<ToolpathMove>,
    entry: &EntryPoint,
    tool: &Tool,
    gcode_config: &GcodeConfig,
    profile_config: &ProfileCutConfig,
) {
    let r = profile_config.lead_in_radius.max(0.1);
    let edge_dir = entry_edge_dir(entry);
    let center = Point2D {
        x: entry.position.x + entry.outward_normal.x * r,
        y: entry.position.y + entry.outward_normal.y * r,
    };
    let approach = Point2D {
        x: center.x - edge_dir.x * r,
        y: center.y - edge_dir.y * r,
    };

    moves.push(ToolpathMove::Rapid {
        to: Point3D {
            x: approach.x,
            y: approach.y,
            z: gcode_config.safe_z,
        },
    });

    moves.push(ToolpathMove::ArcCut {
        to: Point3D {
            x: entry.position.x,
            y: entry.position.y,
            z: -gcode_config.cut_depth,
        },
        center,
        clockwise: true,
        feed: tool.plunge_rate,
    });
}

fn append_lead_out(
    moves: &mut Vec<ToolpathMove>,
    entry: &EntryPoint,
    tool: &Tool,
    gcode_config: &GcodeConfig,
    profile_config: &ProfileCutConfig,
) {
    let r = profile_config.lead_in_radius.max(0.1);
    let edge_dir = entry_edge_dir(entry);
    let center = Point2D {
        x: entry.position.x + entry.outward_normal.x * r,
        y: entry.position.y + entry.outward_normal.y * r,
    };
    let departure = Point2D {
        x: center.x + edge_dir.x * r,
        y: center.y + edge_dir.y * r,
    };

    moves.push(ToolpathMove::ArcCut {
        to: Point3D {
            x: departure.x,
            y: departure.y,
            z: -gcode_config.cut_depth,
        },
        center,
        clockwise: false,
        feed: tool.feed_rate,
    });
}

fn build_path_points(contour: &Polygon, entry: &EntryPoint) -> Vec<Point2D> {
    let n = contour.outer.len();
    let mut points = Vec::with_capacity(n + 2);
    points.push(entry.position);

    for i in 1..=n {
        let idx = (entry.edge_index + i) % n;
        points.push(contour.outer[idx]);
    }

    points.push(entry.position);
    points
}

fn build_segment_lengths(points: &[Point2D]) -> Vec<f64> {
    points
        .windows(2)
        .map(|w| polygon_ops::edge_length(w[0], w[1]))
        .collect()
}

fn point_along_path(points: &[Point2D], segment_lengths: &[f64], distance: f64) -> Point2D {
    let mut remaining = distance.max(0.0);
    for (idx, seg_len) in segment_lengths.iter().enumerate() {
        if remaining <= *seg_len + EPS {
            let t = if *seg_len <= EPS {
                0.0
            } else {
                remaining / *seg_len
            };
            return Point2D {
                x: points[idx].x + (points[idx + 1].x - points[idx].x) * t,
                y: points[idx].y + (points[idx + 1].y - points[idx].y) * t,
            };
        }
        remaining -= *seg_len;
    }
    *points.last().unwrap_or(&Point2D { x: 0.0, y: 0.0 })
}

fn z_at_distance(distance: f64, cut_depth: f64, intervals: &[TabInterval]) -> f64 {
    let base = -cut_depth;
    let mut z = base;

    for interval in intervals {
        if distance < interval.start - EPS || distance > interval.end + EPS {
            continue;
        }

        let tab_z = (-(cut_depth - interval.height)).min(0.0).max(base);
        let ramp = interval.ramp.max(EPS);
        let z_i = if distance <= interval.start + ramp {
            let t = ((distance - interval.start) / ramp).clamp(0.0, 1.0);
            base + (tab_z - base) * t
        } else if distance >= interval.end - ramp {
            let t = ((distance - (interval.end - ramp)) / ramp).clamp(0.0, 1.0);
            tab_z + (base - tab_z) * t
        } else {
            tab_z
        };

        if z_i > z {
            z = z_i;
        }
    }

    z
}

fn build_tab_intervals(
    projected_tabs: &[ProjectedTab],
    contour: &Polygon,
    entry_dist: f64,
    perimeter: f64,
) -> Vec<TabInterval> {
    let mut out = Vec::new();

    for tab in projected_tabs {
        let center = edge_prefix_length(&contour.outer, tab.edge_index)
            + polygon_ops::edge_length(
                contour.outer[tab.edge_index],
                contour.outer[(tab.edge_index + 1) % contour.outer.len()],
            ) * tab.t;
        let mut rel = center - entry_dist;
        if rel < 0.0 {
            rel += perimeter;
        }

        let half = (tab.width * 0.5).max(0.0);
        let start = rel - half;
        let end = rel + half;

        if start < 0.0 {
            let len1 = end.min(perimeter).max(0.0);
            push_tab_interval(&mut out, 0.0, len1, tab.height);
            push_tab_interval(&mut out, perimeter + start, perimeter, tab.height);
        } else if end > perimeter {
            push_tab_interval(&mut out, start, perimeter, tab.height);
            push_tab_interval(&mut out, 0.0, end - perimeter, tab.height);
        } else {
            push_tab_interval(&mut out, start, end, tab.height);
        }
    }

    out
}

fn push_tab_interval(out: &mut Vec<TabInterval>, start: f64, end: f64, height: f64) {
    if end - start <= EPS {
        return;
    }
    let len = end - start;
    out.push(TabInterval {
        start,
        end,
        height,
        ramp: 2.0_f64.min(len * 0.5),
    });
}

fn edge_prefix_length(ring: &[Point2D], edge_index: usize) -> f64 {
    if ring.is_empty() {
        return 0.0;
    }
    let mut sum = 0.0;
    for i in 0..edge_index.min(ring.len()) {
        sum += polygon_ops::edge_length(ring[i], ring[(i + 1) % ring.len()]);
    }
    sum
}

fn nearest_point_on_segment(p: Point2D, a: Point2D, b: Point2D) -> (Point2D, f64) {
    let abx = b.x - a.x;
    let aby = b.y - a.y;
    let len2 = abx * abx + aby * aby;
    if len2 <= EPS {
        return (a, 0.0);
    }

    let apx = p.x - a.x;
    let apy = p.y - a.y;
    let t = ((apx * abx + apy * aby) / len2).clamp(0.0, 1.0);
    (
        Point2D {
            x: a.x + abx * t,
            y: a.y + aby * t,
        },
        t,
    )
}

fn ensure_ccw(poly: &mut Polygon) {
    if signed_ring_area(&poly.outer) < 0.0 {
        poly.outer.reverse();
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

fn ring_centroid(points: &[Point2D]) -> Point2D {
    if points.is_empty() {
        return Point2D { x: 0.0, y: 0.0 };
    }
    let (sx, sy) = points
        .iter()
        .fold((0.0, 0.0), |(ax, ay), p| (ax + p.x, ay + p.y));
    let n = points.len() as f64;
    Point2D {
        x: sx / n,
        y: sy / n,
    }
}

fn unit_dir(a: Point2D, b: Point2D) -> Point2D {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let len = (dx * dx + dy * dy).sqrt();
    if len <= EPS {
        Point2D { x: 1.0, y: 0.0 }
    } else {
        Point2D {
            x: dx / len,
            y: dy / len,
        }
    }
}

fn entry_edge_dir(entry: &EntryPoint) -> Point2D {
    // `outward = (dy, -dx)` for a unit edge direction (dx, dy)
    Point2D {
        x: -entry.outward_normal.y,
        y: entry.outward_normal.x,
    }
}

fn to_geo_polygon(poly: &Polygon) -> GeoPolygon<f64> {
    let mut outer = poly
        .outer
        .iter()
        .map(|p| (p.x, p.y))
        .collect::<Vec<(f64, f64)>>();
    if let Some(first) = outer.first().copied()
        && outer.last().copied() != Some(first)
    {
        outer.push(first);
    }

    let holes = poly
        .holes
        .iter()
        .map(|ring| {
            let mut points = ring.iter().map(|p| (p.x, p.y)).collect::<Vec<(f64, f64)>>();
            if let Some(first) = points.first().copied()
                && points.last().copied() != Some(first)
            {
                points.push(first);
            }
            LineString::from(points)
        })
        .collect::<Vec<_>>();

    GeoPolygon::new(LineString::from(outer), holes)
}

fn from_largest_multipolygon(multi: &MultiPolygon<f64>) -> Option<Polygon> {
    let best = multi
        .0
        .iter()
        .max_by(|a, b| area_abs(&a.exterior().0).total_cmp(&area_abs(&b.exterior().0)))?;

    let mut outer = best
        .exterior()
        .0
        .iter()
        .map(|coord| Point2D {
            x: coord.x,
            y: coord.y,
        })
        .collect::<Vec<_>>();
    if outer.len() > 1 && outer.first() == outer.last() {
        outer.pop();
    }

    Some(Polygon {
        outer,
        holes: Vec::new(),
    })
}

fn area_abs(coords: &[geo_types::Coord<f64>]) -> f64 {
    if coords.len() < 3 {
        return 0.0;
    }
    let mut sum = 0.0;
    for i in 0..(coords.len() - 1) {
        let a = coords[i];
        let b = coords[i + 1];
        sum += a.x * b.y - b.x * a.y;
    }
    (0.5 * sum).abs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use thetta_geometry::test_shapes;

    fn sample_tool() -> Tool {
        Tool {
            number: 1,
            name: "6mm".to_string(),
            diameter: 6.0,
            flute_count: 2,
            feed_rate: 1500.0,
            plunge_rate: 500.0,
            spindle_speed: 18000,
        }
    }

    fn sample_cfg() -> GcodeConfig {
        GcodeConfig {
            safe_z: 5.0,
            cut_depth: 18.0,
            tool_number: Some(1),
            coordinate_system: "G54".to_string(),
        }
    }

    #[test]
    fn test_polygon_offset_rectangle() {
        let rect = test_shapes::rectangle(100.0, 50.0);
        let out = offset_contour(&rect, 3.0);
        let bb = polygon_ops::bounding_box(&out);
        assert!((bb.width() - 106.0).abs() < 1e-6);
        assert!((bb.height() - 56.0).abs() < 1e-6);
    }

    #[test]
    fn test_tab_projection() {
        let rect = test_shapes::rectangle(100.0, 50.0);
        let out = offset_contour(&rect, 3.0);
        let tabs = vec![Tab {
            position: Point2D { x: 50.0, y: 0.0 },
            edge_index: 0,
            t: 0.5,
            width: 10.0,
            height: 6.0,
        }];

        let projected = project_tabs(&tabs, &out, 3.0);
        assert_eq!(projected.len(), 1);
        assert!((projected[0].position.x - 50.0).abs() < 1e-6);
        assert!((projected[0].position.y + 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_simple_rectangle_no_tabs() {
        let rect = test_shapes::rectangle(100.0, 50.0);
        let path = plan_profile_toolpath(
            &rect,
            &[],
            &sample_tool(),
            &sample_cfg(),
            &ProfileCutConfig::default(),
        );

        assert!(!path.moves.is_empty());
        assert!(matches!(
            path.moves.first(),
            Some(ToolpathMove::Rapid { .. })
        ));
        assert!(matches!(
            path.moves.last(),
            Some(ToolpathMove::Retract { .. })
        ));
        assert!(
            path.moves
                .iter()
                .any(|m| matches!(m, ToolpathMove::ArcCut { .. }))
        );
    }

    #[test]
    fn test_rectangle_with_one_tab_lifts_z() {
        let rect = test_shapes::rectangle(100.0, 50.0);
        let tabs = vec![Tab {
            position: Point2D { x: 50.0, y: 0.0 },
            edge_index: 0,
            t: 0.5,
            width: 12.0,
            height: 7.0,
        }];
        let path = plan_profile_toolpath(
            &rect,
            &tabs,
            &sample_tool(),
            &sample_cfg(),
            &ProfileCutConfig::default(),
        );

        let mut saw_tab_height = false;
        for mv in &path.moves {
            match mv {
                ToolpathMove::LinearCut { to, .. } | ToolpathMove::TabLift { to, .. } => {
                    if (to.z + 11.0).abs() < 1e-6 {
                        saw_tab_height = true;
                    }
                }
                _ => {}
            }
        }
        assert!(saw_tab_height);
    }

    #[test]
    fn test_tab_z_clamped_above_zero() {
        let rect = test_shapes::rectangle(100.0, 50.0);
        let tabs = vec![Tab {
            position: Point2D { x: 50.0, y: 0.0 },
            edge_index: 0,
            t: 0.5,
            width: 12.0,
            height: 40.0,
        }];
        let path = plan_profile_toolpath(
            &rect,
            &tabs,
            &sample_tool(),
            &sample_cfg(),
            &ProfileCutConfig::default(),
        );

        let mut max_z = f64::NEG_INFINITY;
        for mv in &path.moves {
            if let ToolpathMove::LinearCut { to, .. } | ToolpathMove::TabLift { to, .. } = mv {
                max_z = max_z.max(to.z);
            }
        }
        assert!(max_z <= 0.0 + 1e-9);
    }

    #[test]
    fn test_no_contour() {
        let empty = Polygon {
            outer: vec![],
            holes: vec![],
        };
        let path = plan_profile_toolpath(
            &empty,
            &[],
            &sample_tool(),
            &sample_cfg(),
            &ProfileCutConfig::default(),
        );
        assert!(path.moves.is_empty());
    }
}
