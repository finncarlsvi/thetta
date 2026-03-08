use jagua_rs::geometry::geo_enums::RotationRange;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::time::Duration;
use thetta_geometry::test_shapes;
use thetta_geometry::types::{Piece, PieceId, SheetConfig};
use thetta_nesting::gls::OptimizerConfig;
use thetta_nesting::lbf::{LbfConfig, run_lbf_multisheet};
use thetta_nesting::multi_sheet::MultiSheetEngine;
use thetta_nesting::optimizer::run_optimizer;
use thetta_nesting::runner::{NestingMode, NestingUpdate};

fn rect_piece(id: u64, w: f64, h: f64) -> Piece {
    Piece {
        id: PieceId(id),
        polygon: test_shapes::rectangle(w, h),
        source_file: "test.json".into(),
        quantity: 1,
    }
}

#[test]
fn multisheet_overflow_opens_second_sheet_for_large_parts() {
    let pieces = vec![rect_piece(1, 90.0, 90.0), rect_piece(2, 90.0, 90.0)];
    let sheet = SheetConfig {
        width: 100.0,
        height: 100.0,
        separation: 0.0,
    };
    let orientations = vec![RotationRange::None; pieces.len()];
    let mut engine = MultiSheetEngine::new(&pieces, &sheet, &orientations);

    let result = run_lbf_multisheet(
        &mut engine,
        LbfConfig {
            grid_n: 20,
            rotations_n: 1,
            orientations: Some(orientations),
        },
        None,
        |_| {},
    );

    assert_eq!(result.unplaced.len(), 0);
    assert_eq!(result.sheets.len(), 2);
    assert_eq!(result.sheets[0].placements.len(), 1);
    assert_eq!(result.sheets[1].placements.len(), 1);
}

#[test]
fn multisheet_keeps_one_sheet_when_all_fit() {
    let pieces = (0..4)
        .map(|i| rect_piece(i as u64, 20.0, 20.0))
        .collect::<Vec<_>>();
    let sheet = SheetConfig {
        width: 100.0,
        height: 100.0,
        separation: 0.0,
    };
    let orientations = vec![RotationRange::None; pieces.len()];
    let mut engine = MultiSheetEngine::new(&pieces, &sheet, &orientations);

    let result = run_lbf_multisheet(
        &mut engine,
        LbfConfig {
            grid_n: 20,
            rotations_n: 1,
            orientations: Some(orientations),
        },
        None,
        |_| {},
    );

    assert_eq!(result.unplaced.len(), 0);
    assert_eq!(result.sheets.len(), 1);
    assert_eq!(result.sheets[0].placements.len(), 4);
}

#[test]
fn place_and_remove_updates_assignment() {
    let pieces = vec![rect_piece(1, 20.0, 20.0)];
    let sheet = SheetConfig {
        width: 100.0,
        height: 100.0,
        separation: 0.0,
    };
    let orientations = vec![RotationRange::None; pieces.len()];
    let mut engine = MultiSheetEngine::new(&pieces, &sheet, &orientations);
    let sheet_idx = engine.open_sheet();

    assert!(engine.place_on_sheet(0, sheet_idx, 0.0, 0.0, 0.0));
    let placed = engine.current_result();
    assert_eq!(placed.sheets.len(), 1);
    assert_eq!(placed.sheets[0].placements.len(), 1);
    assert!(placed.unplaced.is_empty());

    engine.remove_piece(0);
    let removed = engine.current_result();
    assert!(removed.sheets.is_empty());
    assert_eq!(removed.unplaced, vec![PieceId(1)]);
}

#[test]
fn oversized_piece_stays_unplaced() {
    let pieces = vec![rect_piece(1, 200.0, 40.0)];
    let sheet = SheetConfig {
        width: 100.0,
        height: 100.0,
        separation: 0.0,
    };
    let orientations = vec![RotationRange::None; pieces.len()];
    let mut engine = MultiSheetEngine::new(&pieces, &sheet, &orientations);

    let result = run_lbf_multisheet(
        &mut engine,
        LbfConfig {
            grid_n: 20,
            rotations_n: 1,
            orientations: Some(orientations),
        },
        None,
        |_| {},
    );

    assert!(result.sheets.is_empty());
    assert_eq!(result.unplaced, vec![PieceId(1)]);
}

#[test]
fn multisheet_lbf_places_24_parts_on_three_or_more_sheets_without_overlap() {
    let pieces = (0..24)
        .map(|i| rect_piece(i as u64, 40.0, 40.0))
        .collect::<Vec<_>>();
    let sheet = SheetConfig {
        width: 120.0,
        height: 120.0,
        separation: 0.0,
    };
    let orientations = vec![RotationRange::None; pieces.len()];
    let mut engine = MultiSheetEngine::new(&pieces, &sheet, &orientations);

    let result = run_lbf_multisheet(
        &mut engine,
        LbfConfig {
            grid_n: 45,
            rotations_n: 1,
            orientations: Some(orientations),
        },
        None,
        |_| {},
    );

    assert!(result.unplaced.is_empty());
    assert!(result.sheets.len() >= 3);
    for sheet_result in &result.sheets {
        assert!(sheet_result.strip_length <= sheet.width + 1e-6);
        assert_no_rect_overlaps(sheet_result, &pieces);
    }
}

#[test]
fn optimizer_mode_dispatch_strip_vs_sheet() {
    let pieces = (0..10)
        .map(|i| rect_piece(i as u64, 40.0, 40.0))
        .collect::<Vec<_>>();
    let sheet = SheetConfig {
        width: 120.0,
        height: 120.0,
        separation: 0.0,
    };
    let config = OptimizerConfig {
        time_limit_secs: 0.2,
        ..OptimizerConfig::default()
    };

    let strip_result = run_optimizer_and_wait(
        pieces.clone(),
        sheet.clone(),
        vec![RotationRange::None; 10],
        config.clone(),
        NestingMode::Strip,
    );
    assert_eq!(strip_result.sheets.len(), 1);

    let sheet_result = run_optimizer_and_wait(
        pieces,
        sheet,
        vec![RotationRange::None; 10],
        config,
        NestingMode::Sheet,
    );
    assert!(sheet_result.sheets.len() > 1);
}

fn run_optimizer_and_wait(
    pieces: Vec<Piece>,
    sheet: SheetConfig,
    orientations: Vec<RotationRange>,
    config: OptimizerConfig,
    mode: NestingMode,
) -> thetta_geometry::types::NestingResult {
    let (tx, rx) = mpsc::channel();
    let cancel = Arc::new(AtomicBool::new(false));

    std::thread::spawn(move || {
        run_optimizer(
            pieces,
            sheet,
            Some(orientations),
            config,
            mode,
            tx,
            cancel.clone(),
        );
    });

    loop {
        match rx.recv_timeout(Duration::from_secs(10)) {
            Ok(NestingUpdate::Finished(result)) => return result,
            Ok(_) => {}
            Err(err) => panic!("optimizer did not finish: {err}"),
        }
    }
}

fn assert_no_rect_overlaps(sheet: &thetta_geometry::types::Sheet, pieces: &[Piece]) {
    for i in 0..sheet.placements.len() {
        for j in (i + 1)..sheet.placements.len() {
            let a = &sheet.placements[i];
            let b = &sheet.placements[j];
            let ap = pieces
                .iter()
                .find(|p| p.id == a.piece_id)
                .expect("placement piece exists");
            let bp = pieces
                .iter()
                .find(|p| p.id == b.piece_id)
                .expect("placement piece exists");

            let aw = ap.polygon.outer[1].x - ap.polygon.outer[0].x;
            let ah = ap.polygon.outer[2].y - ap.polygon.outer[1].y;
            let bw = bp.polygon.outer[1].x - bp.polygon.outer[0].x;
            let bh = bp.polygon.outer[2].y - bp.polygon.outer[1].y;

            let overlap_x = a.x < b.x + bw && b.x < a.x + aw;
            let overlap_y = a.y < b.y + bh && b.y < a.y + ah;
            assert!(
                !(overlap_x && overlap_y),
                "overlap between piece {} and {}",
                a.piece_id.0,
                b.piece_id.0
            );
        }
    }
}
