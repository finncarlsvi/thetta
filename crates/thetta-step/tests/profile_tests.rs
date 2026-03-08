use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use thetta_geometry::polygon_ops;
use thetta_step::{ExtractionConfig, import_step};

fn temp_step_path(name: &str) -> PathBuf {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("thetta_step_profiles_{ts}"));
    fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

#[test]
fn extracts_rectangular_profile_area() {
    let path = temp_step_path("rect.json");
    fs::write(
        &path,
        r#"{
  "solids": [
    { "thickness_mm": 1.5, "profile": { "outer": [[0,0],[20,0],[20,10],[0,10]], "holes": [] } }
  ]
}"#,
    )
    .unwrap();

    let result = import_step(&path, None, &ExtractionConfig::default()).unwrap();
    assert_eq!(result.pieces.len(), 1);
    let poly = &result.pieces[0].polygon;
    assert!(poly.outer.len() >= 4);
    assert!((polygon_ops::area(poly) - 200.0).abs() < 1e-6);
}

#[test]
fn preserves_hole_and_winding_convention() {
    let path = temp_step_path("hole.json");
    fs::write(
        &path,
        r#"{
  "solids": [
    {
      "thickness_mm": 2.0,
      "profile": {
        "outer": [[0,0],[12,0],[12,12],[0,12]],
        "holes": [[[4,4],[4,8],[8,8],[8,4]]]
      }
    }
  ]
}"#,
    )
    .unwrap();

    let result = import_step(&path, None, &ExtractionConfig::default()).unwrap();
    let poly = &result.pieces[0].polygon;
    assert_eq!(poly.holes.len(), 1);
    assert!((polygon_ops::area(poly) - 128.0).abs() < 1e-6);
}

#[test]
fn extracts_l_shaped_profile() {
    let path = temp_step_path("lshape.json");
    fs::write(
        &path,
        r#"{
  "solids": [
    {
      "thickness_mm": 2.0,
      "profile": {
        "outer": [[0,0],[8,0],[8,2],[2,2],[2,8],[0,8]],
        "holes": []
      }
    }
  ]
}"#,
    )
    .unwrap();

    let result = import_step(&path, None, &ExtractionConfig::default()).unwrap();
    let poly = &result.pieces[0].polygon;
    assert!(poly.outer.len() >= 6);
    assert!((polygon_ops::area(poly) - 28.0).abs() < 1e-6);
}
