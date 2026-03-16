use thetta_geometry::types::{
    GcodeConfig, Point2D, Point3D, PocketConfig, PocketRegion, Polygon, Tool, Toolpath,
    ToolpathMove,
};

use crate::toolpath::offset_contour;

pub fn plan_pocket_toolpath(
    pocket: &PocketRegion,
    tool: &Tool,
    pocket_config: &PocketConfig,
    gcode_config: &GcodeConfig,
) -> Toolpath {
    if pocket.contour.len() < 3 || tool.diameter <= 0.0 {
        return Toolpath {
            moves: Vec::new(),
            config: gcode_config.clone(),
        };
    }

    let mut contour = pocket.contour.clone();
    if signed_ring_area(&contour) < 0.0 {
        contour.reverse();
    }

    let (min_x, max_x, min_y, max_y) = ring_bbox(&contour);
    let min_dimension = (max_x - min_x).min(max_y - min_y);
    if tool.diameter > min_dimension {
        eprintln!(
            "warning: skipping pocket, tool diameter {:.3} exceeds minimum dimension {:.3}",
            tool.diameter, min_dimension
        );
        return Toolpath {
            moves: Vec::new(),
            config: gcode_config.clone(),
        };
    }

    let base = Polygon {
        outer: contour,
        holes: Vec::new(),
    };
    let rings = generate_rings(&base, tool.diameter, pocket_config.step_over);
    if rings.is_empty() {
        eprintln!("warning: skipping pocket, no valid offset rings generated");
        return Toolpath {
            moves: Vec::new(),
            config: gcode_config.clone(),
        };
    }

    let cut_z = -gcode_config.cut_depth;
    let ramp_center = ring_centroid(rings.last().expect("non-empty rings"));
    let ramp_radius = tool.diameter * 0.25;
    let mut moves = Vec::new();
    moves.push(ToolpathMove::Rapid {
        to: Point3D {
            x: ramp_center.x + ramp_radius,
            y: ramp_center.y,
            z: gcode_config.safe_z,
        },
    });
    moves.push(ToolpathMove::HelicalRamp {
        center: ramp_center,
        radius: ramp_radius,
        start_z: 0.0,
        end_z: cut_z,
        clockwise: true,
        feed: tool.plunge_rate,
    });

    for (idx, ring) in rings.iter().enumerate() {
        if ring.is_empty() {
            continue;
        }
        if idx > 0 {
            moves.push(ToolpathMove::Retract {
                to_z: gcode_config.safe_z,
            });
        }
        let start = ring[0];
        moves.push(ToolpathMove::Rapid {
            to: Point3D {
                x: start.x,
                y: start.y,
                z: gcode_config.safe_z,
            },
        });
        moves.push(ToolpathMove::Plunge {
            to_z: cut_z,
            feed: tool.plunge_rate,
        });
        for point in ring.iter().skip(1) {
            moves.push(ToolpathMove::LinearCut {
                to: Point3D {
                    x: point.x,
                    y: point.y,
                    z: cut_z,
                },
                feed: tool.feed_rate,
            });
        }
        moves.push(ToolpathMove::LinearCut {
            to: Point3D {
                x: start.x,
                y: start.y,
                z: cut_z,
            },
            feed: tool.feed_rate,
        });
    }

    if pocket_config.finish_pass
        && let Some(first_ring) = rings.first()
        && !first_ring.is_empty()
    {
        let start = first_ring[0];
        moves.push(ToolpathMove::Retract {
            to_z: gcode_config.safe_z,
        });
        moves.push(ToolpathMove::Rapid {
            to: Point3D {
                x: start.x,
                y: start.y,
                z: gcode_config.safe_z,
            },
        });
        moves.push(ToolpathMove::Plunge {
            to_z: cut_z,
            feed: tool.plunge_rate,
        });
        for point in first_ring.iter().skip(1) {
            moves.push(ToolpathMove::LinearCut {
                to: Point3D {
                    x: point.x,
                    y: point.y,
                    z: cut_z,
                },
                feed: tool.feed_rate,
            });
        }
        moves.push(ToolpathMove::LinearCut {
            to: Point3D {
                x: start.x,
                y: start.y,
                z: cut_z,
            },
            feed: tool.feed_rate,
        });
    }

    moves.push(ToolpathMove::Retract {
        to_z: gcode_config.safe_z,
    });

    Toolpath {
        moves,
        config: gcode_config.clone(),
    }
}

fn generate_rings(polygon: &Polygon, tool_diameter: f64, step_over: f64) -> Vec<Vec<Point2D>> {
    let mut rings = Vec::new();
    let mut offset_distance = tool_diameter * 0.5;
    let step = tool_diameter * step_over;
    let min_area = tool_diameter * tool_diameter;

    for _ in 0..1024 {
        let offset = offset_contour(polygon, -offset_distance);
        if offset.outer.len() < 3 {
            break;
        }
        let area = signed_ring_area(&offset.outer).abs();
        rings.push(offset.outer);
        if area < min_area {
            break;
        }
        offset_distance += step;
        if step <= 0.0 {
            break;
        }
    }

    rings
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
        .fold((0.0, 0.0), |(sx, sy), p| (sx + p.x, sy + p.y));
    let n = points.len() as f64;
    Point2D {
        x: sx / n,
        y: sy / n,
    }
}

fn ring_bbox(points: &[Point2D]) -> (f64, f64, f64, f64) {
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for p in points {
        min_x = min_x.min(p.x);
        max_x = max_x.max(p.x);
        min_y = min_y.min(p.y);
        max_y = max_y.max(p.y);
    }
    (min_x, max_x, min_y, max_y)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tool() -> Tool {
        Tool {
            number: 1,
            name: "6mm".to_string(),
            diameter: 6.0,
            flute_count: 2,
            feed_rate: 1200.0,
            plunge_rate: 400.0,
            spindle_speed: 16000,
        }
    }

    fn cfg() -> GcodeConfig {
        GcodeConfig {
            safe_z: 5.0,
            cut_depth: 12.0,
            tool_number: Some(1),
            coordinate_system: "G54".to_string(),
        }
    }

    fn rect_pocket(w: f64, h: f64) -> PocketRegion {
        PocketRegion {
            contour: vec![
                Point2D { x: 0.0, y: 0.0 },
                Point2D { x: w, y: 0.0 },
                Point2D { x: w, y: h },
                Point2D { x: 0.0, y: h },
            ],
        }
    }

    #[test]
    fn rectangular_pocket_produces_multiple_rings() {
        let path = plan_pocket_toolpath(&rect_pocket(40.0, 20.0), &tool(), &PocketConfig::default(), &cfg());
        let plunge_count = path
            .moves
            .iter()
            .filter(|m| matches!(m, ToolpathMove::Plunge { .. }))
            .count();
        assert!(plunge_count >= 2);
        assert!(path
            .moves
            .iter()
            .any(|m| matches!(m, ToolpathMove::HelicalRamp { .. })));
    }

    #[test]
    fn small_pocket_skipped() {
        let path = plan_pocket_toolpath(&rect_pocket(4.0, 4.0), &tool(), &PocketConfig::default(), &cfg());
        assert!(path.moves.is_empty());
    }

    #[test]
    fn square_ring_count_matches_formula() {
        let pocket = rect_pocket(40.0, 40.0);
        let rings = generate_rings(
            &Polygon {
                outer: pocket.contour.clone(),
                holes: Vec::new(),
            },
            6.0,
            0.5,
        );
        let expected = (((40.0_f64 / 2.0 - 3.0) / 3.0).floor() as usize) + 1;
        assert_eq!(rings.len(), expected);
    }

    #[test]
    fn linear_cuts_at_cut_depth() {
        let path = plan_pocket_toolpath(&rect_pocket(40.0, 20.0), &tool(), &PocketConfig::default(), &cfg());
        for mv in path.moves {
            if let ToolpathMove::LinearCut { to, .. } = mv {
                assert!((to.z + cfg().cut_depth).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn finish_pass_adds_extra_contour() {
        let mut cfg_pocket = PocketConfig::default();
        cfg_pocket.finish_pass = true;
        let path = plan_pocket_toolpath(&rect_pocket(40.0, 20.0), &tool(), &cfg_pocket, &cfg());
        let plunge_count = path
            .moves
            .iter()
            .filter(|m| matches!(m, ToolpathMove::Plunge { .. }))
            .count();
        assert!(plunge_count >= 3);
    }
}
