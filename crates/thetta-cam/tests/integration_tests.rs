use std::collections::HashMap;

use thetta_cam::generate::generate_all_sheets;
use thetta_geometry::test_shapes;
use thetta_geometry::types::{
    GcodeConfig, NestingResult, Piece, PieceId, Placement, Point2D, ProfileCutConfig, Sheet, Tab, Tool,
};

fn sample_tool() -> Tool {
    Tool {
        number: 1,
        name: "6mm".to_string(),
        diameter: 6.0,
        flute_count: 2,
        feed_rate: 1500.0,
        plunge_rate: 500.0,
        spindle_speed: 18000,
    }
}

fn sample_cfg() -> GcodeConfig {
    GcodeConfig {
        safe_z: 5.0,
        cut_depth: 18.0,
        tool_number: Some(1),
        coordinate_system: "G54".to_string(),
    }
}

#[test]
fn test_empty_sheet_skipped() {
    let result = NestingResult {
        sheets: vec![Sheet::default()],
        unplaced: vec![],
    };
    let pieces = vec![];
    let tabs: HashMap<(usize, usize), Vec<Tab>> = HashMap::new();

    let gcode = generate_all_sheets(
        &result,
        &pieces,
        &tabs,
        &sample_tool(),
        &sample_cfg(),
        &ProfileCutConfig::default(),
    );

    assert!(gcode.is_empty());
}

#[test]
fn test_multi_piece_sheet_generates_program() {
    let pieces = vec![
        Piece {
            id: PieceId(1),
            polygon: test_shapes::rectangle(100.0, 50.0),
            source_file: Default::default(),
            quantity: 1,
        },
        Piece {
            id: PieceId(2),
            polygon: test_shapes::rectangle(80.0, 40.0),
            source_file: Default::default(),
            quantity: 1,
        },
    ];

    let result = NestingResult {
        sheets: vec![Sheet {
            placements: vec![
                Placement {
                    piece_id: PieceId(1),
                    x: 0.0,
                    y: 0.0,
                    rotation: 0.0,
                },
                Placement {
                    piece_id: PieceId(2),
                    x: 200.0,
                    y: 0.0,
                    rotation: 0.0,
                },
            ],
            utilization: 0.0,
            strip_length: 0.0,
        }],
        unplaced: vec![],
    };

    let mut tabs: HashMap<(usize, usize), Vec<Tab>> = HashMap::new();
    tabs.insert(
        (0, 0),
        vec![Tab {
            position: Point2D { x: 50.0, y: 0.0 },
            edge_index: 0,
            t: 0.5,
            width: 10.0,
            height: 6.0,
        }],
    );

    let all = generate_all_sheets(
        &result,
        &pieces,
        &tabs,
        &sample_tool(),
        &sample_cfg(),
        &ProfileCutConfig::default(),
    );

    assert_eq!(all.len(), 1);
    let text = &all[0].1;
    assert!(text.contains("(Piece 1)"));
    assert!(text.contains("(Piece 2)"));
    assert!(text.contains("G2") || text.contains("G3"));
}
