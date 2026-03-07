use anyhow::{Result, bail};

use crate::geometry::geo_traits::Transformable;
use crate::geometry::transformation::Transformation;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point(pub f32, pub f32);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x_min: f32,
    pub y_min: f32,
    pub x_max: f32,
    pub y_max: f32,
}

impl Rect {
    pub fn try_new(x_min: f32, y_min: f32, x_max: f32, y_max: f32) -> Result<Self> {
        if x_max < x_min || y_max < y_min {
            bail!("invalid rect");
        }
        Ok(Self {
            x_min,
            y_min,
            x_max,
            y_max,
        })
    }

    pub fn width(&self) -> f32 {
        self.x_max - self.x_min
    }

    pub fn height(&self) -> f32 {
        self.y_max - self.y_min
    }

    pub fn surrounds(&self, other: &Rect) -> bool {
        other.x_min >= self.x_min
            && other.y_min >= self.y_min
            && other.x_max <= self.x_max
            && other.y_max <= self.y_max
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        !(self.x_max < other.x_min
            || other.x_max < self.x_min
            || self.y_max < other.y_min
            || other.y_max < self.y_min)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Edge {
    pub start: Point,
    pub end: Point,
}

#[derive(Clone, Debug)]
pub struct SPolygon {
    pub vertices: Vec<Point>,
    pub bbox: Rect,
    pub area: f32,
}

impl SPolygon {
    pub fn new(points: Vec<Point>) -> Result<Self> {
        if points.len() < 3 {
            bail!("polygon must have at least 3 vertices");
        }
        let area = Self::calculate_area(&points).abs();
        if area <= f32::EPSILON {
            bail!("polygon has no area");
        }
        let bbox = Self::bbox_of(&points);
        Ok(Self {
            vertices: points,
            bbox,
            area,
        })
    }

    pub fn n_vertices(&self) -> usize {
        self.vertices.len()
    }

    pub fn edge_iter(&self) -> impl Iterator<Item = Edge> + '_ {
        (0..self.vertices.len()).map(|i| {
            let j = (i + 1) % self.vertices.len();
            Edge {
                start: self.vertices[i],
                end: self.vertices[j],
            }
        })
    }

    pub fn centroid(&self) -> Point {
        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut a = 0.0;

        for i in 0..self.vertices.len() {
            let j = (i + 1) % self.vertices.len();
            let Point(x0, y0) = self.vertices[i];
            let Point(x1, y1) = self.vertices[j];
            let cross = x0 * y1 - x1 * y0;
            a += cross;
            cx += (x0 + x1) * cross;
            cy += (y0 + y1) * cross;
        }

        if a.abs() < f32::EPSILON {
            return self.vertices[0];
        }
        let s = 1.0 / (3.0 * a);
        Point(cx * s, cy * s)
    }

    pub fn inflate(&mut self, distance: f32) {
        let Point(cx, cy) = self.centroid();
        for Point(x, y) in &mut self.vertices {
            let dx = *x - cx;
            let dy = *y - cy;
            let len = (dx * dx + dy * dy).sqrt();
            if len > f32::EPSILON {
                *x += dx / len * distance;
                *y += dy / len * distance;
            }
        }
        self.bbox = Self::bbox_of(&self.vertices);
        self.area = Self::calculate_area(&self.vertices).abs();
    }

    fn bbox_of(points: &[Point]) -> Rect {
        let mut x_min = f32::INFINITY;
        let mut y_min = f32::INFINITY;
        let mut x_max = f32::NEG_INFINITY;
        let mut y_max = f32::NEG_INFINITY;
        for Point(x, y) in points {
            x_min = x_min.min(*x);
            y_min = y_min.min(*y);
            x_max = x_max.max(*x);
            y_max = y_max.max(*y);
        }
        Rect {
            x_min,
            y_min,
            x_max,
            y_max,
        }
    }

    fn calculate_area(points: &[Point]) -> f32 {
        let mut sum = 0.0;
        for i in 0..points.len() {
            let j = (i + 1) % points.len();
            let Point(x0, y0) = points[i];
            let Point(x1, y1) = points[j];
            sum += x0 * y1 - x1 * y0;
        }
        0.5 * sum
    }
}

impl Transformable for SPolygon {
    fn transform(&mut self, t: &Transformation) -> &mut Self {
        let (sin_r, cos_r) = t.rotation.sin_cos();
        for Point(x, y) in &mut self.vertices {
            let rx = *x * cos_r - *y * sin_r;
            let ry = *x * sin_r + *y * cos_r;
            *x = rx + t.translation.0;
            *y = ry + t.translation.1;
        }
        self.bbox = Self::bbox_of(&self.vertices);
        self
    }
}
