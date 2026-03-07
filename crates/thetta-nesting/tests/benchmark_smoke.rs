use std::fs;
use std::path::Path;

use thetta_nesting::benchmark::load_benchmark;
use thetta_nesting::engine::NestingEngine;
use thetta_nesting::lbf::{LbfConfig, run_lbf};

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

        let mut engine = NestingEngine::new_with_orientations(&pieces, &sheet, Some(&meta.orientations));
        let result = run_lbf(
            &mut engine,
            LbfConfig {
                grid_n: 30,
                rotations_n: 8,
                orientations: Some(meta.orientations),
            },
        );

        assert_eq!(result.unplaced.len(), 0, "unplaced items in {name}");
        let strip_length = result.sheets[0].strip_length;
        assert!(strip_length.is_finite() && strip_length > 0.0);

        let utilization = result.sheets[0].utilization;
        assert!(utilization.is_finite());
        assert!((0.0..=1.0).contains(&utilization));
    }
}

fn benchmark_input_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../research/repos/sparrow/data/input")
}
