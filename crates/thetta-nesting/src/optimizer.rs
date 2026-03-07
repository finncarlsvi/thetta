use crate::gls::OptimizerConfig;
use crate::runner::{NestingUpdate, run_nesting};
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
    tx: mpsc::Sender<NestingUpdate>,
    cancel: Arc<AtomicBool>,
) {
    run_nesting(pieces, sheet_config, orientations, config, tx, cancel);
}
