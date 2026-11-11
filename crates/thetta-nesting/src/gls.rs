use crate::engine::NestingEngine;
use crate::sampler::{coordinate_descent_refine, uniform_sample};
use rand::SeedableRng;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use thetta_geometry::types::NestingResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

pub struct GLSOptimizer {
    pub engine: NestingEngine,
    pub penalty_weights: Vec<f64>,
    pub config: OptimizerConfig,
    pub best_feasible: Option<NestingResult>,
    pub rng: StdRng,
}

impl GLSOptimizer {
    pub fn new(engine: NestingEngine, config: OptimizerConfig) -> Self {
        let penalty_weights = vec![1.0; engine.piece_count()];
        Self {
            engine,
            penalty_weights,
            config,
            best_feasible: None,
            rng: StdRng::seed_from_u64(42),
        }
    }

    pub fn run<F, P>(
        &mut self,
        mut on_update: F,
        mut on_progress: P,
        stop_after: Duration,
        cancel: Option<&Arc<AtomicBool>>,
    ) -> NestingResult
    where
        F: FnMut(NestingResult),
        P: FnMut(usize, f64, f64, usize),
    {
        let began = Instant::now();
        let sheet = self.engine.sheet_config().clone();

        // Only randomize pieces that LBF left with collisions or unplaced.
        // Keep good LBF placements intact.
        for idx in 0..self.engine.piece_count() {
            let has_collision = self
                .engine
                .placement_of(idx)
                .map(|p| self.engine.collision_cost(idx, p.x, p.y, p.rotation) > 0.0)
                .unwrap_or(true); // unplaced → randomize
            if has_collision {
                let s = uniform_sample(&mut self.rng, &sheet, self.engine.rotation_range(idx));
                self.engine.force_place_item(idx, s.x, s.y, s.rotation);
            }
        }

        // If LBF already produced a feasible solution, record it
        let initial_overlap = total_overlap(&self.engine);
        if initial_overlap == 0.0 {
            self.engine.fit_strip();
            let result = self.engine.current_result();
            if result.unplaced.is_empty() {
                self.best_feasible = Some(result);
            }
        }

        let mut best_total_overlap = initial_overlap;
        let mut iteration: usize = 0;
        let mut feasible_count: usize = 0;

        let cancelled =
            |c: &Option<&Arc<AtomicBool>>| c.as_ref().is_some_and(|a| a.load(Ordering::Relaxed));

        while !cancelled(&cancel) && began.elapsed() < stop_after {
            iteration += 1;

            let mut best_idx = None;
            let mut best_weighted = 0.0_f64; // only consider pieces with actual collisions

            for idx in 0..self.engine.piece_count() {
                let cost = if let Some(p) = self.engine.placement_of(idx) {
                    self.engine.collision_cost(idx, p.x, p.y, p.rotation)
                } else {
                    1_000.0
                };
                let probe = cost * self.penalty_weights[idx];
                if probe > best_weighted {
                    best_weighted = probe;
                    best_idx = Some(idx);
                }
            }

            let Some(idx) = best_idx else {
                // All pieces are collision-free — GLS has nothing to improve
                break;
            };

            let mut sampled = Vec::with_capacity(self.config.samples_per_move);
            for _ in 0..self.config.samples_per_move {
                if cancelled(&cancel) || began.elapsed() >= stop_after {
                    break;
                }
                let sample = uniform_sample(&mut self.rng, &sheet, self.engine.rotation_range(idx));
                let score = self
                    .engine
                    .collision_cost(idx, sample.x, sample.y, sample.rotation);
                sampled.push((sample, score));
            }

            sampled.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
            let mut candidates = sampled.into_iter().take(5).collect::<Vec<_>>();
            let mut best_candidate = None;
            let mut best_candidate_score = f64::INFINITY;

            for (seed, _) in candidates.drain(..) {
                let refined = coordinate_descent_refine(
                    seed,
                    |s| self.engine.collision_cost(idx, s.x, s.y, s.rotation),
                    10.0,
                    self.config.refinement_depth,
                    self.engine.rotation_range(idx),
                );
                let score = self
                    .engine
                    .collision_cost(idx, refined.x, refined.y, refined.rotation);
                if score < best_candidate_score {
                    best_candidate = Some(refined);
                    best_candidate_score = score;
                }
            }

            if let Some(cand) = best_candidate {
                self.engine
                    .force_place_item(idx, cand.x, cand.y, cand.rotation);
            }

            for idx in 0..self.engine.piece_count() {
                if let Some(p) = self.engine.placement_of(idx) {
                    if self.engine.collision_cost(idx, p.x, p.y, p.rotation) > 0.0 {
                        self.penalty_weights[idx] += self.config.penalty_increment;
                    }
                } else {
                    self.penalty_weights[idx] += self.config.penalty_increment;
                }
            }

            let current_overlap = total_overlap(&self.engine);
            if current_overlap < best_total_overlap {
                best_total_overlap = current_overlap;
            }
            if current_overlap == 0.0 {
                self.engine.fit_strip();
                let feasible = self.engine.current_result();
                if feasible.unplaced.is_empty() {
                    feasible_count += 1;
                    let is_better = self
                        .best_feasible
                        .as_ref()
                        .map(|prev| {
                            let prev_strip = prev
                                .sheets
                                .first()
                                .map(|s| s.strip_length)
                                .unwrap_or(f64::MAX);
                            let new_strip = feasible
                                .sheets
                                .first()
                                .map(|s| s.strip_length)
                                .unwrap_or(f64::MAX);
                            new_strip < prev_strip
                        })
                        .unwrap_or(true);
                    if is_better {
                        self.best_feasible = Some(feasible.clone());
                        on_update(feasible);
                    }
                }
            }

            on_progress(
                iteration,
                current_overlap,
                best_total_overlap,
                feasible_count,
            );
        }

        self.best_feasible
            .clone()
            .unwrap_or_else(|| self.engine.current_result())
    }
}

fn total_overlap(engine: &NestingEngine) -> f64 {
    let mut total = 0.0;
    for idx in 0..engine.piece_count() {
        if let Some(p) = engine.placement_of(idx) {
            total += engine.collision_cost(idx, p.x, p.y, p.rotation);
        } else {
            total += 1_000.0;
        }
    }
    total
}
