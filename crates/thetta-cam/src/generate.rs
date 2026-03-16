use std::collections::HashMap;

use thetta_geometry::polygon_ops;
use thetta_geometry::types::{
    GcodeConfig, NestingResult, Piece, Placement, Point2D, ProfileCutConfig, Tab, Tool,
};

pub struct SheetGcodeInput<'a> {
    pub sheet_index: usize,
    pub placements: &'a [Placement],
    pub pieces: &'a [Piece],
    pub tabs: &'a HashMap<(usize, usize), Vec<Tab>>,
    pub tool: &'a Tool,
    pub gcode_config: &'a GcodeConfig,
    pub profile_config: &'a ProfileCutConfig,
}

pub fn generate_sheet_gcode(input: &SheetGcodeInput<'_>) -> String {
    let mut toolpaths = Vec::new();

    for (placement_idx, placement) in input.placements.iter().enumerate() {
        let Some(piece) = input.pieces.iter().find(|p| p.id == placement.piece_id) else {
            continue;
        };

        let rotated = polygon_ops::rotate(&piece.polygon, placement.rotation, Point2D { x: 0.0, y: 0.0 });
        let placed = polygon_ops::translate(&rotated, placement.x, placement.y);

        let empty_tabs = Vec::new();
        let tabs = input
            .tabs
            .get(&(input.sheet_index, placement_idx))
            .unwrap_or(&empty_tabs);

        let toolpath = crate::toolpath::plan_profile_toolpath(
            &placed,
            tabs,
            input.tool,
            input.gcode_config,
            input.profile_config,
        );
        toolpaths.push(toolpath);
    }

    crate::gcode::emit_gcode(&toolpaths, input.tool, input.gcode_config, input.sheet_index)
}

pub fn generate_all_sheets(
    result: &NestingResult,
    pieces: &[Piece],
    tabs: &HashMap<(usize, usize), Vec<Tab>>,
    tool: &Tool,
    gcode_config: &GcodeConfig,
    profile_config: &ProfileCutConfig,
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
                tool,
                gcode_config,
                profile_config,
            };
            (sheet_index, generate_sheet_gcode(&input))
        })
        .collect()
}
