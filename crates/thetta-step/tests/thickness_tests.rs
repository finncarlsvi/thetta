use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use thetta_step::{ExtractionConfig, import_step};

fn temp_step_path(name: &str) -> PathBuf {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("thetta_step_tests_{ts}"));
    fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

#[test]
fn detects_single_thickness_group() {
    let path = temp_step_path("single.json");
    fs::write(
        &path,
        r#"{
  "solids": [
    { "thickness_mm": 2.0, "profile": { "outer": [[0,0],[10,0],[10,5],[0,5]], "holes": [] } }
  ]
}"#,
    )
    .unwrap();

    let result = import_step(&path, None, &ExtractionConfig::default()).unwrap();
    assert_eq!(result.thicknesses.len(), 1);
    assert!((result.thicknesses[0].value_mm - 2.0).abs() < 1e-6);
    assert_eq!(result.thicknesses[0].count, 1);
}

#[test]
fn defaults_to_most_common_thickness_group() {
    let path = temp_step_path("multi.json");
    fs::write(
        &path,
        r#"{
  "solids": [
    { "thickness_mm": 2.0, "profile": { "outer": [[0,0],[5,0],[5,5],[0,5]], "holes": [] } },
    { "thickness_mm": 2.04, "profile": { "outer": [[0,0],[4,0],[4,4],[0,4]], "holes": [] } },
    { "thickness_mm": 3.0, "profile": { "outer": [[0,0],[6,0],[6,6],[0,6]], "holes": [] } }
  ]
}"#,
    )
    .unwrap();

    let cfg = ExtractionConfig::default();
    let result = import_step(&path, None, &cfg).unwrap();
    assert_eq!(result.thicknesses.len(), 2);
    assert!((result.selected_thickness_mm - 2.0).abs() < 1e-6);

    let selected = import_step(&path, Some(3.0), &cfg).unwrap();
    assert!((selected.selected_thickness_mm - 3.0).abs() < 1e-6);
    assert_eq!(selected.pieces.len(), 1);
}
