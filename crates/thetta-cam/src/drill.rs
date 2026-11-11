use thetta_geometry::polygon_ops::edge_length;
use thetta_geometry::types::{
    DrillConfig, DrillHole, GcodeConfig, Point3D, Tool, Toolpath, ToolpathMove,
};

pub fn plan_drill_toolpath(
    holes: &[DrillHole],
    tool: &Tool,
    drill_config: &DrillConfig,
    gcode_config: &GcodeConfig,
) -> Toolpath {
    if holes.is_empty() {
        return Toolpath {
            moves: Vec::new(),
            config: gcode_config.clone(),
        };
    }

    let filtered: Vec<DrillHole> = holes
        .iter()
        .filter_map(|hole| {
            if (tool.diameter - hole.diameter).abs() > 0.02 {
                eprintln!(
                    "warning: skipping drill hole at ({:.3}, {:.3}) diameter {:.3} for tool {:.3}",
                    hole.center.x, hole.center.y, hole.diameter, tool.diameter
                );
                None
            } else {
                Some(hole.clone())
            }
        })
        .collect();

    if filtered.is_empty() {
        return Toolpath {
            moves: Vec::new(),
            config: gcode_config.clone(),
        };
    }

    let ordered = nearest_neighbor_sort(&filtered);
    let first = ordered[0].center;
    let mut moves = Vec::with_capacity(ordered.len() + 4);
    moves.push(ToolpathMove::Rapid {
        to: Point3D {
            x: first.x,
            y: first.y,
            z: gcode_config.safe_z,
        },
    });
    moves.push(ToolpathMove::DrillCycleStart {
        depth: gcode_config.cut_depth,
        peck_depth: drill_config.peck_depth,
        retract_z: gcode_config.safe_z,
        feed: tool.plunge_rate,
    });
    for hole in ordered {
        moves.push(ToolpathMove::DrillHole {
            position: hole.center,
        });
    }
    moves.push(ToolpathMove::DrillCycleEnd);
    moves.push(ToolpathMove::Retract {
        to_z: gcode_config.safe_z,
    });

    Toolpath {
        moves,
        config: gcode_config.clone(),
    }
}

fn nearest_neighbor_sort(holes: &[DrillHole]) -> Vec<DrillHole> {
    if holes.len() < 2 {
        return holes.to_vec();
    }

    let mut unvisited: Vec<DrillHole> = holes.to_vec();
    let mut ordered = Vec::with_capacity(unvisited.len());
    let mut current = unvisited.remove(0);
    ordered.push(current.clone());

    while !unvisited.is_empty() {
        let mut best_idx = 0usize;
        let mut best_dist = f64::INFINITY;
        for (idx, hole) in unvisited.iter().enumerate() {
            let d = edge_length(current.center, hole.center);
            if d < best_dist {
                best_dist = d;
                best_idx = idx;
            }
        }
        current = unvisited.remove(best_idx);
        ordered.push(current.clone());
    }

    ordered
}

#[cfg(test)]
mod tests {
    use super::*;
    use thetta_geometry::types::Point2D;

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
            cut_depth: 10.0,
            tool_number: Some(1),
            coordinate_system: "G54".to_string(),
        }
    }

    #[test]
    fn single_hole_cycle_sequence() {
        let holes = vec![DrillHole {
            center: Point2D { x: 10.0, y: 20.0 },
            diameter: 6.0,
        }];
        let path = plan_drill_toolpath(&holes, &tool(), &DrillConfig::default(), &cfg());
        assert_eq!(path.moves.len(), 5);
        assert!(matches!(path.moves[0], ToolpathMove::Rapid { .. }));
        assert!(matches!(
            path.moves[1],
            ToolpathMove::DrillCycleStart { .. }
        ));
        assert!(matches!(path.moves[2], ToolpathMove::DrillHole { .. }));
        assert!(matches!(path.moves[3], ToolpathMove::DrillCycleEnd));
        assert!(matches!(path.moves[4], ToolpathMove::Retract { .. }));
    }

    #[test]
    fn nearest_neighbor_ordering() {
        let holes = vec![
            DrillHole {
                center: Point2D { x: 0.0, y: 0.0 },
                diameter: 6.0,
            },
            DrillHole {
                center: Point2D { x: 100.0, y: 0.0 },
                diameter: 6.0,
            },
            DrillHole {
                center: Point2D { x: 10.0, y: 0.0 },
                diameter: 6.0,
            },
        ];
        let path = plan_drill_toolpath(&holes, &tool(), &DrillConfig::default(), &cfg());
        let drill_moves: Vec<Point2D> = path
            .moves
            .iter()
            .filter_map(|mv| match mv {
                ToolpathMove::DrillHole { position } => Some(*position),
                _ => None,
            })
            .collect();
        assert_eq!(drill_moves.len(), 3);
        assert!((drill_moves[0].x - 0.0).abs() < 1e-9);
        assert!((drill_moves[1].x - 10.0).abs() < 1e-9);
        assert!((drill_moves[2].x - 100.0).abs() < 1e-9);
    }

    #[test]
    fn empty_holes_empty_toolpath() {
        let path = plan_drill_toolpath(&[], &tool(), &DrillConfig::default(), &cfg());
        assert!(path.moves.is_empty());
    }

    #[test]
    fn smaller_hole_than_tool_skipped() {
        let holes = vec![DrillHole {
            center: Point2D { x: 1.0, y: 2.0 },
            diameter: 2.0,
        }];
        let path = plan_drill_toolpath(&holes, &tool(), &DrillConfig::default(), &cfg());
        assert!(path.moves.is_empty());
    }
}
