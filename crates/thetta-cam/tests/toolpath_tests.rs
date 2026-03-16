use thetta_cam::toolpath::{offset_contour, plan_profile_toolpath};
use thetta_geometry::polygon_ops;
use thetta_geometry::test_shapes;
use thetta_geometry::types::{
    GcodeConfig, Point2D, ProfileCutConfig, Tab, Tool, ToolpathMove,
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
fn test_start_and_end_at_safe_z() {
    let rect = test_shapes::rectangle(100.0, 50.0);
    let path = plan_profile_toolpath(
        &rect,
        &[],
        &sample_tool(),
        &sample_cfg(),
        &ProfileCutConfig::default(),
    );

    match path.moves.first() {
        Some(ToolpathMove::Rapid { to }) => assert!((to.z - 5.0).abs() < 1e-6),
        other => panic!("unexpected first move: {other:?}"),
    }

    match path.moves.last() {
        Some(ToolpathMove::Retract { to_z }) => assert!((*to_z - 5.0).abs() < 1e-6),
        other => panic!("unexpected last move: {other:?}"),
    }
}

#[test]
fn test_offset_contour_dimensions() {
    let rect = test_shapes::rectangle(100.0, 50.0);
    let offset = offset_contour(&rect, 3.0);
    let bb = polygon_ops::bounding_box(&offset);
    assert!((bb.width() - 106.0).abs() < 1e-6);
    assert!((bb.height() - 56.0).abs() < 1e-6);
}

#[test]
fn test_deterministic_output() {
    let rect = test_shapes::rectangle(120.0, 80.0);
    let tabs = vec![Tab {
        position: Point2D { x: 60.0, y: 0.0 },
        edge_index: 0,
        t: 0.5,
        width: 10.0,
        height: 6.0,
    }];

    let a = plan_profile_toolpath(
        &rect,
        &tabs,
        &sample_tool(),
        &sample_cfg(),
        &ProfileCutConfig::default(),
    );
    let b = plan_profile_toolpath(
        &rect,
        &tabs,
        &sample_tool(),
        &sample_cfg(),
        &ProfileCutConfig::default(),
    );

    assert_eq!(a, b);
}
