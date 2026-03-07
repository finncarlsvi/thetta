use std::path::Path;

use thetta_geometry::test_shapes;
use thetta_geometry::types::{Piece, PieceId, SheetConfig};
use thetta_nesting::benchmark::load_benchmark;
use thetta_nesting::engine::NestingEngine;
use thetta_nesting::lbf::{LbfConfig, run_lbf};

fn rect_piece(id: u64, w: f64, h: f64) -> Piece {
    Piece {
        id: PieceId(id),
        polygon: test_shapes::rectangle(w, h),
        source_file: "test.json".into(),
        quantity: 1,
    }
}

#[test]
fn place_single_rectangle_no_collision() {
    let pieces = vec![rect_piece(1, 50.0, 20.0)];
    let sheet = SheetConfig {
        width: 500.0,
        height: 500.0,
        separation: 0.0,
    };

    let mut engine = NestingEngine::new(&pieces, &sheet);
    assert!(engine.place_item(0, 100.0, 100.0, 0.0));
    assert_eq!(engine.collision_cost(0, 100.0, 100.0, 0.0), 0.0);
}

#[test]
fn overlapping_rectangles_detected() {
    let pieces = vec![rect_piece(1, 100.0, 100.0), rect_piece(2, 100.0, 100.0)];
    let sheet = SheetConfig {
        width: 500.0,
        height: 500.0,
        separation: 0.0,
    };

    let mut engine = NestingEngine::new(&pieces, &sheet);
    assert!(engine.place_item(0, 200.0, 200.0, 0.0));
    assert!(!engine.place_item(1, 200.0, 200.0, 0.0));
}

#[test]
fn remove_item_clears_collision() {
    let pieces = vec![rect_piece(1, 100.0, 100.0), rect_piece(2, 100.0, 100.0)];
    let sheet = SheetConfig {
        width: 500.0,
        height: 500.0,
        separation: 0.0,
    };

    let mut engine = NestingEngine::new(&pieces, &sheet);
    assert!(engine.place_item(0, 200.0, 200.0, 0.0));
    assert!(engine.collision_cost(1, 200.0, 200.0, 0.0) > 0.0);
    engine.remove_item(0);
    assert_eq!(engine.collision_cost(1, 200.0, 200.0, 0.0), 0.0);
}

#[test]
fn force_place_replaces_existing_without_underflow() {
    let pieces = vec![rect_piece(1, 50.0, 50.0)];
    let sheet = SheetConfig {
        width: 500.0,
        height: 500.0,
        separation: 0.0,
    };

    let mut engine = NestingEngine::new(&pieces, &sheet);
    engine.force_place_item(0, 100.0, 100.0, 0.0);
    engine.force_place_item(0, 300.0, 100.0, 0.0);

    let p = engine.placement_of(0).expect("placed");
    assert!((p.x - 300.0).abs() < 1e-6);
    assert!((p.y - 100.0).abs() < 1e-6);
}

#[test]
fn load_jakobs1_parses_25_items() {
    let (pieces, _sheet, _meta) = load_benchmark(&benchmark_path("jakobs1")).unwrap();
    assert_eq!(pieces.len(), 25);
}

#[test]
fn lbf_on_jakobs1_places_all_without_collision() {
    let (pieces, sheet, meta) = load_benchmark(&benchmark_path("jakobs1")).unwrap();

    let mut engine = NestingEngine::new_with_orientations(&pieces, &sheet, Some(&meta.orientations));
    let result = run_lbf(
        &mut engine,
        LbfConfig {
            grid_n: 35,
            rotations_n: 4,
            orientations: Some(meta.orientations),
        },
    );

    assert_eq!(result.sheets[0].placements.len(), 25);
    assert!(result.unplaced.is_empty());

    for idx in 0..engine.piece_count() {
        let p = engine.placement_of(idx).expect("placed");
        assert_eq!(engine.collision_cost(idx, p.x, p.y, p.rotation), 0.0);
    }
}

fn benchmark_path(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../research/repos/sparrow/data/input/{name}.json"))
}
