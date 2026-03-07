use thetta_geometry::test_shapes;
use thetta_geometry::types::{Piece, PieceId, SheetConfig};
use thetta_nesting::engine::NestingEngine;
use thetta_nesting::lbf::{run_lbf, LbfConfig};

fn rect_piece(id: u64, w: f64, h: f64) -> Piece {
    Piece {
        id: PieceId(id),
        polygon: test_shapes::rectangle(w, h),
        source_file: "test.json".into(),
        quantity: 1,
    }
}

#[test]
fn place_single_rectangle_valid() {
    let pieces = vec![rect_piece(1, 50.0, 20.0)];
    let sheet = SheetConfig {
        width: 500.0,
        height: 500.0,
        separation: 0.0,
    };
    let mut engine = NestingEngine::new(&pieces, &sheet);
    assert!(engine.place_item(0, 10.0, 10.0, 0.0));
    let result = engine.current_result();
    assert_eq!(result.sheets[0].placements.len(), 1);
    assert!(result.unplaced.is_empty());
}

#[test]
fn overlapping_pieces_detect_collision() {
    let pieces = vec![rect_piece(1, 100.0, 100.0), rect_piece(2, 100.0, 100.0)];
    let sheet = SheetConfig {
        width: 500.0,
        height: 500.0,
        separation: 0.0,
    };
    let mut engine = NestingEngine::new(&pieces, &sheet);

    assert!(engine.place_item(0, 0.0, 0.0, 0.0));
    assert!(!engine.place_item(1, 10.0, 10.0, 0.0));
}

#[test]
fn lbf_on_10_rectangles_places_all_without_overlap() {
    let pieces = (0..10)
        .map(|i| rect_piece(i as u64, 40.0, 20.0))
        .collect::<Vec<_>>();
    let sheet = SheetConfig {
        width: 1000.0,
        height: 1000.0,
        separation: 0.0,
    };

    let mut engine = NestingEngine::new(&pieces, &sheet);
    let result = run_lbf(
        &mut engine,
        LbfConfig {
            grid_n: 35,
            rotations_n: 4,
            ..LbfConfig::default()
        },
    );

    assert_eq!(result.sheets[0].placements.len(), 10);
    assert!(result.unplaced.is_empty());
}

#[test]
fn lbf_with_too_many_pieces_expands_strip() {
    let pieces = (0..10)
        .map(|i| rect_piece(i as u64, 20.0, 20.0))
        .collect::<Vec<_>>();
    let sheet = SheetConfig {
        width: 30.0,
        height: 30.0,
        separation: 0.0,
    };

    let mut engine = NestingEngine::new(&pieces, &sheet);
    let result = run_lbf(
        &mut engine,
        LbfConfig {
            grid_n: 20,
            rotations_n: 1,
            ..LbfConfig::default()
        },
    );

    assert_eq!(result.sheets[0].placements.len(), 10);
    assert!(result.unplaced.is_empty());
    assert!(result.sheets[0].strip_length > sheet.width);
}
