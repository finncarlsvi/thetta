use crate::conversion::rotation_range_to_samples;
use crate::engine::NestingEngine;
use crate::multi_sheet::MultiSheetEngine;
use jagua_rs::geometry::geo_enums::RotationRange;
use std::cmp::Ordering;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
use thetta_geometry::types::NestingResult;

#[derive(Debug, Clone)]
pub struct LbfConfig {
    pub grid_n: usize,
    pub rotations_n: usize,
    pub orientations: Option<Vec<RotationRange>>,
}

impl Default for LbfConfig {
    fn default() -> Self {
        Self {
            grid_n: 15,
            rotations_n: 8,
            orientations: None,
        }
    }
}

pub fn run_lbf(
    engine: &mut NestingEngine,
    config: LbfConfig,
    cancel: Option<&Arc<AtomicBool>>,
    mut on_placed: impl FnMut(&NestingEngine),
) -> NestingResult {
    let mut order = (0..engine.piece_count()).collect::<Vec<_>>();
    order.sort_by(|a, b| {
        engine
            .piece_area(*b)
            .partial_cmp(&engine.piece_area(*a))
            .unwrap_or(Ordering::Equal)
    });

    let is_cancelled = || {
        cancel
            .as_ref()
            .is_some_and(|c| c.load(AtomicOrdering::Relaxed))
    };

    for idx in order {
        if is_cancelled() {
            break;
        }

        let rotation_range = config
            .orientations
            .as_ref()
            .and_then(|v| v.get(idx))
            .or_else(|| engine.rotation_range(idx));
        let rotations = rotation_range
            .map(|r| rotation_range_to_samples(r, config.rotations_n))
            .unwrap_or_else(|| {
                rotation_range_to_samples(&RotationRange::Continuous, config.rotations_n)
            });

        if let Some((x, y, r)) = find_lbf_position(engine, idx, &rotations, &config, false) {
            let _ = engine.place_item(idx, x, y, r);
            on_placed(engine);
        }
    }

    engine.fit_strip();
    engine.current_result()
}

pub fn run_lbf_multisheet(
    engine: &mut MultiSheetEngine,
    config: LbfConfig,
    cancel: Option<&Arc<AtomicBool>>,
    mut on_progress: impl FnMut(&MultiSheetEngine),
) -> NestingResult {
    let mut order = (0..engine.piece_count()).collect::<Vec<_>>();
    order.sort_by(|a, b| {
        engine
            .piece_area(*b)
            .partial_cmp(&engine.piece_area(*a))
            .unwrap_or(Ordering::Equal)
    });

    let is_cancelled = || {
        cancel
            .as_ref()
            .is_some_and(|c| c.load(AtomicOrdering::Relaxed))
    };

    if engine.sheet_count() == 0 {
        let _ = engine.open_sheet();
    }

    for idx in order {
        if is_cancelled() {
            break;
        }

        let rotation_range = config
            .orientations
            .as_ref()
            .and_then(|v| v.get(idx))
            .or_else(|| engine.rotation_range(idx));
        let rotations = rotation_range
            .map(|r| rotation_range_to_samples(r, config.rotations_n))
            .unwrap_or_else(|| {
                rotation_range_to_samples(&RotationRange::Continuous, config.rotations_n)
            });

        let mut placed = false;
        for sheet_idx in 0..engine.sheet_count() {
            let candidate = {
                let sheet_engine = engine.sheet_engine_mut(sheet_idx);
                find_lbf_position(sheet_engine, idx, &rotations, &config, true)
            };
            if let Some((x, y, r)) = candidate {
                placed = engine.place_on_sheet(idx, sheet_idx, x, y, r);
                if placed {
                    break;
                }
            }
        }

        if !placed {
            let new_sheet_idx = engine.open_sheet();
            let candidate = {
                let sheet_engine = engine.sheet_engine_mut(new_sheet_idx);
                find_lbf_position(sheet_engine, idx, &rotations, &config, true)
            };
            if let Some((x, y, r)) = candidate {
                placed = engine.place_on_sheet(idx, new_sheet_idx, x, y, r);
            }
        }

        if placed {
            on_progress(engine);
        }
    }

    for sheet_idx in 0..engine.sheet_count() {
        engine.sheet_engine_mut(sheet_idx).fit_strip();
    }

    engine.current_result()
}

/// Find the best LBF position for one piece in one engine.
/// Returns Some((x, y, rotation)) if found.
fn find_lbf_position(
    engine: &mut NestingEngine,
    piece_idx: usize,
    rotations: &[f64],
    config: &LbfConfig,
    fixed_width: bool,
) -> Option<(f64, f64, f64)> {
    let sheet = engine.sheet_config().clone();
    let grid_n = config.grid_n.max(2);
    let attempts = if fixed_width { 1 } else { 20 };

    for _ in 0..attempts {
        let current_width = engine.strip_width();
        let mut best: Option<(f64, f64, f64)> = None;

        for gx in 0..grid_n {
            for gy in 0..grid_n {
                let x = current_width * gx as f64 / (grid_n - 1) as f64;
                let y = sheet.height * gy as f64 / (grid_n - 1) as f64;

                for rotation in rotations {
                    if engine.collision_cost(piece_idx, x, y, *rotation) == 0.0 {
                        let cand = (y, x, *rotation);
                        let better = best
                            .map(|b| cand.0 < b.0 || (cand.0 == b.0 && cand.1 < b.1))
                            .unwrap_or(true);
                        if better {
                            best = Some(cand);
                        }
                    }
                }
            }
        }

        if let Some((y, x, r)) = best {
            return Some((x, y, r));
        }

        if fixed_width {
            break;
        }

        engine.change_strip_width(current_width * 1.15 + 1.0);
    }

    None
}
