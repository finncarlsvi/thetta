/// Diagnostic: load STEP, extract pieces, run LBF, and report placement results.
/// Usage: cargo run --example test_nesting -- /path/to/file.step [thickness_mm]
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use thetta_geometry::types::SheetConfig;
use thetta_nesting::gls::OptimizerConfig;
use thetta_nesting::runner::{NestingUpdate, run_nesting};
use thetta_step::{ExtractionConfig, import_step};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: test_nesting <step_file> [thickness_mm]");
        std::process::exit(1);
    }
    let path = Path::new(&args[1]);
    let thickness: Option<f64> = args.get(2).and_then(|s| s.parse().ok());
    let config = ExtractionConfig::default();

    eprintln!("Loading {:?} ...", path);
    let result = import_step(path, thickness, &config).unwrap_or_else(|e| {
        eprintln!("Import failed: {e}");
        std::process::exit(1);
    });

    eprintln!(
        "Selected: {:.3} mm, {} pieces",
        result.selected_thickness_mm,
        result.pieces.len()
    );

    for (i, piece) in result.pieces.iter().enumerate() {
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        for pt in &piece.polygon.outer {
            min_x = min_x.min(pt.x);
            min_y = min_y.min(pt.y);
            max_x = max_x.max(pt.x);
            max_y = max_y.max(pt.y);
        }
        eprintln!(
            "  Piece {} (id={}): {} pts, {} holes, bbox ({:.1},{:.1})-({:.1},{:.1}) = {:.1}x{:.1}",
            i,
            piece.id.0,
            piece.polygon.outer.len(),
            piece.polygon.holes.len(),
            min_x,
            min_y,
            max_x,
            max_y,
            max_x - min_x,
            max_y - min_y,
        );
    }

    let sheet = SheetConfig::default();
    eprintln!(
        "\nSheet: {:.0} x {:.0}, sep={:.1}",
        sheet.width, sheet.height, sheet.separation
    );

    // Run with a short time limit to see LBF + GLS results
    let mut opt_config = OptimizerConfig::default();
    opt_config.time_limit_secs = 5.0;

    let (tx, rx) = mpsc::channel();
    let cancel = Arc::new(AtomicBool::new(false));

    let pieces = result.pieces.clone();
    let sheet2 = sheet.clone();
    std::thread::spawn(move || {
        run_nesting(pieces, sheet2, None, opt_config, tx, cancel);
    });

    eprintln!("\nRunning nesting...");
    loop {
        match rx.recv() {
            Ok(NestingUpdate::Improved(r)) => {
                let placed = r.sheets.first().map(|s| s.placements.len()).unwrap_or(0);
                let strip = r.sheets.first().map(|s| s.strip_length).unwrap_or(0.0);
                let util = r.sheets.first().map(|s| s.utilization).unwrap_or(0.0);
                eprintln!(
                    "  Improved: placed {}/{}, strip={:.1}, util={:.1}%, unplaced={}",
                    placed,
                    result.pieces.len(),
                    strip,
                    util * 100.0,
                    r.unplaced.len()
                );
                for p in &r.sheets.first().unwrap().placements {
                    eprintln!(
                        "    piece_id={}, x={:.1}, y={:.1}, rot={:.3}",
                        p.piece_id.0, p.x, p.y, p.rotation
                    );
                }
            }
            Ok(NestingUpdate::Progress {
                iteration,
                total_overlap,
                best_overlap,
                feasible_count,
            }) => {
                if iteration % 100 == 0 || iteration <= 5 {
                    eprintln!(
                        "  Progress: iter={}, overlap={:.1}, best_overlap={:.1}, feasible={}",
                        iteration, total_overlap, best_overlap, feasible_count
                    );
                }
            }
            Ok(NestingUpdate::Finished(r)) => {
                let placed = r.sheets.first().map(|s| s.placements.len()).unwrap_or(0);
                let strip = r.sheets.first().map(|s| s.strip_length).unwrap_or(0.0);
                eprintln!(
                    "  Finished: placed {}/{}, strip={:.1}, unplaced={}",
                    placed,
                    result.pieces.len(),
                    strip,
                    r.unplaced.len()
                );
                break;
            }
            Err(_) => {
                eprintln!("  Channel disconnected (optimizer crashed?)");
                break;
            }
        }
    }
}
