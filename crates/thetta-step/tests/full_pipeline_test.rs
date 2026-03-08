//! End-to-end pipeline test: load real STEP → create placements → export → verify output.

#![cfg(feature = "occt")]

use std::sync::Mutex;

use thetta_geometry::types::{NestingResult, Placement, Sheet};
use thetta_step::{ExtractionConfig, export_all_sheets, import_step};

static OCCT_LOCK: Mutex<()> = Mutex::new(());

fn test_data_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("test_data/step")
}

#[test]
fn full_pipeline_single_box() {
    let _lock = OCCT_LOCK.lock().unwrap();
    let path = test_data_dir().join("single_box_20x10x2.step");
    assert!(
        path.exists(),
        "Run gen_test_step first to generate test data"
    );

    // 1. Import
    let cfg = ExtractionConfig::default();
    let imported = import_step(&path, None, &cfg).unwrap();
    assert_eq!(imported.pieces.len(), 1);
    assert!(
        (imported.thicknesses[0].value_mm - 2.0).abs() < 0.5,
        "thickness = {}",
        imported.thicknesses[0].value_mm
    );

    // 2. Create manual placements
    let placements = imported
        .pieces
        .iter()
        .enumerate()
        .map(|(i, p)| Placement {
            piece_id: p.id,
            x: i as f64 * 30.0,
            y: 0.0,
            rotation: 0.0,
        })
        .collect();

    let result = NestingResult {
        sheets: vec![Sheet {
            placements,
            utilization: 0.5,
            strip_length: 100.0,
        }],
        unplaced: Vec::new(),
    };

    // 3. Export
    let out_dir = std::env::temp_dir().join("thetta_pipeline_test");
    std::fs::create_dir_all(&out_dir).unwrap();
    let exported = export_all_sheets(
        &imported.model,
        &imported.piece_refs,
        &result,
        &out_dir,
        "pipeline",
    )
    .unwrap();
    assert_eq!(exported.len(), 1);
    assert!(exported[0].exists(), "output STEP file should exist");
    assert!(
        std::fs::metadata(&exported[0]).unwrap().len() > 100,
        "output STEP file should not be empty"
    );

    // 4. Reimport and verify
    let reimported = import_step(&exported[0], None, &cfg).unwrap();
    assert_eq!(
        reimported.pieces.len(),
        imported.pieces.len(),
        "reimport should have same piece count"
    );
}

#[test]
fn full_pipeline_multi_thickness() {
    let _lock = OCCT_LOCK.lock().unwrap();
    let path = test_data_dir().join("multi_thickness.step");
    assert!(
        path.exists(),
        "Run gen_test_step first to generate test data"
    );

    let cfg = ExtractionConfig::default();

    // Import with auto-selection (should pick 2mm, 3 solids)
    let imported = import_step(&path, None, &cfg).unwrap();
    assert!(
        imported.thicknesses.len() >= 2,
        "should have at least 2 thickness groups"
    );
    assert!(
        (imported.selected_thickness_mm - 2.0).abs() < 0.5,
        "should auto-select 2mm (most common), got {}",
        imported.selected_thickness_mm
    );
    assert_eq!(imported.pieces.len(), 3, "3 solids at 2mm");

    // Import with explicit 5mm
    let imported5 = import_step(&path, Some(5.0), &cfg).unwrap();
    assert_eq!(imported5.pieces.len(), 1, "1 solid at 5mm");
}
