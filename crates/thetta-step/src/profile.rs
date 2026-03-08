use thetta_geometry::types::{Point2D, Polygon};

pub fn normalize_polygon(mut polygon: Polygon) -> Result<Polygon, String> {
    polygon.outer = normalize_ring(&polygon.outer)?;
    if polygon.outer.len() < 3 {
        return Err("outer ring must contain at least 3 points".to_string());
    }

    let mut holes = Vec::with_capacity(polygon.holes.len());
    for hole in polygon.holes {
        let ring = normalize_ring(&hole)?;
        if ring.len() >= 3 {
            holes.push(ring);
        }
    }
    polygon.holes = holes;

    enforce_winding(&mut polygon);

    // Center polygon at bbox midpoint so the nesting engine works with
    // origin-relative coordinates (better f32 precision, correct rotation).
    center_at_origin(&mut polygon);

    Ok(polygon)
}

fn center_at_origin(polygon: &mut Polygon) {
    let (mut min_x, mut min_y) = (f64::INFINITY, f64::INFINITY);
    let (mut max_x, mut max_y) = (f64::NEG_INFINITY, f64::NEG_INFINITY);
    for p in &polygon.outer {
        min_x = min_x.min(p.x);
        min_y = min_y.min(p.y);
        max_x = max_x.max(p.x);
        max_y = max_y.max(p.y);
    }
    let dx = (min_x + max_x) * 0.5;
    let dy = (min_y + max_y) * 0.5;
    if dx.abs() < 1e-9 && dy.abs() < 1e-9 {
        return;
    }
    for p in &mut polygon.outer {
        p.x -= dx;
        p.y -= dy;
    }
    for hole in &mut polygon.holes {
        for p in hole {
            p.x -= dx;
            p.y -= dy;
        }
    }
}

fn normalize_ring(points: &[Point2D]) -> Result<Vec<Point2D>, String> {
    if points.is_empty() {
        return Ok(Vec::new());
    }

    let mut cleaned = Vec::with_capacity(points.len());
    for p in points {
        if !p.x.is_finite() || !p.y.is_finite() {
            return Err("ring contains non-finite point".to_string());
        }
        let is_dup = cleaned
            .last()
            .is_some_and(|last: &Point2D| almost_eq(last.x, p.x) && almost_eq(last.y, p.y));
        if !is_dup {
            cleaned.push(*p);
        }
    }

    if cleaned.len() >= 2 {
        let first = cleaned[0];
        let last = cleaned[cleaned.len() - 1];
        if almost_eq(first.x, last.x) && almost_eq(first.y, last.y) {
            cleaned.pop();
        }
    }

    Ok(cleaned)
}

pub fn polygon_area(polygon: &Polygon) -> f64 {
    ring_area(&polygon.outer).abs()
        - polygon
            .holes
            .iter()
            .map(|ring| ring_area(ring).abs())
            .sum::<f64>()
}

pub fn transform_polygon(polygon: &Polygon, dx: f64, dy: f64, rotation: f64) -> Polygon {
    let (sin_a, cos_a) = rotation.sin_cos();
    let map_point = |p: &Point2D| Point2D {
        x: p.x * cos_a - p.y * sin_a + dx,
        y: p.x * sin_a + p.y * cos_a + dy,
    };

    let mut out = Polygon {
        outer: polygon.outer.iter().map(map_point).collect(),
        holes: polygon
            .holes
            .iter()
            .map(|ring| ring.iter().map(map_point).collect())
            .collect(),
    };
    enforce_winding(&mut out);
    out
}

pub fn enforce_winding(polygon: &mut Polygon) {
    if ring_area(&polygon.outer) < 0.0 {
        polygon.outer.reverse();
    }
    for hole in &mut polygon.holes {
        if ring_area(hole) > 0.0 {
            hole.reverse();
        }
    }
}

fn ring_area(ring: &[Point2D]) -> f64 {
    if ring.len() < 3 {
        return 0.0;
    }

    let mut sum = 0.0;
    for i in 0..ring.len() {
        let j = (i + 1) % ring.len();
        sum += ring[i].x * ring[j].y - ring[j].x * ring[i].y;
    }
    0.5 * sum
}

fn almost_eq(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_ring_duplicates_and_winding() {
        let poly = Polygon {
            outer: vec![
                Point2D { x: 0.0, y: 0.0 },
                Point2D { x: 0.0, y: 2.0 },
                Point2D { x: 2.0, y: 2.0 },
                Point2D { x: 2.0, y: 0.0 },
                Point2D { x: 0.0, y: 0.0 },
            ],
            holes: vec![vec![
                Point2D { x: 0.5, y: 0.5 },
                Point2D { x: 1.5, y: 0.5 },
                Point2D { x: 1.5, y: 1.5 },
                Point2D { x: 0.5, y: 1.5 },
            ]],
        };

        let normalized = normalize_polygon(poly).unwrap();
        assert_eq!(normalized.outer.len(), 4);
        assert!(ring_area(&normalized.outer) > 0.0);
        assert!(ring_area(&normalized.holes[0]) < 0.0);
    }
}
