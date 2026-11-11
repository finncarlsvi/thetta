use jagua_rs::geometry::geo_enums::RotationRange;
use std::{
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
};
use thetta_geometry::{polygon_ops, test_shapes, types::*};
use thetta_nesting::improve::{consolidate, improve, validate};
fn piece(id: u64, polygon: Polygon) -> Piece {
    Piece {
        id: PieceId(id),
        polygon,
        quantity: 1,
        source_file: Default::default(),
    }
}
fn placement(p: &Piece, x: f64, y: f64, rotation: f64) -> Placement {
    let poly = polygon_ops::rotate(&p.polygon, rotation, Point2D { x: 0., y: 0. });
    let minx = poly.outer.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
    let miny = poly.outer.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
    Placement {
        piece_id: p.id,
        x: x - minx,
        y: y - miny,
        rotation,
    }
}
fn result(sheets: Vec<Vec<Placement>>) -> NestingResult {
    NestingResult {
        sheets: sheets
            .into_iter()
            .map(|placements| Sheet {
                placements,
                ..Default::default()
            })
            .collect(),
        unplaced: vec![],
    }
}
#[test]
fn automatic_consolidation_returns_time_when_sheet_count_is_provably_minimal() {
    let pieces = vec![piece(1, test_shapes::rectangle(90., 90.))];
    let rotations = vec![RotationRange::None];
    let stock = SheetConfig {
        width: 100.,
        height: 100.,
        separation: 5.,
    };
    let baseline = result(vec![vec![placement(&pieces[0], 10., 0., 0.)]]);
    let mut passes = 0;
    let r = consolidate(
        &pieces,
        &rotations,
        &stock,
        &baseline,
        Duration::from_secs(30),
        &AtomicBool::new(false),
        |_| {},
        |_| passes += 1,
    )
    .unwrap();
    validate(&r, &pieces, &rotations, &stock).unwrap();
    assert_eq!(r.sheets.len(), 1);
    assert_eq!(
        passes, 0,
        "Impossible sheet-count reductions must not consume other materials' time"
    );
    assert!(r.sheets[0].placements[0].x.abs() < 0.001);
}
#[test]
fn interlocks_actual_hooks_with_twelve_mm_clearance() {
    let poly: Polygon = serde_json::from_str(include_str!("fixtures/hooked-panel.json")).unwrap();
    let pieces = vec![piece(1, poly.clone()), piece(2, poly)];
    let stock = SheetConfig {
        width: 2440.,
        height: 1220.,
        separation: 12.,
    };
    let rotations = vec![RotationRange::Continuous; 2];
    let baseline = result(vec![vec![
        placement(&pieces[0], 0., 0., 0.),
        placement(&pieces[1], 881.2, 0., std::f64::consts::PI),
    ]]);
    let mut updates = 0;
    let r = improve(
        &pieces,
        &rotations,
        &stock,
        &baseline,
        Duration::from_secs(6),
        &AtomicBool::new(false),
        |_| updates += 1,
        |_| {},
    )
    .unwrap();
    validate(&r, &pieces, &rotations, &stock).unwrap();
    let points: Vec<_> = r.sheets[0]
        .placements
        .iter()
        .flat_map(|p| {
            let poly = polygon_ops::rotate(
                &pieces[(p.piece_id.0 - 1) as usize].polygon,
                p.rotation,
                Point2D { x: 0., y: 0. },
            );
            poly.outer.into_iter().map(move |v| (v.x + p.x, v.y + p.y))
        })
        .collect();
    let minx = points.iter().map(|v| v.0).fold(f64::INFINITY, f64::min);
    let maxx = points.iter().map(|v| v.0).fold(f64::NEG_INFINITY, f64::max);
    let miny = points.iter().map(|v| v.1).fold(f64::INFINITY, f64::min);
    let maxy = points.iter().map(|v| v.1).fold(f64::NEG_INFINITY, f64::max);
    let reduction = 1. - (maxx - minx) * (maxy - miny) / (1750.4 * 502.2);
    println!("hook footprint reduction: {:.2}%", reduction * 100.);
    assert!(updates > 0);
    assert!(reduction >= 0.30);
}
#[test]
fn empties_sparse_sheet_and_preserves_fixed_rotations() {
    let pieces = vec![
        piece(1, test_shapes::rectangle(40., 40.)),
        piece(2, test_shapes::rectangle(40., 40.)),
    ];
    let stock = SheetConfig {
        width: 100.,
        height: 100.,
        separation: 5.,
    };
    let rotations = vec![RotationRange::None; 2];
    let b = result(vec![
        vec![placement(&pieces[0], 0., 0., 0.)],
        vec![placement(&pieces[1], 0., 0., 0.)],
    ]);
    let r = improve(
        &pieces,
        &rotations,
        &stock,
        &b,
        Duration::from_secs(1),
        &AtomicBool::new(false),
        |r| assert_eq!(r.sheets.len(), 1),
        |_| {},
    )
    .unwrap();
    assert_eq!(r.sheets.len(), 1);
    validate(&r, &pieces, &rotations, &stock).unwrap();
}
#[test]
fn validates_identities_rotations_spacing_boundaries_and_containment() {
    let pieces = vec![
        piece(1, test_shapes::rectangle(40., 40.)),
        piece(2, test_shapes::rectangle(20., 20.)),
    ];
    let stock = SheetConfig {
        width: 100.,
        height: 100.,
        separation: 5.,
    };
    let rotations = vec![RotationRange::Discrete(vec![0.]); 2];
    let good = result(vec![vec![
        placement(&pieces[0], 0., 0., 0.),
        placement(&pieces[1], 45., 0., 0.),
    ]]);
    assert!(validate(&good, &pieces, &rotations, &stock).is_ok());
    for (x, y, a, id) in [
        (10., 10., 0., 2),
        (41., 0., 0., 2),
        (90., 0., 0., 2),
        (45., 0., 0.1, 2),
        (45., 0., 0., 1),
    ] {
        let mut r = good.clone();
        let p = &mut r.sheets[0].placements[1];
        p.x = x;
        p.y = y;
        p.rotation = a;
        p.piece_id = PieceId(id);
        assert!(validate(&r, &pieces, &rotations, &stock).is_err())
    }
    let mut r = good;
    r.sheets[0].placements.pop();
    assert!(validate(&r, &pieces, &rotations, &stock).is_err());
}
#[test]
fn cancellation_and_deadline_keep_baseline_without_updates() {
    let pieces = vec![piece(1, test_shapes::rectangle(40., 40.))];
    let stock = SheetConfig::default();
    let b = result(vec![vec![placement(&pieces[0], 0., 0., 0.)]]);
    for (budget, cancel) in [(Duration::ZERO, false), (Duration::from_secs(3), true)] {
        let began = Instant::now();
        let r = improve(
            &pieces,
            &[RotationRange::None],
            &stock,
            &b,
            budget,
            &AtomicBool::new(cancel),
            |_| panic!("unexpected update"),
            |_| {},
        )
        .unwrap();
        assert!(began.elapsed() < Duration::from_millis(100));
        assert_eq!(
            serde_json::to_value(r).unwrap(),
            serde_json::to_value(&b).unwrap()
        );
    }
}

#[test]
fn zero_spacing_allows_touching_but_rejects_shared_interior() {
    let pieces = vec![
        piece(1, test_shapes::rectangle(40., 40.)),
        piece(2, test_shapes::rectangle(40., 40.)),
    ];
    let stock = SheetConfig {
        width: 100.,
        height: 100.,
        separation: 0.,
    };
    let rotations = vec![RotationRange::None; 2];
    for (x, valid) in [(40., true), (20., false), (0., false)] {
        let r = result(vec![vec![
            placement(&pieces[0], 0., 0., 0.),
            placement(&pieces[1], x, 0., 0.),
        ]]);
        assert_eq!(
            validate(&r, &pieces, &rotations, &stock).is_ok(),
            valid,
            "x={x}"
        );
    }
}

#[test]
fn emitted_layout_survives_cancellation_from_callback() {
    let pieces = vec![
        piece(1, test_shapes::rectangle(40., 40.)),
        piece(2, test_shapes::rectangle(40., 40.)),
    ];
    let stock = SheetConfig {
        width: 100.,
        height: 100.,
        separation: 5.,
    };
    let rotations = vec![RotationRange::None; 2];
    let b = result(vec![
        vec![placement(&pieces[0], 0., 0., 0.)],
        vec![placement(&pieces[1], 0., 0., 0.)],
    ]);
    let cancel = AtomicBool::new(false);
    let mut emitted = None;
    let r = improve(
        &pieces,
        &rotations,
        &stock,
        &b,
        Duration::from_secs(3),
        &cancel,
        |r| {
            emitted = Some(r);
            cancel.store(true, std::sync::atomic::Ordering::Relaxed);
        },
        |_| {},
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(r).unwrap(),
        serde_json::to_value(emitted.unwrap()).unwrap()
    );
}

#[test]
fn gls_honors_its_stage_deadline() {
    use thetta_nesting::{
        engine::NestingEngine,
        gls::{GLSOptimizer, OptimizerConfig},
    };
    let pieces = vec![
        piece(1, test_shapes::rectangle(90., 90.)),
        piece(2, test_shapes::rectangle(90., 90.)),
    ];
    let stock = SheetConfig {
        width: 100.,
        height: 100.,
        separation: 5.,
    };
    let engine = NestingEngine::new(&pieces, &stock);
    let mut gls = GLSOptimizer::new(engine, OptimizerConfig::default());
    let began = Instant::now();
    gls.run(|_| {}, |_, _, _, _| {}, Duration::from_millis(5), None);
    assert!(began.elapsed() < Duration::from_millis(500));
}

#[test]
fn repairs_legacy_clearance_before_publishing() {
    let pieces = vec![
        piece(1, test_shapes::rectangle(40., 40.)),
        piece(2, test_shapes::rectangle(40., 40.)),
    ];
    let stock = SheetConfig {
        width: 100.,
        height: 100.,
        separation: 12.,
    };
    let rotations = vec![RotationRange::None; 2];
    let baseline = result(vec![vec![
        placement(&pieces[0], 0., 0., 0.),
        placement(&pieces[1], 42., 0., 0.),
    ]]);
    assert!(validate(&baseline, &pieces, &rotations, &stock).is_err());
    let mut emitted = 0;
    let r = improve(
        &pieces,
        &rotations,
        &stock,
        &baseline,
        Duration::from_secs(1),
        &AtomicBool::new(false),
        |r| {
            validate(&r, &pieces, &rotations, &stock).unwrap();
            emitted += 1;
        },
        |_| {},
    )
    .unwrap();
    assert!(emitted > 0);
    validate(&r, &pieces, &rotations, &stock).unwrap();
    assert_eq!(r.sheets.len(), 1);
    let r = improve(
        &pieces,
        &rotations,
        &stock,
        &baseline,
        Duration::ZERO,
        &AtomicBool::new(false),
        |_| panic!("deadline expired"),
        |_| {},
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(r).unwrap(),
        serde_json::to_value(baseline).unwrap()
    );
}

#[test]
fn compacts_panel_seven_with_four_hooks_and_two_strips() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/sparse-hook-panel.json")).unwrap();
    let pieces: Vec<Piece> = serde_json::from_value(fixture["pieces"].clone()).unwrap();
    let stock: SheetConfig = serde_json::from_value(fixture["stock"].clone()).unwrap();
    let baseline: NestingResult = serde_json::from_value(fixture["baseline"].clone()).unwrap();
    let rotations = vec![RotationRange::Continuous; pieces.len()];
    let footprint = |r: &NestingResult| {
        let mut points = vec![];
        for p in &r.sheets[0].placements {
            let piece = pieces.iter().find(|x| x.id == p.piece_id).unwrap();
            let polygon = polygon_ops::rotate(&piece.polygon, p.rotation, Point2D { x: 0., y: 0. });
            points.extend(polygon.outer.into_iter().map(|v| (v.x + p.x, v.y + p.y)));
        }
        let minx = points.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
        let maxx = points.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
        let miny = points.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
        let maxy = points.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
        (maxx - minx) * (maxy - miny)
    };
    let r = improve(
        &pieces,
        &rotations,
        &stock,
        &baseline,
        Duration::from_secs(12),
        &AtomicBool::new(false),
        |r| validate(&r, &pieces, &rotations, &stock).unwrap(),
        |_| {},
    )
    .unwrap();
    assert_eq!(r.sheets.len(), 1);
    validate(&r, &pieces, &rotations, &stock).unwrap();
    let reduction = 1. - footprint(&r) / footprint(&baseline);
    println!("panel 7 footprint reduction: {:.1}%", reduction * 100.);
    assert!(
        reduction >= 0.30,
        "Actual sparse hook panel must improve materially: {reduction}"
    );
}

#[test]
fn left_packing_uses_height_even_when_the_bounding_area_increases() {
    let pieces = vec![
        piece(1, test_shapes::rectangle(40., 10.)),
        piece(2, test_shapes::rectangle(40., 10.)),
    ];
    let stock = SheetConfig {
        width: 100.,
        height: 100.,
        separation: 5.,
    };
    let rotations = vec![RotationRange::None; 2];
    let baseline = result(vec![vec![
        placement(&pieces[0], 0., 0., 0.),
        placement(&pieces[1], 45., 0., 0.),
    ]]);
    let r = improve(
        &pieces,
        &rotations,
        &stock,
        &baseline,
        Duration::from_secs(1),
        &AtomicBool::new(false),
        |_| {},
        |_| {},
    )
    .unwrap();
    validate(&r, &pieces, &rotations, &stock).unwrap();
    assert_eq!(r.sheets.len(), 1);
    assert!(
        r.sheets[0].strip_length <= 40.01,
        "Must stack at the left rather than use an 85 mm wide row: {}",
        r.sheets[0].strip_length
    );
    assert!(r.sheets[0].placements.iter().all(|p| p.x.abs() < 0.01));
    let height = r.sheets[0]
        .placements
        .iter()
        .map(|p| p.y + 10.)
        .fold(0., f64::max);
    assert!(height >= 24.99);
}

#[test]
fn left_packing_rotates_narrow_and_anchors_an_offset_cluster() {
    let pieces = vec![piece(1, test_shapes::rectangle(60., 20.))];
    let stock = SheetConfig {
        width: 100.,
        height: 100.,
        separation: 5.,
    };
    let rotations = vec![RotationRange::Continuous];
    let baseline = result(vec![vec![placement(&pieces[0], 30., 5., 0.)]]);
    let r = improve(
        &pieces,
        &rotations,
        &stock,
        &baseline,
        Duration::from_secs(1),
        &AtomicBool::new(false),
        |_| {},
        |_| {},
    )
    .unwrap();
    validate(&r, &pieces, &rotations, &stock).unwrap();
    assert!(r.sheets[0].strip_length <= 20.01);
    let p = &r.sheets[0].placements[0];
    let poly = polygon_ops::rotate(&pieces[0].polygon, p.rotation, Point2D { x: 0., y: 0. });
    let left = poly
        .outer
        .iter()
        .map(|v| v.x + p.x)
        .fold(f64::INFINITY, f64::min);
    assert!(left.abs() < 0.01);
}

#[test]
fn merges_actual_panels_seven_and_eight_by_moving_clusters_together() {
    let f: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/merge-hook-panels.json")).unwrap();
    let pieces: Vec<Piece> = serde_json::from_value(f["pieces"].clone()).unwrap();
    let stock: SheetConfig = serde_json::from_value(f["stock"].clone()).unwrap();
    let baseline: NestingResult = serde_json::from_value(f["baseline"].clone()).unwrap();
    let rotations = vec![RotationRange::Continuous; pieces.len()];
    let cancel = AtomicBool::new(false);
    let mut accepted = false;
    let r = improve(
        &pieces,
        &rotations,
        &stock,
        &baseline,
        Duration::from_secs(5),
        &cancel,
        |r| {
            validate(&r, &pieces, &rotations, &stock).unwrap();
            if r.sheets.len() == 1 {
                accepted = true;
                cancel.store(true, std::sync::atomic::Ordering::Relaxed);
            }
        },
        |_| {},
    )
    .unwrap();
    assert!(
        accepted,
        "All nine parts from the actual sparse panels must combine"
    );
    assert_eq!(r.sheets.len(), 1);
    assert_eq!(r.sheets[0].placements.len(), 9);
    validate(&r, &pieces, &rotations, &stock).unwrap();
}

#[test]
fn consolidates_fresh_panels_without_preserving_their_original_groups() {
    let f: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/unpaired-hook-panels.json")).unwrap();
    let pieces: Vec<Piece> = serde_json::from_value(f["pieces"].clone()).unwrap();
    let stock: SheetConfig = serde_json::from_value(f["stock"].clone()).unwrap();
    let baseline: NestingResult = serde_json::from_value(f["baseline"].clone()).unwrap();
    let rotations = vec![RotationRange::Continuous; pieces.len()];
    assert_eq!(baseline.sheets[0].placements.len(), 7);
    assert_eq!(baseline.sheets[1].placements.len(), 2);
    validate(&baseline, &pieces, &rotations, &stock).unwrap();
    let r = consolidate(
        &pieces,
        &rotations,
        &stock,
        &baseline,
        Duration::from_secs(10),
        &AtomicBool::new(false),
        |r| validate(&r, &pieces, &rotations, &stock).unwrap(),
        |_| {},
    )
    .unwrap();
    assert_eq!(r.sheets.len(), 1);
    assert_eq!(r.sheets[0].placements.len(), 9);
    validate(&r, &pieces, &rotations, &stock).unwrap();
}

#[test]
fn preserves_valid_small_parts_in_a_border_gap() {
    let pieces = vec![
        piece(1, test_shapes::rectangle(90., 90.)),
        piece(2, test_shapes::rectangle(3., 3.)),
    ];
    let stock = SheetConfig {
        width: 100.,
        height: 100.,
        separation: 5.,
    };
    let rotations = vec![RotationRange::None; 2];
    let baseline = result(vec![
        vec![placement(&pieces[0], 0., 0., 0.)],
        vec![placement(&pieces[1], 0., 0., 0.)],
    ]);
    let r = improve(
        &pieces,
        &rotations,
        &stock,
        &baseline,
        Duration::from_secs(1),
        &AtomicBool::new(false),
        |_| {},
        |_| {},
    )
    .unwrap();
    assert_eq!(r.sheets.len(), 1);
    validate(&r, &pieces, &rotations, &stock).unwrap();
}
