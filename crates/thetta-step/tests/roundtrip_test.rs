use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use thetta_geometry::polygon_ops;
use thetta_geometry::types::{NestingResult, Placement, Sheet, SheetConfig};
use thetta_step::{ExtractionConfig, export_all_sheets, import_step};

fn temp_dir(name: &str) -> PathBuf {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("thetta_step_{name}_{ts}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn roundtrip_import_export_reimport_preserves_count_and_area() {
    let dir = temp_dir("roundtrip");
    let source = dir.join("source.json");
    fs::write(
        &source,
        r#"{
  "solids": [
    { "thickness_mm": 2.0, "profile": { "outer": [[0,0],[10,0],[10,5],[0,5]], "holes": [] } },
    { "thickness_mm": 2.0, "profile": { "outer": [[0,0],[4,0],[4,4],[0,4]], "holes": [] } }
  ]
}"#,
    )
    .unwrap();

    let cfg = ExtractionConfig::default();
    let imported = import_step(&source, None, &cfg).unwrap();
    let before_area: f64 = imported
        .pieces
        .iter()
        .map(|p| polygon_ops::area(&p.polygon))
        .sum();

    let placements = imported
        .pieces
        .iter()
        .enumerate()
        .map(|(idx, p)| Placement {
            piece_id: p.id,
            x: idx as f64 * 20.0,
            y: idx as f64 * 10.0,
            rotation: if idx % 2 == 0 {
                0.0
            } else {
                std::f64::consts::FRAC_PI_2
            },
        })
        .collect::<Vec<_>>();

    let result = NestingResult {
        sheets: vec![Sheet {
            placements,
            utilization: 0.5,
            strip_length: 100.0,
        }],
        unplaced: Vec::new(),
    };

    let sheet_config = SheetConfig {
        width: 200.0,
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

    let reimported = import_step(&out[0], None, &cfg).unwrap();
    assert_eq!(reimported.pieces.len(), imported.pieces.len());

    let after_area: f64 = reimported
        .pieces
        .iter()
        .map(|p| polygon_ops::area(&p.polygon))
        .sum();
    let rel = ((before_area - after_area).abs() / before_area.max(1e-9)) * 100.0;
    assert!(rel <= 5.0, "relative area drift too high: {rel:.2}%");
}
