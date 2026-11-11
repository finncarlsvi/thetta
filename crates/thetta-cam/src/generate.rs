use std::collections::HashMap;

use thetta_geometry::polygon_ops;
use thetta_geometry::types::{
    DrillConfig, DrillHole, GcodeConfig, NestingResult, Piece, Placement, PocketConfig,
    PocketRegion, Point2D, ProfileCutConfig, Tab, Tool,
};

pub struct SheetGcodeInput<'a> {
    pub sheet_index: usize,
    pub placements: &'a [Placement],
    pub pieces: &'a [Piece],
    pub tabs: &'a HashMap<(usize, usize), Vec<Tab>>,
    pub gcode_config: &'a GcodeConfig,
    pub profile_tool: &'a Tool,
    pub profile_config: &'a ProfileCutConfig,
    pub drill_tool: Option<&'a Tool>,
    pub drill_config: &'a DrillConfig,
    pub pocket_tool: Option<&'a Tool>,
    pub pocket_config: &'a PocketConfig,
}

pub fn generate_sheet_gcode(input: &SheetGcodeInput<'_>) -> String {
    generate_internal(input, None).expect("manual planner is infallible")
}
pub fn generate_sheet_gcode_planned(
    input: &SheetGcodeInput<'_>,
    profile_step: f64,
    pocket_step: f64,
    ramp_angle: f64,
    compression: bool,
    pocket_angle: f64,
) -> Result<String, String> {
    generate_internal(
        input,
        Some((
            profile_step,
            pocket_step,
            ramp_angle,
            compression,
            pocket_angle,
        )),
    )
}
fn generate_internal(
    input: &SheetGcodeInput<'_>,
    passes: Option<(f64, f64, f64, bool, f64)>,
) -> Result<String, String> {
    let mut profile_toolpaths = Vec::new();
    let mut drill_holes: Vec<DrillHole> = Vec::new();
    let mut pockets: Vec<PocketRegion> = Vec::new();

    let drill_diameter = if input.drill_config.enabled {
        input.drill_tool.map(|tool| tool.diameter)
    } else {
        None
    };

    for (placement_idx, placement) in input.placements.iter().enumerate() {
        let Some(piece) = input.pieces.iter().find(|p| p.id == placement.piece_id) else {
            continue;
        };

        let rotated = polygon_ops::rotate(
            &piece.polygon,
            placement.rotation,
            Point2D { x: 0.0, y: 0.0 },
        );
        let placed = polygon_ops::translate(&rotated, placement.x, placement.y);

        let classified = crate::hole_classify::classify_holes(
            &placed.holes,
            input.drill_config.circularity_threshold,
            drill_diameter,
        );
        for cls in classified {
            match cls {
                thetta_geometry::types::HoleClassification::Drill(hole) => {
                    if input.drill_config.enabled && input.drill_tool.is_some() {
                        drill_holes.push(hole);
                    }
                }
                thetta_geometry::types::HoleClassification::Pocket(region) => {
                    if input.pocket_config.enabled && input.pocket_tool.is_some() {
                        pockets.push(region);
                    }
                }
            }
        }

        let empty_tabs = Vec::new();
        let tabs = input
            .tabs
            .get(&(input.sheet_index, placement_idx))
            .unwrap_or(&empty_tabs);

        let toolpath = if let Some((step, _, angle, compression, _)) = passes {
            crate::passes::profile_entry(
                &placed,
                tabs,
                input.profile_tool,
                input.gcode_config,
                input.profile_config,
                step,
                angle,
                compression,
            )?
        } else {
            crate::toolpath::plan_profile_toolpath(
                &placed,
                tabs,
                input.profile_tool,
                input.gcode_config,
                input.profile_config,
            )
        };
        profile_toolpaths.push(toolpath);
    }

    let drill_toolpath = if input.drill_config.enabled {
        input.drill_tool.and_then(|tool| {
            let path = crate::drill::plan_drill_toolpath(
                &drill_holes,
                tool,
                input.drill_config,
                input.gcode_config,
            );
            (!path.moves.is_empty()).then_some(path)
        })
    } else {
        None
    };

    let mut pocket_toolpaths = Vec::new();
    if input.pocket_config.enabled
        && let Some(pocket_tool) = input.pocket_tool
    {
        let ordered_pockets = nearest_neighbor_pockets(&pockets);
        for pocket in ordered_pockets {
            let path = if let Some((_, step, _, _, angle)) = passes {
                crate::passes::pocket(
                    &pocket,
                    pocket_tool,
                    input.gcode_config,
                    input.pocket_config,
                    step,
                    angle,
                )?
            } else {
                crate::pocket::plan_pocket_toolpath(
                    &pocket,
                    pocket_tool,
                    input.pocket_config,
                    input.gcode_config,
                )
            };
            if !path.moves.is_empty() {
                pocket_toolpaths.push(path);
            }
        }
    }

    let drill_paths = drill_toolpath.into_iter().collect::<Vec<_>>();
    Ok(crate::gcode::emit_multi_op_gcode(
        &drill_paths,
        &pocket_toolpaths,
        &profile_toolpaths,
        input.drill_tool,
        input.pocket_tool,
        input.profile_tool,
        input.gcode_config,
        input.sheet_index,
    ))
}

pub fn generate_all_sheets(
    result: &NestingResult,
    pieces: &[Piece],
    tabs: &HashMap<(usize, usize), Vec<Tab>>,
    profile_tool: &Tool,
    gcode_config: &GcodeConfig,
    profile_config: &ProfileCutConfig,
    drill_tool: Option<&Tool>,
    drill_config: &DrillConfig,
    pocket_tool: Option<&Tool>,
    pocket_config: &PocketConfig,
) -> Vec<(usize, String)> {
    result
        .sheets
        .iter()
        .enumerate()
        .filter(|(_, sheet)| !sheet.placements.is_empty())
        .map(|(sheet_index, sheet)| {
            let input = SheetGcodeInput {
                sheet_index,
                placements: &sheet.placements,
                pieces,
                tabs,
                gcode_config,
                profile_tool,
                profile_config,
                drill_tool,
                drill_config,
                pocket_tool,
                pocket_config,
            };
            (sheet_index, generate_sheet_gcode(&input))
        })
        .collect()
}

fn nearest_neighbor_pockets(pockets: &[PocketRegion]) -> Vec<PocketRegion> {
    if pockets.len() < 2 {
        return pockets.to_vec();
    }

    let mut unvisited = pockets.to_vec();
    let mut ordered = Vec::with_capacity(unvisited.len());
    let mut current = unvisited.remove(0);
    ordered.push(current.clone());

    while !unvisited.is_empty() {
        let curr_center = ring_center(&current.contour);
        let mut best_idx = 0usize;
        let mut best_dist2 = f64::INFINITY;
        for (idx, candidate) in unvisited.iter().enumerate() {
            let c = ring_center(&candidate.contour);
            let dx = c.x - curr_center.x;
            let dy = c.y - curr_center.y;
            let d2 = dx * dx + dy * dy;
            if d2 < best_dist2 {
                best_dist2 = d2;
                best_idx = idx;
            }
        }
        current = unvisited.remove(best_idx);
        ordered.push(current.clone());
    }

    ordered
}

fn ring_center(contour: &[Point2D]) -> Point2D {
    if contour.is_empty() {
        return Point2D { x: 0.0, y: 0.0 };
    }
    let (sx, sy) = contour
        .iter()
        .fold((0.0, 0.0), |(sx, sy), p| (sx + p.x, sy + p.y));
    let n = contour.len() as f64;
    Point2D {
        x: sx / n,
        y: sy / n,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use thetta_geometry::types::{Piece, PieceId, Polygon, Sheet, Tool};

    fn tool(number: u32, diameter: f64) -> Tool {
        Tool {
            number,
            name: format!("T{number}"),
            diameter,
            flute_count: 2,
            feed_rate: 1200.0,
            plunge_rate: 400.0,
            spindle_speed: 16000,
        }
    }

    fn gcfg() -> GcodeConfig {
        GcodeConfig {
            safe_z: 5.0,
            cut_depth: 6.0,
            tool_number: Some(3),
            coordinate_system: "G54".to_string(),
        }
    }

    fn piece_with_circle_hole() -> Piece {
        let hole = (0..32)
            .map(|i| {
                let t = (i as f64 / 32.0) * std::f64::consts::TAU;
                Point2D {
                    x: 10.0 + 2.0 * t.cos(),
                    y: 10.0 + 2.0 * t.sin(),
                }
            })
            .collect::<Vec<_>>();
        let slot = vec![
            Point2D { x: 20.0, y: 6.0 },
            Point2D { x: 26.0, y: 6.0 },
            Point2D { x: 26.0, y: 12.0 },
            Point2D { x: 20.0, y: 12.0 },
        ];
        Piece {
            id: PieceId(1),
            polygon: Polygon {
                outer: vec![
                    Point2D { x: 0.0, y: 0.0 },
                    Point2D { x: 30.0, y: 0.0 },
                    Point2D { x: 30.0, y: 20.0 },
                    Point2D { x: 0.0, y: 20.0 },
                ],
                holes: vec![hole, slot],
            },
            source_file: Default::default(),
            quantity: 1,
        }
    }

    #[test]
    fn operation_order_is_drill_then_pocket_then_profile() {
        let piece = piece_with_circle_hole();
        let input = SheetGcodeInput {
            sheet_index: 0,
            placements: &[Placement {
                piece_id: PieceId(1),
                x: 0.0,
                y: 0.0,
                rotation: 0.0,
            }],
            pieces: std::slice::from_ref(&piece),
            tabs: &HashMap::new(),
            gcode_config: &gcfg(),
            profile_tool: &tool(3, 6.0),
            profile_config: &ProfileCutConfig::default(),
            drill_tool: Some(&tool(1, 4.0)),
            drill_config: &DrillConfig {
                tool_number: Some(1),
                ..DrillConfig::default()
            },
            pocket_tool: Some(&tool(2, 4.0)),
            pocket_config: &PocketConfig {
                tool_number: Some(2),
                ..PocketConfig::default()
            },
        };
        let g = generate_sheet_gcode(&input);
        let d = g.find("(Drill Operations)").unwrap_or(usize::MAX);
        let p = g.find("(Pocket Operations)").unwrap_or(usize::MAX);
        let r = g.find("(Profile Operations)").unwrap_or(usize::MAX);
        assert!(d < p && p < r);
    }

    #[test]
    fn hole_coordinates_are_world_transformed() {
        let piece = piece_with_circle_hole();
        let input = SheetGcodeInput {
            sheet_index: 0,
            placements: &[Placement {
                piece_id: PieceId(1),
                x: 100.0,
                y: 50.0,
                rotation: 0.0,
            }],
            pieces: std::slice::from_ref(&piece),
            tabs: &HashMap::new(),
            gcode_config: &gcfg(),
            profile_tool: &tool(3, 6.0),
            profile_config: &ProfileCutConfig::default(),
            drill_tool: Some(&tool(1, 4.0)),
            drill_config: &DrillConfig {
                tool_number: Some(1),
                ..DrillConfig::default()
            },
            pocket_tool: None,
            pocket_config: &PocketConfig {
                enabled: false,
                ..PocketConfig::default()
            },
        };
        let g = generate_sheet_gcode(&input);
        assert!(
            g.contains("X110.000 Y60.000"),
            "expected transformed drill center in output"
        );
    }

    #[test]
    fn profile_only_when_no_holes() {
        let piece = Piece {
            id: PieceId(1),
            polygon: Polygon {
                outer: vec![
                    Point2D { x: 0.0, y: 0.0 },
                    Point2D { x: 20.0, y: 0.0 },
                    Point2D { x: 20.0, y: 20.0 },
                    Point2D { x: 0.0, y: 20.0 },
                ],
                holes: Vec::new(),
            },
            source_file: Default::default(),
            quantity: 1,
        };
        let result = NestingResult {
            sheets: vec![Sheet {
                placements: vec![Placement {
                    piece_id: PieceId(1),
                    x: 0.0,
                    y: 0.0,
                    rotation: 0.0,
                }],
                utilization: 0.0,
                strip_length: 0.0,
            }],
            unplaced: Vec::new(),
        };
        let out = generate_all_sheets(
            &result,
            &[piece],
            &HashMap::new(),
            &tool(1, 6.0),
            &gcfg(),
            &ProfileCutConfig::default(),
            Some(&tool(1, 4.0)),
            &DrillConfig {
                tool_number: Some(1),
                ..DrillConfig::default()
            },
            Some(&tool(2, 6.0)),
            &PocketConfig {
                tool_number: Some(2),
                ..PocketConfig::default()
            },
        );
        let g = &out[0].1;
        assert!(g.contains("(Profile Operations)"));
        assert!(!g.contains("(Drill Operations)"));
        assert!(!g.contains("(Pocket Operations)"));
    }
}
