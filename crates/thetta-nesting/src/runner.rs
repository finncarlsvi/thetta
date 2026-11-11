use crate::compression::compress_strip;
use crate::conversion::default_orientations;
use crate::engine::NestingEngine;
use crate::gls::{GLSOptimizer, OptimizerConfig};
use crate::lbf::{LbfConfig, run_lbf, run_lbf_multisheet};
use crate::multi_sheet::MultiSheetEngine;
use jagua_rs::geometry::geo_enums::RotationRange;
use std::cmp::Ordering as CmpOrdering;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::time::Duration;
use thetta_geometry::types::{NestingResult, Piece, Placement, SheetConfig};

#[derive(Debug, Clone)]
pub enum NestingUpdate {
    Improved(NestingResult),
    Finished(NestingResult),
    Progress {
        iteration: usize,
        total_overlap: f64,
        best_overlap: f64,
        feasible_count: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NestingMode {
    Strip,
    Sheet,
}

pub fn run_nesting(
    pieces: Vec<Piece>,
    sheet: SheetConfig,
    orientations: Option<Vec<RotationRange>>,
    config: OptimizerConfig,
    tx: mpsc::Sender<NestingUpdate>,
    cancel: Arc<AtomicBool>,
) {
    let mut engine = NestingEngine::new_with_orientations(&pieces, &sheet, orientations.as_deref());
    let tx_lbf = tx.clone();
    let baseline = run_lbf(
        &mut engine,
        LbfConfig {
            orientations: orientations.clone(),
            ..LbfConfig::default()
        },
        Some(&cancel),
        |eng| {
            let _ = tx_lbf.send(NestingUpdate::Improved(eng.current_result()));
        },
    );
    let _ = tx.send(NestingUpdate::Improved(baseline.clone()));

    if cancel.load(Ordering::Relaxed) {
        let _ = tx.send(NestingUpdate::Finished(baseline));
        return;
    }

    let mut gls = GLSOptimizer::new(engine, config.clone());
    let tx2 = tx.clone();
    let gls_result = gls.run(
        |r| {
            let _ = tx.send(NestingUpdate::Improved(r));
        },
        move |iteration, total_overlap, best_overlap, feasible_count| {
            let _ = tx2.send(NestingUpdate::Progress {
                iteration,
                total_overlap,
                best_overlap,
                feasible_count,
            });
        },
        Duration::MAX,
        Some(&cancel),
    );

    // Use GLS result only if feasible, otherwise fall back to LBF baseline
    let best_so_far = if gls.best_feasible.is_some() {
        gls_result
    } else {
        baseline
    };

    if cancel.load(Ordering::Relaxed) {
        let _ = tx.send(NestingUpdate::Finished(best_so_far));
        return;
    }

    let compressed = compress_strip(
        &pieces,
        sheet,
        config.compression_step,
        orientations.as_deref(),
        Some(&cancel),
        None, // strip mode: allow expansion beyond sheet width
    );
    let _ = tx.send(NestingUpdate::Improved(compressed.clone()));
    let _ = tx.send(NestingUpdate::Finished(compressed));
}

pub fn run_nesting_multisheet(
    pieces: Vec<Piece>,
    sheet: SheetConfig,
    orientations: Option<Vec<RotationRange>>,
    config: OptimizerConfig,
    tx: mpsc::Sender<NestingUpdate>,
    cancel: Arc<AtomicBool>,
) {
    let resolved_orientations = orientations
        .clone()
        .unwrap_or_else(|| default_orientations(pieces.len()));
    let mut ms_engine = MultiSheetEngine::new(&pieces, &sheet, &resolved_orientations);

    let tx_lbf = tx.clone();
    let baseline = run_lbf_multisheet(
        &mut ms_engine,
        LbfConfig {
            orientations: Some(resolved_orientations.clone()),
            ..LbfConfig::default()
        },
        Some(&cancel),
        |eng| {
            let _ = tx_lbf.send(NestingUpdate::Improved(eng.current_result()));
        },
    );
    let _ = tx.send(NestingUpdate::Improved(baseline.clone()));

    if cancel.load(Ordering::Relaxed) {
        let _ = tx.send(NestingUpdate::Finished(baseline));
        return;
    }

    let total_budget = Duration::from_secs_f64(config.time_limit_secs.max(0.1));
    let sheet_count = ms_engine.sheet_count().max(1);
    let mut per_sheet_secs = total_budget.as_secs_f64() / sheet_count as f64;
    if !per_sheet_secs.is_finite() || per_sheet_secs <= 0.0 {
        per_sheet_secs = 0.1;
    }
    let per_sheet_budget = Duration::from_secs_f64(per_sheet_secs);
    let mut best_result = baseline;

    for sheet_idx in 0..ms_engine.sheet_count() {
        if cancel.load(Ordering::Relaxed) {
            break;
        }

        let sheet_piece_indices = ms_engine.pieces_on_sheet(sheet_idx);
        if sheet_piece_indices.is_empty() {
            continue;
        }

        let sub_pieces = sheet_piece_indices
            .iter()
            .map(|&idx| pieces[idx].clone())
            .collect::<Vec<_>>();
        let sub_orientations = sheet_piece_indices
            .iter()
            .map(|&idx| resolved_orientations[idx].clone())
            .collect::<Vec<_>>();
        let sub_placements =
            collect_sub_placements(ms_engine.sheet_engine(sheet_idx), &sheet_piece_indices);

        let mut sub_engine =
            NestingEngine::new_with_orientations(&sub_pieces, &sheet, Some(&sub_orientations));
        for (local_idx, placement) in &sub_placements {
            sub_engine.force_place_item(*local_idx, placement.x, placement.y, placement.rotation);
        }

        let mut gls = GLSOptimizer::new(sub_engine, config.clone());
        let tx_progress = tx.clone();
        let gls_result = gls.run(
            |_| {},
            move |iteration, total_overlap, best_overlap, feasible_count| {
                let _ = tx_progress.send(NestingUpdate::Progress {
                    iteration,
                    total_overlap,
                    best_overlap,
                    feasible_count,
                });
            },
            per_sheet_budget,
            Some(&cancel),
        );

        let candidate = if gls.best_feasible.is_some() {
            gls_result
        } else {
            continue;
        };

        write_back_subsheet(
            &mut ms_engine,
            sheet_idx,
            &sheet_piece_indices,
            &sub_pieces,
            candidate.sheets.first(),
        );
        best_result = ms_engine.current_result();
        let _ = tx.send(NestingUpdate::Improved(best_result.clone()));
    }

    if cancel.load(Ordering::Relaxed) {
        let _ = tx.send(NestingUpdate::Finished(best_result));
        return;
    }

    for sheet_idx in 0..ms_engine.sheet_count() {
        if cancel.load(Ordering::Relaxed) {
            break;
        }

        let sheet_piece_indices = ms_engine.pieces_on_sheet(sheet_idx);
        if sheet_piece_indices.is_empty() {
            continue;
        }

        let sub_pieces = sheet_piece_indices
            .iter()
            .map(|&idx| pieces[idx].clone())
            .collect::<Vec<_>>();
        let sub_orientations = sheet_piece_indices
            .iter()
            .map(|&idx| resolved_orientations[idx].clone())
            .collect::<Vec<_>>();

        let compressed = compress_strip(
            &sub_pieces,
            sheet.clone(),
            config.compression_step,
            Some(&sub_orientations),
            Some(&cancel),
            Some(sheet.width), // multi-sheet: cap at sheet width
        );
        let strip_fits = compressed
            .sheets
            .first()
            .map_or(false, |s| s.strip_length <= sheet.width + 1.0);
        if compressed.unplaced.is_empty() && strip_fits {
            write_back_subsheet(
                &mut ms_engine,
                sheet_idx,
                &sheet_piece_indices,
                &sub_pieces,
                compressed.sheets.first(),
            );
            best_result = ms_engine.current_result();
        }
        // Always send a progress update after each sheet's compression so
        // the receiver doesn't starve waiting for messages.
        let _ = tx.send(NestingUpdate::Improved(best_result.clone()));
    }

    if !cancel.load(Ordering::Relaxed) {
        consolidate_multisheet(
            &mut ms_engine,
            &pieces,
            &sheet,
            config.compression_step,
            &resolved_orientations,
            &tx,
            &cancel,
            &mut best_result,
        );
    }

    let _ = tx.send(NestingUpdate::Finished(best_result));
}

fn collect_sub_placements(
    engine: &NestingEngine,
    sheet_piece_indices: &[usize],
) -> Vec<(usize, Placement)> {
    sheet_piece_indices
        .iter()
        .enumerate()
        .filter_map(|(local_idx, &global_idx)| {
            engine.placement_of(global_idx).map(|p| (local_idx, p))
        })
        .collect()
}

fn write_back_subsheet(
    ms_engine: &mut MultiSheetEngine,
    sheet_idx: usize,
    sheet_piece_indices: &[usize],
    sub_pieces: &[Piece],
    sheet: Option<&thetta_geometry::types::Sheet>,
) {
    let Some(sheet) = sheet else {
        return;
    };

    for &global_idx in sheet_piece_indices {
        ms_engine.remove_piece(global_idx);
    }

    for (local_idx, &global_idx) in sheet_piece_indices.iter().enumerate() {
        let piece_id = sub_pieces[local_idx].id;
        if let Some(placement) = sheet.placements.iter().find(|p| p.piece_id == piece_id) {
            ms_engine.force_place_on_sheet(
                global_idx,
                sheet_idx,
                placement.x,
                placement.y,
                placement.rotation,
            );
        }
    }
}

fn consolidate_multisheet(
    ms_engine: &mut MultiSheetEngine,
    pieces: &[Piece],
    sheet: &SheetConfig,
    compression_step: f64,
    orientations: &[RotationRange],
    tx: &mpsc::Sender<NestingUpdate>,
    cancel: &Arc<AtomicBool>,
    best_result: &mut NestingResult,
) {
    const MAX_PASSES: usize = 4;
    const AREA_UTILIZATION_CUTOFF: f64 = 0.995;
    let sheet_area_limit = sheet.width * sheet.height * AREA_UTILIZATION_CUTOFF;

    for _ in 0..MAX_PASSES {
        if cancel.load(Ordering::Relaxed) {
            break;
        }

        let mut moved_any = false;
        let sheet_count = ms_engine.sheet_count();
        if sheet_count <= 1 {
            break;
        }

        for source_sheet in (1..sheet_count).rev() {
            if cancel.load(Ordering::Relaxed) {
                break;
            }

            let mut source_pieces = ms_engine.pieces_on_sheet(source_sheet);
            if source_pieces.is_empty() {
                continue;
            }

            let mut merged_entire_sheet = false;
            for target_sheet in 0..source_sheet {
                if cancel.load(Ordering::Relaxed) {
                    break;
                }
                let mut combined_indices = ms_engine.pieces_on_sheet(target_sheet);
                if combined_indices.is_empty() {
                    continue;
                }
                combined_indices.extend(source_pieces.iter().copied());

                if try_repack_into_sheet(
                    ms_engine,
                    pieces,
                    sheet,
                    compression_step,
                    orientations,
                    cancel,
                    target_sheet,
                    &combined_indices,
                ) {
                    merged_entire_sheet = true;
                    moved_any = true;
                    *best_result = ms_engine.current_result();
                    let _ = tx.send(NestingUpdate::Improved(best_result.clone()));
                    break;
                }
            }
            if merged_entire_sheet {
                continue;
            }

            // Small parts are easier to migrate into remaining gaps first.
            source_pieces.sort_by(|a, b| {
                ms_engine
                    .piece_area(*a)
                    .partial_cmp(&ms_engine.piece_area(*b))
                    .unwrap_or(CmpOrdering::Equal)
            });

            for piece_idx in source_pieces {
                if cancel.load(Ordering::Relaxed) {
                    break;
                }
                if ms_engine.assignment_of(piece_idx) != Some(source_sheet) {
                    continue;
                }

                let piece_area = ms_engine.piece_area(piece_idx);

                let mut moved_piece = false;
                for target_sheet in 0..source_sheet {
                    let mut target_piece_indices = ms_engine.pieces_on_sheet(target_sheet);
                    let target_area = target_piece_indices
                        .iter()
                        .map(|idx| ms_engine.piece_area(*idx))
                        .sum::<f64>();
                    if target_area + piece_area > sheet_area_limit {
                        continue;
                    }

                    target_piece_indices.push(piece_idx);
                    if try_repack_into_sheet(
                        ms_engine,
                        pieces,
                        sheet,
                        compression_step,
                        orientations,
                        cancel,
                        target_sheet,
                        &target_piece_indices,
                    ) {
                        moved_piece = true;
                        break;
                    }
                }

                if !moved_piece {
                    continue;
                }
                moved_any = true;
                *best_result = ms_engine.current_result();
                let _ = tx.send(NestingUpdate::Improved(best_result.clone()));

                if ms_engine.pieces_on_sheet(source_sheet).is_empty() {
                    break;
                }
            }
        }

        if !moved_any {
            break;
        }
    }
}

fn try_repack_into_sheet(
    ms_engine: &mut MultiSheetEngine,
    pieces: &[Piece],
    sheet: &SheetConfig,
    compression_step: f64,
    orientations: &[RotationRange],
    cancel: &Arc<AtomicBool>,
    target_sheet: usize,
    piece_indices: &[usize],
) -> bool {
    let sub_pieces = piece_indices
        .iter()
        .map(|&idx| pieces[idx].clone())
        .collect::<Vec<_>>();
    let sub_orientations = piece_indices
        .iter()
        .map(|&idx| orientations[idx].clone())
        .collect::<Vec<_>>();

    let candidate = compress_strip(
        &sub_pieces,
        sheet.clone(),
        compression_step,
        Some(&sub_orientations),
        Some(cancel),
        Some(sheet.width),
    );
    let strip_fits = candidate
        .sheets
        .first()
        .map_or(false, |s| s.strip_length <= sheet.width + 1.0);
    if !candidate.unplaced.is_empty() || !strip_fits {
        return false;
    }

    write_back_subsheet(
        ms_engine,
        target_sheet,
        piece_indices,
        &sub_pieces,
        candidate.sheets.first(),
    );
    true
}
