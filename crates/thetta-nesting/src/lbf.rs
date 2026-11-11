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

        if let Some((x, y, r)) = find_lbf_position(engine, idx, &rotations, &config, false, cancel)
        {
            let _ = engine.place_item(idx, x, y, r);
            on_placed(engine);
        }
    }

    compact_left(engine);
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
        let mut placed_sheet_idx = None;
        for sheet_idx in 0..engine.sheet_count() {
            let candidate = {
                let sheet_engine = engine.sheet_engine_mut(sheet_idx);
                find_lbf_position(sheet_engine, idx, &rotations, &config, true, cancel)
            };
            if let Some((x, y, r)) = candidate {
                placed = engine.place_on_sheet(idx, sheet_idx, x, y, r);
                if placed {
                    placed_sheet_idx = Some(sheet_idx);
                    break;
                }
            }
        }

        if !placed && !is_cancelled() {
            let new_sheet_idx = engine.open_sheet();
            let candidate = {
                let sheet_engine = engine.sheet_engine_mut(new_sheet_idx);
                find_lbf_position(sheet_engine, idx, &rotations, &config, true, cancel)
            };
            if let Some((x, y, r)) = candidate {
                placed = engine.place_on_sheet(idx, new_sheet_idx, x, y, r);
                if placed {
                    placed_sheet_idx = Some(new_sheet_idx);
                }
            }
        }

        if placed {
            if let Some(sheet_idx) = placed_sheet_idx {
                compact_left(engine.sheet_engine_mut(sheet_idx));
            }
            on_progress(engine);
        }
    }

    for sheet_idx in 0..engine.sheet_count() {
        let sheet_engine = engine.sheet_engine_mut(sheet_idx);
        compact_left(sheet_engine);
        sheet_engine.fit_strip();
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
    cancel: Option<&Arc<AtomicBool>>,
) -> Option<(f64, f64, f64)> {
    let sheet = engine.sheet_config().clone();
    let grid_n = if fixed_width {
        config.grid_n.max(35)
    } else {
        config.grid_n.max(2)
    };
    let attempts = if fixed_width { 8 } else { 20 };
    let fixed_offsets = [
        (0.0, 0.0),
        (0.5, 0.0),
        (0.0, 0.5),
        (0.5, 0.5),
        (0.25, 0.25),
        (0.75, 0.25),
        (0.25, 0.75),
        (0.75, 0.75),
    ];

    let mut global_best: Option<(f64, f64, f64)> = None;
    'attempts: for attempt in 0..attempts {
        let current_width = engine.strip_width();
        let mut best: Option<(f64, f64, f64)> = None;
        let x_step = current_width / (grid_n - 1) as f64;
        let y_step = sheet.height / (grid_n - 1) as f64;
        let (x_offset, y_offset) = if fixed_width {
            fixed_offsets[attempt % fixed_offsets.len()]
        } else {
            (0.0, 0.0)
        };

        // Visit candidates in the same (y, x) order used by the objective.
        // The first feasible cell is therefore this attempt's optimum; scanning
        // the remaining cells cannot improve it. Check cancellation within the
        // scan so a difficult part cannot consume the consolidation budget.
        'grid: for gy in 0..grid_n {
            if cancel.is_some_and(|c| c.load(AtomicOrdering::Relaxed)) {
                break 'attempts;
            }
            for gx in 0..grid_n {
                let x = (x_step * (gx as f64 + x_offset)).min(current_width);
                let y = (y_step * (gy as f64 + y_offset)).min(sheet.height);

                for rotation in rotations {
                    if engine.collision_cost(piece_idx, x, y, *rotation) == 0.0 {
                        best = Some((y, x, *rotation));
                        break 'grid;
                    }
                }
            }
        }

        if let Some(candidate) = best {
            let better = global_best
                .map(|g| candidate.0 < g.0 || (candidate.0 == g.0 && candidate.1 < g.1))
                .unwrap_or(true);
            if better {
                global_best = Some(candidate);
            }
        }

        if fixed_width {
            continue;
        }

        engine.change_strip_width(current_width * 1.15 + 1.0);
    }

    global_best.map(|(y, x, r)| (x, y, r))
}

fn compact_left(engine: &mut NestingEngine) {
    let mut placed = (0..engine.piece_count())
        .filter_map(|idx| engine.placement_of(idx).map(|p| (idx, p)))
        .collect::<Vec<_>>();
    placed.sort_by(|a, b| a.1.x.partial_cmp(&b.1.x).unwrap_or(Ordering::Equal));

    for (idx, placement) in placed {
        let original_x = placement.x.max(0.0);
        engine.remove_item(idx);

        let mut low = 0.0_f64;
        let mut high = original_x;
        let mut best = original_x;
        for _ in 0..22 {
            let mid = (low + high) * 0.5;
            if engine.collision_cost(idx, mid, placement.y, placement.rotation) == 0.0 {
                best = mid;
                high = mid;
            } else {
                low = mid;
            }
        }

        engine.force_place_item(idx, best, placement.y, placement.rotation);
    }
}
