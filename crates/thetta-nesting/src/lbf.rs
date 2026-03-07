use crate::conversion::rotation_range_to_samples;
use crate::engine::NestingEngine;
use jagua_rs::geometry::geo_enums::RotationRange;
use std::cmp::Ordering;
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
            grid_n: 25,
            rotations_n: 12,
            orientations: None,
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

    for idx in order {
        let rotation_range = config
            .orientations
            .as_ref()
            .and_then(|v| v.get(idx))
            .or_else(|| engine.rotation_range(idx));
        let rotations = rotation_range
            .map(|r| rotation_range_to_samples(r, config.rotations_n))
            .unwrap_or_else(|| rotation_range_to_samples(&RotationRange::Continuous, config.rotations_n));

        let mut placed = false;
        for _ in 0..20 {
            let mut best: Option<(f64, f64, f64)> = None;
            let current_width = engine.strip_width();

            for gx in 0..grid_n {
                for gy in 0..grid_n {
                    let x = current_width * gx as f64 / (grid_n - 1) as f64;
                    let y = sheet.height * gy as f64 / (grid_n - 1) as f64;

                    for r in &rotations {
                        if engine.collision_cost(idx, x, y, *r) == 0.0 {
                            let cand = (y, x, *r);
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
                placed = true;
                break;
            }

            engine.change_strip_width(current_width * 1.15 + 1.0);
        }

        if !placed {
            continue;
        }
    }

    engine.fit_strip();
    engine.current_result()
}
