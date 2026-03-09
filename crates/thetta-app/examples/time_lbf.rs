/// Time LBF placement for a STEP file
use std::path::Path;
use std::time::Instant;
use thetta_geometry::types::SheetConfig;
use thetta_nesting::engine::NestingEngine;
use thetta_nesting::lbf::{LbfConfig, run_lbf};
use thetta_step::{ExtractionConfig, import_step};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = Path::new(&args[1]);
    let thickness: Option<f64> = args.get(2).and_then(|s| s.parse().ok());

    let result = import_step(path, thickness, &ExtractionConfig::default()).unwrap();
    eprintln!(
        "{} pieces at {:.2}mm",
        result.pieces.len(),
        result.selected_thickness_mm
    );

    let sheet = SheetConfig::default();
    let t0 = Instant::now();
    let mut engine = NestingEngine::new(&result.pieces, &sheet);
    eprintln!("Engine init: {:.1}ms", t0.elapsed().as_millis());

    let t1 = Instant::now();
    let res = run_lbf(&mut engine, LbfConfig::default(), None, |_| {});
    let elapsed = t1.elapsed();
    eprintln!(
        "LBF: placed {}/{} in {:.1}s, strip={:.1}",
        res.sheets[0].placements.len(),
        result.pieces.len(),
        elapsed.as_secs_f64(),
        res.sheets[0].strip_length,
    );
}
