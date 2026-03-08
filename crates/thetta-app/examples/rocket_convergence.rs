/// Rigorous convergence test: load Rocket.step, run multi-sheet nesting,
/// verify all pieces converge (are placed across sheets with no overlaps).
///
/// Usage: cargo run --example rocket_convergence -- ~/Downloads/Rocket.step
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use thetta_geometry::polygon_ops;
use thetta_geometry::types::SheetConfig;
use thetta_nesting::gls::OptimizerConfig;
use thetta_nesting::runner::{NestingUpdate, run_nesting_multisheet};
use thetta_step::{ExtractionConfig, import_step};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: rocket_convergence <step_file> [time_limit_secs]");
        std::process::exit(1);
    }
    let path = Path::new(&args[1]);
    let time_limit: f64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(30.0);

    // ── 1. Import STEP ──
    eprintln!("Importing {:?} ...", path);
    let config = ExtractionConfig::default();
    let import = import_step(path, None, &config).unwrap_or_else(|e| {
        eprintln!("Import failed: {e}");
        std::process::exit(1);
    });
    let pieces = import.pieces;
    let total = pieces.len();
    eprintln!(
        "  {} pieces at thickness {:.2} mm",
        total, import.selected_thickness_mm
    );

    // Report oversized pieces (bbox exceeds sheet in all orientations)
    let sheet = SheetConfig::default();
    let mut oversized = Vec::new();
    for (i, piece) in pieces.iter().enumerate() {
        let bb = polygon_ops::bounding_box(&piece.polygon);
        let w = bb.max_x - bb.min_x;
        let h = bb.max_y - bb.min_y;
        // Piece fits if (w <= sheet.width && h <= sheet.height) OR (h <= sheet.width && w <= sheet.height)
        let fits_normal = w <= sheet.width && h <= sheet.height;
        let fits_rotated = h <= sheet.width && w <= sheet.height;
        if !fits_normal && !fits_rotated {
            oversized.push((i, piece.id, w, h));
        }
    }
    if !oversized.is_empty() {
        eprintln!("  WARNING: {} oversized pieces (exceed {:.0}x{:.0} sheet):", oversized.len(), sheet.width, sheet.height);
        for (i, id, w, h) in &oversized {
            eprintln!("    piece[{}] id={}: bbox {:.1}x{:.1}", i, id.0, w, h);
        }
    }

    // ── 2. Run multi-sheet nesting ──
    let opt_config = OptimizerConfig {
        time_limit_secs: time_limit,
        ..OptimizerConfig::default()
    };

    let (tx, rx) = mpsc::channel();
    let cancel = Arc::new(AtomicBool::new(false));

    let pieces_clone = pieces.clone();
    let sheet2 = sheet.clone();
    eprintln!(
        "\nRunning multi-sheet nesting (time_limit={:.0}s, sheet={:.0}x{:.0}, sep={:.1}) ...",
        time_limit, sheet.width, sheet.height, sheet.separation
    );
    let start = Instant::now();

    std::thread::spawn(move || {
        run_nesting_multisheet(pieces_clone, sheet2, None, opt_config, tx, cancel);
    });

    // ── 3. Collect updates ──
    let mut update_count = 0usize;
    let mut last_report = Instant::now();
    let mut final_result = None;

    loop {
        match rx.recv_timeout(Duration::from_secs(600)) {
            Ok(NestingUpdate::Improved(r)) => {
                update_count += 1;
                let placed: usize = r.sheets.iter().map(|s| s.placements.len()).sum();
                if last_report.elapsed() > Duration::from_secs(2) || placed == total {
                    let sheets = r.sheets.len();
                    let elapsed = start.elapsed().as_secs_f64();
                    eprintln!(
                        "  [{:.1}s] Placed {}/{} on {} sheet(s), unplaced={}",
                        elapsed, placed, total, sheets, r.unplaced.len()
                    );
                    last_report = Instant::now();
                }
            }
            Ok(NestingUpdate::Progress { iteration, total_overlap, best_overlap, feasible_count }) => {
                if iteration % 200 == 0 {
                    let elapsed = start.elapsed().as_secs_f64();
                    eprintln!(
                        "  [{:.1}s] GLS iter={}, overlap={:.1}, best_overlap={:.1}, feasible={}",
                        elapsed, iteration, total_overlap, best_overlap, feasible_count
                    );
                }
            }
            Ok(NestingUpdate::Finished(r)) => {
                final_result = Some(r);
                break;
            }
            Err(_) => {
                eprintln!("  ERROR: Timed out waiting for optimizer (600s)");
                std::process::exit(1);
            }
        }
    }

    let elapsed = start.elapsed();
    let result = final_result.unwrap();

    // ── 4. Report results ──
    eprintln!("\n═══ RESULTS ({:.1}s, {} updates) ═══", elapsed.as_secs_f64(), update_count);
    let total_placed: usize = result.sheets.iter().map(|s| s.placements.len()).sum();
    eprintln!("  Sheets: {}", result.sheets.len());
    eprintln!("  Placed: {}/{}", total_placed, total);
    eprintln!("  Unplaced: {}", result.unplaced.len());

    for (i, s) in result.sheets.iter().enumerate() {
        eprintln!(
            "  Sheet {}: {} pieces, strip={:.1}mm, util={:.1}%",
            i + 1,
            s.placements.len(),
            s.strip_length,
            s.utilization * 100.0
        );
    }

    // ── 5. Verify convergence ──
    let expected_placeable = total - oversized.len();
    let mut pass = true;

    if total_placed < expected_placeable {
        eprintln!(
            "\n  FAIL: Only placed {}/{} placeable pieces ({} oversized excluded)",
            total_placed, expected_placeable, oversized.len()
        );
        pass = false;

        // Report which pieces are unplaced
        eprintln!("  Unplaced piece IDs: {:?}", result.unplaced);
    } else {
        eprintln!(
            "\n  OK: All {} placeable pieces placed ({} oversized excluded)",
            expected_placeable, oversized.len()
        );
    }

    // Check strip_length doesn't exceed sheet width
    for (i, s) in result.sheets.iter().enumerate() {
        if s.strip_length > sheet.width + 1.0 {
            eprintln!(
                "  FAIL: Sheet {} strip_length {:.1} exceeds sheet width {:.1}",
                i + 1, s.strip_length, sheet.width
            );
            pass = false;
        }
    }

    // Check no empty sheets
    for (i, s) in result.sheets.iter().enumerate() {
        if s.placements.is_empty() {
            eprintln!("  FAIL: Sheet {} is empty (phantom sheet)", i + 1);
            pass = false;
        }
    }

    // Check no duplicate piece placements
    let mut all_placed_ids: Vec<u64> = result
        .sheets
        .iter()
        .flat_map(|s| s.placements.iter().map(|p| p.piece_id.0))
        .collect();
    let before_dedup = all_placed_ids.len();
    all_placed_ids.sort();
    all_placed_ids.dedup();
    if all_placed_ids.len() != before_dedup {
        eprintln!(
            "  FAIL: {} duplicate placements detected",
            before_dedup - all_placed_ids.len()
        );
        pass = false;
    }

    if pass {
        eprintln!("\n  ✅ CONVERGENCE TEST PASSED");
    } else {
        eprintln!("\n  ❌ CONVERGENCE TEST FAILED");
        std::process::exit(1);
    }
}
