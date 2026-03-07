use crate::engine::NestingEngine;
use crate::lbf::{LbfConfig, run_lbf};
use jagua_rs::geometry::geo_enums::RotationRange;
use thetta_geometry::types::{NestingResult, Piece, SheetConfig};

pub fn compress_strip(
    pieces: &[Piece],
    sheet: SheetConfig,
    compression_step: f64,
    orientations: Option<&[RotationRange]>,
) -> NestingResult {
    let mut engine = NestingEngine::new_with_orientations(pieces, &sheet, orientations);
    let mut best = run_lbf(
        &mut engine,
        LbfConfig {
            orientations: orientations.map(|o| o.to_vec()),
            ..LbfConfig::default()
        },
    );

    let mut low = (sheet.width * 0.2).max(1.0);
    let mut high = engine.strip_width().max(1.0);

    for _ in 0..8 {
        let mid = (low + high) * 0.5;
        engine.change_strip_width(mid);
        engine.clear_placements();

        let candidate = run_lbf(
            &mut engine,
            LbfConfig {
                orientations: orientations.map(|o| o.to_vec()),
                ..LbfConfig::default()
            },
        );

        if candidate.unplaced.is_empty() {
            best = candidate;
            high = (mid - compression_step).max(1.0);
        } else {
            low = mid + compression_step;
        }
    }

    best
}
