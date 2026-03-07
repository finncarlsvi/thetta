use std::sync::mpsc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use thetta_geometry::types::{Piece, SheetConfig, NestingResult, Sheet, Placement};

pub struct OptimizerConfig {
    pub samples_per_move: usize,
    pub refinement_depth: usize,
    pub penalty_increment: f64,
    pub compression_step: f64,
    pub time_limit_secs: f64,
}

impl Default for OptimizerConfig {
    fn default() -> Self {
        Self {
            samples_per_move: 1000,
            refinement_depth: 5,
            penalty_increment: 0.1,
            compression_step: 5.0,
            time_limit_secs: 60.0,
        }
    }
}

pub enum NestingUpdate {
    Improved(NestingResult),
    Finished(NestingResult),
}

pub fn run_optimizer(
    _pieces: Vec<Piece>,
    _sheet_config: SheetConfig,
    _config: OptimizerConfig,
    _tx: mpsc::Sender<NestingUpdate>,
    _cancel: Arc<AtomicBool>,
) {
    todo!("GLS optimizer with jagua-rs integration")
}
