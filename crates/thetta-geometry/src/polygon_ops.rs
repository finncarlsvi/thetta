use crate::types::{BoundingBox, Point2D, Polygon};

pub fn area(poly: &Polygon) -> f64 {
    fn ring_area(points: &[Point2D]) -> f64 {
        if points.len() < 3 {
            return 0.0;
        }
        let mut sum = 0.0;
        for i in 0..points.len() {
            let j = (i + 1) % points.len();
            sum += points[i].x * points[j].y - points[j].x * points[i].y;
        }
        0.5 * sum.abs()
    }

    let holes_area: f64 = poly.holes.iter().map(|h| ring_area(h)).sum();
    ring_area(&poly.outer) - holes_area
}

pub fn bounding_box(poly: &Polygon) -> BoundingBox {
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for p in poly
        .outer
        .iter()
        .copied()
        .chain(poly.holes.iter().flat_map(|h| h.iter().copied()))
    {
        min_x = min_x.min(p.x);
        min_y = min_y.min(p.y);
        max_x = max_x.max(p.x);
        max_y = max_y.max(p.y);
    }

    if !min_x.is_finite() {
        return BoundingBox {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 0.0,
            max_y: 0.0,
        };
    }

    BoundingBox {
        min_x,
        min_y,
        max_x,
        max_y,
    }
}

pub fn translate(poly: &Polygon, dx: f64, dy: f64) -> Polygon {
    let mut out = poly.clone();
    for p in &mut out.outer {
        p.x += dx;
        p.y += dy;
    }
    for hole in &mut out.holes {
        for p in hole {
            p.x += dx;
            p.y += dy;
        }
    }
    out
}

pub fn rotate(poly: &Polygon, angle: f64, center: Point2D) -> Polygon {
    let (sin_a, cos_a) = angle.sin_cos();
    let mut out = poly.clone();

    let rotate_point = |p: &mut Point2D| {
        let tx = p.x - center.x;
        let ty = p.y - center.y;
        p.x = center.x + tx * cos_a - ty * sin_a;
        p.y = center.y + tx * sin_a + ty * cos_a;
    };

    for p in &mut out.outer {
        rotate_point(p);
    }
    for hole in &mut out.holes {
        for p in hole {
            rotate_point(p);
        }
    }

    out
}

pub fn offset(poly: &Polygon, distance: f64) -> Polygon {
    // Lightweight centroid-radial offset for sprint baseline.
    fn ring_centroid(points: &[Point2D]) -> Point2D {
        if points.is_empty() {
            return Point2D { x: 0.0, y: 0.0 };
        }
        let (sx, sy) = points
            .iter()
            .fold((0.0, 0.0), |(sx, sy), p| (sx + p.x, sy + p.y));
        let n = points.len() as f64;
        Point2D {
            x: sx / n,
            y: sy / n,
        }
    }

    fn offset_ring(points: &[Point2D], distance: f64, center: Point2D) -> Vec<Point2D> {
        points
            .iter()
            .map(|p| {
                let dx = p.x - center.x;
                let dy = p.y - center.y;
                let len = (dx * dx + dy * dy).sqrt();
                if len <= f64::EPSILON {
                    *p
                } else {
                    Point2D {
                        x: p.x + (dx / len) * distance,
                        y: p.y + (dy / len) * distance,
                    }
                }
            })
            .collect()
    }

    let center = ring_centroid(&poly.outer);
    Polygon {
        outer: offset_ring(&poly.outer, distance, center),
        holes: poly
            .holes
            .iter()
            .map(|h| offset_ring(h, -distance, center))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_shapes;

    const EPS: f64 = 1e-6;

    #[test]
    fn area_rectangle() {
        let p = test_shapes::rectangle(10.0, 20.0);
        assert!((area(&p) - 200.0).abs() < EPS);
    }

    #[test]
    fn bbox_triangle() {
        let p = test_shapes::triangle(10.0, 5.0);
        let b = bounding_box(&p);
        assert!((b.min_x - 0.0).abs() < EPS);
        assert!((b.min_y - 0.0).abs() < EPS);
        assert!((b.max_x - 10.0).abs() < EPS);
        assert!((b.max_y - 5.0).abs() < EPS);
    }

    #[test]
    fn translate_preserves_area() {
        let p = test_shapes::l_shape(12.0, 8.0, 4.0);
        let a0 = area(&p);
        let p2 = translate(&p, 100.0, -42.5);
        let a1 = area(&p2);
        assert!((a0 - a1).abs() < EPS);
    }

    #[test]
    fn rotate_preserves_area() {
        let p = test_shapes::t_shape(14.0, 4.0, 8.0, 10.0);
        let a0 = area(&p);
        let p2 = rotate(&p, std::f64::consts::FRAC_PI_3, Point2D { x: 0.0, y: 0.0 });
        let a1 = area(&p2);
        assert!((a0 - a1).abs() < EPS);
    }

    #[test]
    fn serde_roundtrip_polygon() {
        let p = test_shapes::circle(10.0, 32);
        let s = serde_json::to_string(&p).unwrap();
        let out: Polygon = serde_json::from_str(&s).unwrap();
        assert_eq!(p.outer.len(), out.outer.len());
        for (a, b) in p.outer.iter().zip(out.outer.iter()) {
            assert!((a.x - b.x).abs() < 1e-12);
            assert!((a.y - b.y).abs() < 1e-12);
        }
        assert_eq!(p.holes.len(), out.holes.len());
    }
}
