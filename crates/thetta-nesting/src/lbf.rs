use crate::engine::NestingEngine;
use std::cmp::Ordering;
use thetta_geometry::types::NestingResult;

#[derive(Debug, Clone, Copy)]
pub struct LbfConfig {
    pub grid_n: usize,
    pub rotations_n: usize,
}

impl Default for LbfConfig {
    fn default() -> Self {
        Self {
            grid_n: 25,
            rotations_n: 12,
        }
    }
}

pub fn run_lbf(engine: &mut NestingEngine, config: LbfConfig) -> NestingResult {
    let mut order = (0..engine.piece_count()).collect::<Vec<_>>();
    order.sort_by(|a, b| {
        engine
            .piece_area(*b)
            .partial_cmp(&engine.piece_area(*a))
            .unwrap_or(Ordering::Equal)
    });

    let sheet = engine.sheet_config().clone();
    let grid_n = config.grid_n.max(2);
    let rotations_n = config.rotations_n.max(1);

    for idx in order {
        let mut best: Option<(f64, f64, f64)> = None;

        for gx in 0..grid_n {
            for gy in 0..grid_n {
                let x = if grid_n == 1 {
                    0.0
                } else {
                    sheet.width * gx as f64 / (grid_n - 1) as f64
                };
                let y = if grid_n == 1 {
                    0.0
                } else {
                    sheet.height * gy as f64 / (grid_n - 1) as f64
                };

                for r_idx in 0..rotations_n {
                    let r = (r_idx as f64) * std::f64::consts::TAU / rotations_n as f64;
                    if engine.collision_cost(idx, x, y, r) == 0.0 {
                        let cand = (y, x, r);
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
            let _ = engine.place_item(idx, x, y, r);
        }
    }

    engine.current_result()
}
