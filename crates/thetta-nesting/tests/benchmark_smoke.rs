use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;

use thetta_nesting::benchmark::load_benchmark;
use thetta_nesting::engine::NestingEngine;
use thetta_nesting::gls::OptimizerConfig;
use thetta_nesting::lbf::{LbfConfig, run_lbf};
use thetta_nesting::optimizer::run_optimizer;
use thetta_nesting::runner::{NestingMode, NestingUpdate};

#[test]
fn load_all_34_benchmark_instances() {
    let input_dir = benchmark_input_dir();
    let files = fs::read_dir(input_dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|x| x.path()))
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
        .collect::<Vec<_>>();

    assert_eq!(files.len(), 34);
    for path in files {
        let loaded = load_benchmark(&path);
        assert!(loaded.is_ok(), "failed to parse {}", path.display());
    }
}

#[test]
fn lbf_places_all_on_small_instances() {
    for name in ["jakobs1", "jakobs2", "shapes0"] {
        let path = benchmark_input_dir().join(format!("{name}.json"));
        let (pieces, sheet, meta) = load_benchmark(&path).unwrap();

        let mut engine =
            NestingEngine::new_with_orientations(&pieces, &sheet, Some(&meta.orientations));
        let result = run_lbf(
            &mut engine,
            LbfConfig {
                grid_n: 30,
                rotations_n: 8,
                orientations: Some(meta.orientations),
            },
            None,
            |_| {},
        );

        assert_eq!(result.unplaced.len(), 0, "unplaced items in {name}");
        let strip_length = result.sheets[0].strip_length;
        assert!(strip_length.is_finite() && strip_length > 0.0);

        let utilization = result.sheets[0].utilization;
        assert!(utilization.is_finite());
        assert!((0.0..=1.0).contains(&utilization));
    }
}

#[test]
fn strip_mode_jakobs1_stays_single_sheet_with_stable_length() {
    let path = benchmark_input_dir().join("jakobs1.json");
    let (pieces, sheet, meta) = load_benchmark(&path).unwrap();
    let mut baseline_engine =
        NestingEngine::new_with_orientations(&pieces, &sheet, Some(&meta.orientations));
    let baseline = run_lbf(
        &mut baseline_engine,
        LbfConfig {
            grid_n: 30,
            rotations_n: 8,
            orientations: Some(meta.orientations.clone()),
        },
        None,
        |_| {},
    );
    let baseline_strip = baseline.sheets[0].strip_length;

    let (tx, rx) = mpsc::channel();
    let cancel = Arc::new(AtomicBool::new(false));

    std::thread::spawn(move || {
        run_optimizer(
            pieces,
            sheet,
            Some(meta.orientations),
            OptimizerConfig {
                time_limit_secs: 0.1,
                ..OptimizerConfig::default()
            },
            NestingMode::Strip,
            tx,
            cancel,
        );
    });

    let mut final_result = None;
    while let Ok(update) = rx.recv() {
        if let NestingUpdate::Finished(result) = update {
            final_result = Some(result);
            break;
        }
    }
    let result = final_result.expect("strip optimizer finished");
    assert_eq!(result.sheets.len(), 1);
    assert!(result.unplaced.is_empty());
    let strip = result.sheets[0].strip_length;
    assert!(strip.is_finite() && strip > 0.0);
    assert!(strip <= baseline_strip * 10.0 + 1.0);
}

fn benchmark_input_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../research/repos/sparrow/data/input")
}
