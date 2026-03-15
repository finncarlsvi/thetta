//! Integration tests that require the `occt` feature.
//! These create real 3D solids, write them to STEP files, and test the full pipeline.
//!
//! OCCT has global state that is not thread-safe, so all tests acquire a mutex.

#![cfg(feature = "occt")]

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use thetta_geometry::polygon_ops;
use thetta_geometry::types::{NestingResult, Placement, Sheet, SheetConfig};
use thetta_step::{ExtractionConfig, export_all_sheets, import_step};

static OCCT_LOCK: Mutex<()> = Mutex::new(());

fn temp_dir(name: &str) -> PathBuf {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("thetta_occt_{name}_{ts}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// Helper: create a STEP file containing one or more boxes using OCCT primitives.
fn create_step_with_boxes(path: &PathBuf, boxes: &[(f64, f64, f64, f64, f64, f64)]) {
    use opencascade_sys::ffi;

    let mut writer = ffi::STEPControl_Writer_ctor();

    for &(x, y, z, dx, dy, dz) in boxes {
        let origin = ffi::new_point(x, y, z);
        let mut box_maker = ffi::BRepPrimAPI_MakeBox_ctor(&origin, dx, dy, dz);
        let box_shape = box_maker.pin_mut().Shape();
        let status = ffi::transfer_shape(writer.pin_mut(), box_shape);
        assert_eq!(status, ffi::IFSelect_ReturnStatus::IFSelect_RetDone);
    }

    let status = ffi::write_step(writer.pin_mut(), path.to_str().unwrap().to_string());
    assert_eq!(status, ffi::IFSelect_ReturnStatus::IFSelect_RetDone);
}

#[test]
fn import_single_box_step_file() {
    let _lock = OCCT_LOCK.lock().unwrap();
    let dir = temp_dir("single_box");
    let path = dir.join("box.step");

    // Create a 20x10x2 box (sheet metal: 2mm thickness)
    create_step_with_boxes(&path, &[(0.0, 0.0, 0.0, 20.0, 10.0, 2.0)]);

    let cfg = ExtractionConfig::default();
    let result = import_step(&path, None, &cfg).unwrap();

    assert_eq!(result.pieces.len(), 1, "should extract 1 piece from 1 box");
    assert_eq!(
        result.thicknesses.len(),
        1,
        "should detect 1 thickness group"
    );
    assert!(
        (result.thicknesses[0].value_mm - 2.0).abs() < 0.5,
        "thickness should be ~2mm, got {}",
        result.thicknesses[0].value_mm
    );

    let poly = &result.pieces[0].polygon;
    let area = polygon_ops::area(poly);
    assert!(
        poly.outer.len() >= 4,
        "rectangular profile needs >= 4 vertices"
    );
    // The profile should be approximately 20x10 = 200 mm²
    // (tolerance accounts for discretization and face selection variation)
    assert!(
        area > 50.0 && area < 250.0,
        "expected area in reasonable range for 20x10 box, got {area}"
    );
}

#[test]
fn import_multiple_boxes_different_thickness() {
    let _lock = OCCT_LOCK.lock().unwrap();
    let dir = temp_dir("multi_box");
    let path = dir.join("boxes.step");

    // Two boxes at 2mm, one at 5mm
    create_step_with_boxes(
        &path,
        &[
            (0.0, 0.0, 0.0, 20.0, 10.0, 2.0),
            (30.0, 0.0, 0.0, 15.0, 8.0, 2.0),
            (60.0, 0.0, 0.0, 12.0, 12.0, 5.0),
        ],
    );

    let cfg = ExtractionConfig::default();
    let result = import_step(&path, None, &cfg).unwrap();

    assert!(
        result.thicknesses.len() >= 2,
        "should detect at least 2 thickness groups, got {}",
        result.thicknesses.len()
    );

    // Auto-selection should pick the most common thickness (2mm, 2 solids)
    assert!(
        (result.selected_thickness_mm - 2.0).abs() < 0.5,
        "should auto-select 2mm thickness, got {}",
        result.selected_thickness_mm
    );
    assert_eq!(result.pieces.len(), 2, "should extract 2 pieces at 2mm");

    // Explicit 5mm selection
    let result5 = import_step(&path, Some(5.0), &cfg).unwrap();
    assert_eq!(result5.pieces.len(), 1, "should extract 1 piece at 5mm");
}

#[test]
fn occt_roundtrip_export_reimport() {
    let _lock = OCCT_LOCK.lock().unwrap();
    let dir = temp_dir("occt_roundtrip");
    let path = dir.join("source.step");

    create_step_with_boxes(
        &path,
        &[
            (0.0, 0.0, 0.0, 20.0, 10.0, 2.0),
            (30.0, 0.0, 0.0, 15.0, 8.0, 2.0),
        ],
    );

    let cfg = ExtractionConfig::default();
    let imported = import_step(&path, None, &cfg).unwrap();

    let before_area: f64 = imported
        .pieces
        .iter()
        .map(|p| polygon_ops::area(&p.polygon))
        .sum();

    // Create fake placements (translate each piece)
    let placements = imported
        .pieces
        .iter()
        .enumerate()
        .map(|(idx, p)| Placement {
            piece_id: p.id,
            x: idx as f64 * 50.0,
            y: 0.0,
            rotation: 0.0,
        })
        .collect();

    let result = NestingResult {
        sheets: vec![Sheet {
            placements,
            utilization: 0.5,
            strip_length: 200.0,
        }],
        unplaced: Vec::new(),
    };

    let sheet_config = SheetConfig {
        width: 300.0,
        height: 200.0,
        separation: 0.0,
    };
    let out = export_all_sheets(
        &imported.model,
        &imported.piece_refs,
        &result,
        &sheet_config,
        &dir,
        "nested",
        None,
    )
    .unwrap();
    assert_eq!(out.len(), 1);

    // Reimport the exported file
    let reimported = import_step(&out[0], None, &cfg).unwrap();
    assert_eq!(
        reimported.pieces.len(),
        imported.pieces.len(),
        "piece count should be preserved"
    );

    let after_area: f64 = reimported
        .pieces
        .iter()
        .map(|p| polygon_ops::area(&p.polygon))
        .sum();
    let rel = ((before_area - after_area).abs() / before_area.max(1e-9)) * 100.0;
    assert!(
        rel <= 10.0,
        "relative area drift too high: {rel:.2}% (before={before_area}, after={after_area})"
    );
}
