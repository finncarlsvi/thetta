use std::f64::consts::PI;

use thetta_geometry::polygon_ops::edge_length;
use thetta_geometry::types::{DrillHole, HoleClassification, PocketRegion, Point2D};

const EPS: f64 = 1e-9;

pub fn circularity(contour: &[Point2D]) -> f64 {
    if contour.len() < 3 {
        return 0.0;
    }

    let area = signed_ring_area(contour).abs();
    let perimeter = ring_perimeter(contour);
    if area <= EPS || perimeter <= EPS {
        return 0.0;
    }

    4.0 * PI * area / (perimeter * perimeter)
}

pub fn contour_center(contour: &[Point2D]) -> Point2D {
    if contour.is_empty() {
        return Point2D { x: 0.0, y: 0.0 };
    }

    let (sx, sy) = contour
        .iter()
        .fold((0.0, 0.0), |(sx, sy), p| (sx + p.x, sy + p.y));
    let n = contour.len() as f64;
    Point2D {
        x: sx / n,
        y: sy / n,
    }
}

pub fn contour_effective_diameter(contour: &[Point2D], center: Point2D) -> f64 {
    if contour.is_empty() {
        return 0.0;
    }

    let avg_radius = contour
        .iter()
        .map(|p| edge_length(*p, center))
        .sum::<f64>()
        / contour.len() as f64;
    2.0 * avg_radius
}

pub fn classify_holes(
    holes: &[Vec<Point2D>],
    circularity_threshold: f64,
    drill_tool_diameter: Option<f64>,
) -> Vec<HoleClassification> {
    let mut out = Vec::new();

    for hole in holes {
        if hole.len() < 3 {
            continue;
        }

        let score = circularity(hole);
        if score > circularity_threshold {
            if let Some(tool_dia) = drill_tool_diameter {
                let center = contour_center(hole);
                let hole_dia = contour_effective_diameter(hole, center);
                if tool_dia <= hole_dia {
                    out.push(HoleClassification::Drill(DrillHole {
                        center,
                        diameter: hole_dia,
                    }));
                } else {
                    out.push(HoleClassification::Pocket(PocketRegion {
                        contour: hole.clone(),
                    }));
                }
            } else {
                out.push(HoleClassification::Pocket(PocketRegion {
                    contour: hole.clone(),
                }));
            }
        } else {
            out.push(HoleClassification::Pocket(PocketRegion {
                contour: hole.clone(),
            }));
        }
    }

    out
}

fn signed_ring_area(points: &[Point2D]) -> f64 {
    if points.len() < 3 {
        return 0.0;
    }

    let mut sum = 0.0;
    for i in 0..points.len() {
        let j = (i + 1) % points.len();
        sum += points[i].x * points[j].y - points[j].x * points[i].y;
    }

    0.5 * sum
}

fn ring_perimeter(points: &[Point2D]) -> f64 {
    if points.len() < 2 {
        return 0.0;
    }

    let mut sum = 0.0;
    for i in 0..points.len() {
        let j = (i + 1) % points.len();
        sum += edge_length(points[i], points[j]);
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::*;

    fn circle(radius: f64, n: usize) -> Vec<Point2D> {
        (0..n)
            .map(|i| {
                let t = (i as f64 / n as f64) * std::f64::consts::TAU;
                Point2D {
                    x: radius * t.cos(),
                    y: radius * t.sin(),
                }
            })
            .collect()
    }

    fn rectangle(w: f64, h: f64) -> Vec<Point2D> {
        vec![
            Point2D { x: 0.0, y: 0.0 },
            Point2D { x: w, y: 0.0 },
            Point2D { x: w, y: h },
            Point2D { x: 0.0, y: h },
        ]
    }

    #[test]
    fn circle_classified_as_drill() {
        let holes = vec![circle(5.0, 32)];
        let out = classify_holes(&holes, 0.95, Some(9.0));
        assert_eq!(out.len(), 1);
        match &out[0] {
            HoleClassification::Drill(hole) => {
                assert!(hole.diameter > 9.5 && hole.diameter < 10.5);
                assert!(circularity(&holes[0]) > 0.99);
            }
            HoleClassification::Pocket(_) => panic!("expected drill"),
        }
    }

    #[test]
    fn rectangle_classified_as_pocket() {
        let holes = vec![rectangle(10.0, 5.0)];
        let out = classify_holes(&holes, 0.95, Some(20.0));
        assert!(matches!(out[0], HoleClassification::Pocket(_)));
        assert!(circularity(&holes[0]) < 0.80);
    }

    #[test]
    fn near_circle_threshold_dependent() {
        let holes = vec![circle(5.0, 8)];
        let hi = classify_holes(&holes, 0.96, Some(9.0));
        let lo = classify_holes(&holes, 0.90, Some(9.0));
        assert!(matches!(hi[0], HoleClassification::Pocket(_)));
        assert!(matches!(lo[0], HoleClassification::Drill(_)));
    }

    #[test]
    fn tool_larger_than_hole_becomes_pocket() {
        let holes = vec![circle(4.0, 32)];
        let out = classify_holes(&holes, 0.95, Some(10.0));
        assert!(matches!(out[0], HoleClassification::Pocket(_)));
    }

    #[test]
    fn empty_input_yields_empty() {
        let out = classify_holes(&[], 0.95, Some(6.0));
        assert!(out.is_empty());
    }

    #[test]
    fn degenerate_contour_skipped() {
        let holes = vec![vec![Point2D { x: 0.0, y: 0.0 }, Point2D { x: 1.0, y: 0.0 }]];
        let out = classify_holes(&holes, 0.95, Some(6.0));
        assert!(out.is_empty());
    }
}
