use crate::gls::OptimizerConfig;
use crate::runner::{NestingMode, NestingUpdate, run_nesting, run_nesting_multisheet};
use jagua_rs::geometry::geo_enums::RotationRange;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use thetta_geometry::types::{Piece, SheetConfig};

pub fn run_optimizer(
    pieces: Vec<Piece>,
    sheet_config: SheetConfig,
    orientations: Option<Vec<RotationRange>>,
    config: OptimizerConfig,
    mode: NestingMode,
    tx: mpsc::Sender<NestingUpdate>,
    cancel: Arc<AtomicBool>,
) {
    match mode {
        NestingMode::Strip => run_nesting(pieces, sheet_config, orientations, config, tx, cancel),
        NestingMode::Sheet => {
            run_nesting_multisheet(pieces, sheet_config, orientations, config, tx, cancel)
        }
    }
}
