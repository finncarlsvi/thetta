use crate::types::{Point2D, Polygon};
use std::f64::consts::TAU;

pub fn rectangle(width: f64, height: f64) -> Polygon {
    Polygon {
        outer: vec![
            Point2D { x: 0.0, y: 0.0 },
            Point2D { x: width, y: 0.0 },
            Point2D {
                x: width,
                y: height,
            },
            Point2D { x: 0.0, y: height },
        ],
        holes: vec![],
    }
}

pub fn l_shape(width: f64, height: f64, cutout: f64) -> Polygon {
    Polygon {
        outer: vec![
            Point2D { x: 0.0, y: 0.0 },
            Point2D { x: width, y: 0.0 },
            Point2D { x: width, y: cutout },
            Point2D {
                x: cutout,
                y: cutout,
            },
            Point2D {
                x: cutout,
                y: height,
            },
            Point2D { x: 0.0, y: height },
        ],
        holes: vec![],
    }
}

pub fn t_shape(top_width: f64, top_height: f64, stem_width: f64, stem_height: f64) -> Polygon {
    let stem_left = (top_width - stem_width) / 2.0;
    let stem_right = stem_left + stem_width;
    Polygon {
        outer: vec![
            Point2D { x: 0.0, y: 0.0 },
            Point2D {
                x: top_width,
                y: 0.0,
            },
            Point2D {
                x: top_width,
                y: top_height,
            },
            Point2D {
                x: stem_right,
                y: top_height,
            },
            Point2D {
                x: stem_right,
                y: top_height + stem_height,
            },
            Point2D {
                x: stem_left,
                y: top_height + stem_height,
            },
            Point2D {
                x: stem_left,
                y: top_height,
            },
            Point2D {
                x: 0.0,
                y: top_height,
            },
        ],
        holes: vec![],
    }
}

pub fn triangle(width: f64, height: f64) -> Polygon {
    Polygon {
        outer: vec![
            Point2D { x: 0.0, y: 0.0 },
            Point2D { x: width, y: 0.0 },
            Point2D {
                x: width / 2.0,
                y: height,
            },
        ],
        holes: vec![],
    }
}

pub fn circle(radius: f64, segments: usize) -> Polygon {
    let segments = segments.max(3);
    let outer = (0..segments)
        .map(|i| {
            let t = i as f64 * TAU / segments as f64;
            Point2D {
                x: radius * t.cos(),
                y: radius * t.sin(),
            }
        })
        .collect();
    Polygon {
        outer,
        holes: vec![],
    }
}
