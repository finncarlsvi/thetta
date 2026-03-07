use crate::engine::NestingEngine;
use crate::lbf::{LbfConfig, run_lbf};
use thetta_geometry::types::NestingResult;

pub fn compress_strip(
    pieces: &[thetta_geometry::types::Piece],
    mut sheet: thetta_geometry::types::SheetConfig,
    compression_step: f64,
) -> NestingResult {
    let mut best = {
        let mut engine = NestingEngine::new(pieces, &sheet);
        run_lbf(&mut engine, LbfConfig::default())
    };

    let mut low = (sheet.width * 0.2).max(1.0);
    let mut high = sheet.width;
    for _ in 0..8 {
        let mid = (low + high) * 0.5;
        sheet.width = mid;
        let mut engine = NestingEngine::new(pieces, &sheet);
        let candidate = run_lbf(&mut engine, LbfConfig::default());
        if candidate.unplaced.is_empty() {
            best = candidate;
            high = (mid - compression_step).max(1.0);
        } else {
            low = mid + compression_step;
        }
    }

    best
}
