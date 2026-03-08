use crate::engine::NestingEngine;
use crate::lbf::{LbfConfig, run_lbf};
use jagua_rs::geometry::geo_enums::RotationRange;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use thetta_geometry::types::{NestingResult, Piece, SheetConfig};

pub fn compress_strip(
    pieces: &[Piece],
    sheet: SheetConfig,
    compression_step: f64,
    orientations: Option<&[RotationRange]>,
    cancel: Option<&Arc<AtomicBool>>,
    max_strip_width: Option<f64>,
) -> NestingResult {
    let is_cancelled = || cancel.as_ref().is_some_and(|c| c.load(Ordering::Relaxed));

    let mut engine = NestingEngine::new_with_orientations(pieces, &sheet, orientations);
    let mut best = run_lbf(
        &mut engine,
        LbfConfig {
            orientations: orientations.map(|o| o.to_vec()),
            ..LbfConfig::default()
        },
        cancel,
        |_| {},
    );

    // If a max width is set and the initial LBF exceeded it, discard that
    // result — it would place pieces beyond the sheet boundary.
    if let Some(max_w) = max_strip_width {
        if engine.strip_width() > max_w + 1.0 {
            // Reset to sheet width and re-run with fixed width so pieces stay
            // within bounds. Use the sheet width as the strip width.
            engine.change_strip_width(max_w);
            engine.clear_placements();
            best = run_lbf(
                &mut engine,
                LbfConfig {
                    orientations: orientations.map(|o| o.to_vec()),
                    ..LbfConfig::default()
                },
                cancel,
                |_| {},
            );
        }
    }

    let mut low = (sheet.width * 0.2).max(1.0);
    let mut high = if let Some(max_w) = max_strip_width {
        engine.strip_width().max(1.0).min(max_w)
    } else {
        engine.strip_width().max(1.0)
    };

    for _ in 0..8 {
        if is_cancelled() {
            break;
        }

        let mid = (low + high) * 0.5;
        engine.change_strip_width(mid);
        engine.clear_placements();

        let candidate = run_lbf(
            &mut engine,
            LbfConfig {
                orientations: orientations.map(|o| o.to_vec()),
                ..LbfConfig::default()
            },
            cancel,
            |_| {},
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
