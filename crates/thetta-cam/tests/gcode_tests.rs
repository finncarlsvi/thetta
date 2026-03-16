use thetta_cam::gcode::emit_gcode;
use thetta_geometry::types::{GcodeConfig, Point2D, Point3D, Tool, Toolpath, ToolpathMove};

fn tool() -> Tool {
    Tool {
        number: 1,
        name: "Router Ø6".to_string(),
        diameter: 6.0,
        flute_count: 2,
        feed_rate: 1500.0,
        plunge_rate: 500.0,
        spindle_speed: 18000,
    }
}

fn cfg() -> GcodeConfig {
    GcodeConfig {
        safe_z: 5.0,
        cut_depth: 18.0,
        tool_number: Some(1),
        coordinate_system: "G54".to_string(),
    }
}

#[test]
fn test_arc_move_format() {
    let tp = Toolpath {
        moves: vec![
            ToolpathMove::Rapid {
                to: Point3D {
                    x: 1.0,
                    y: 2.0,
                    z: 5.0,
                },
            },
            ToolpathMove::ArcCut {
                to: Point3D {
                    x: 4.0,
                    y: 2.0,
                    z: -1.0,
                },
                center: Point2D { x: 2.5, y: 2.0 },
                clockwise: true,
                feed: 500.0,
            },
        ],
        config: cfg(),
    };

    let g = emit_gcode(&[tp], &tool(), &cfg(), 0);
    assert!(g.contains("G2"));
    assert!(g.contains("I1.500 J0.000"));
}

#[test]
fn test_z_never_positive_during_cut_moves() {
    let tp = Toolpath {
        moves: vec![
            ToolpathMove::LinearCut {
                to: Point3D {
                    x: 10.0,
                    y: 0.0,
                    z: 2.0,
                },
                feed: 1000.0,
            },
            ToolpathMove::TabLift {
                to: Point3D {
                    x: 20.0,
                    y: 0.0,
                    z: 1.0,
                },
                tab_height: 20.0,
            },
        ],
        config: cfg(),
    };

    let g = emit_gcode(&[tp], &tool(), &cfg(), 0);
    for line in g.lines() {
        if line.starts_with("G1")
            && line.contains(" Z")
            && let Some(z) = line.split_whitespace().find(|tok| tok.starts_with('Z'))
        {
            let value: f64 = z.trim_start_matches('Z').parse().unwrap();
            assert!(value <= 0.0 + 1e-9, "line had positive cutting Z: {line}");
        }
    }
}

#[test]
fn test_ascii_comments() {
    let g = emit_gcode(&[], &tool(), &cfg(), 0);
    for line in g.lines().filter(|l| l.starts_with('(')) {
        assert!(line.is_ascii());
    }
}
