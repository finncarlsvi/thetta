use crate::compression::compress_strip;
use crate::engine::NestingEngine;
use crate::gls::{GLSOptimizer, OptimizerConfig};
use crate::lbf::{LbfConfig, run_lbf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::time::Duration;
use thetta_geometry::types::{NestingResult, Piece, SheetConfig};

#[derive(Debug, Clone)]
pub enum NestingUpdate {
    Improved(NestingResult),
    Finished(NestingResult),
}

pub fn run_nesting(
    pieces: Vec<Piece>,
    sheet: SheetConfig,
    config: OptimizerConfig,
    tx: mpsc::Sender<NestingUpdate>,
    cancel: Arc<AtomicBool>,
) {
    let mut engine = NestingEngine::new(&pieces, &sheet);
    let baseline = run_lbf(&mut engine, LbfConfig::default());
    let _ = tx.send(NestingUpdate::Improved(baseline.clone()));

    if cancel.load(Ordering::Relaxed) {
        let _ = tx.send(NestingUpdate::Finished(baseline));
        return;
    }

    let mut gls = GLSOptimizer::new(engine, config.clone());
    let stop_after = Duration::from_secs_f64(config.time_limit_secs.max(0.1));
    let gls_result = gls.run(
        |r| {
            let _ = tx.send(NestingUpdate::Improved(r));
        },
        stop_after,
        Some(&cancel),
    );

    if cancel.load(Ordering::Relaxed) {
        let _ = tx.send(NestingUpdate::Finished(gls_result));
        return;
    }

    let compressed = compress_strip(&pieces, sheet, config.compression_step);
    let _ = tx.send(NestingUpdate::Improved(compressed.clone()));
    let _ = tx.send(NestingUpdate::Finished(compressed));
}
