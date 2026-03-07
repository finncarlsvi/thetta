use crate::conversion::{JaguaContainer, JaguaItem, piece_to_jagua_item, sheet_to_jagua_container};
use thetta_geometry::polygon_ops;
use thetta_geometry::types::{NestingResult, Piece, Placement, Point2D, Polygon, Sheet, SheetConfig};

#[derive(Clone)]
pub struct NestingEngine {
    pieces: Vec<Piece>,
    inflated_items: Vec<JaguaItem>,
    sheet: SheetConfig,
    container: JaguaContainer,
    placements: Vec<Option<Placement>>,
}

impl NestingEngine {
    pub fn new(pieces: &[Piece], sheet: &SheetConfig) -> Self {
        Self {
            pieces: pieces.to_vec(),
            inflated_items: pieces
                .iter()
                .map(|p| piece_to_jagua_item(p, sheet))
                .collect(),
            sheet: sheet.clone(),
            container: sheet_to_jagua_container(sheet),
            placements: vec![None; pieces.len()],
        }
    }

    pub fn piece_count(&self) -> usize {
        self.pieces.len()
    }

    pub fn piece_area(&self, idx: usize) -> f64 {
        self.pieces
            .get(idx)
            .map(|p| polygon_ops::area(&p.polygon))
            .unwrap_or(0.0)
    }

    pub fn sheet_config(&self) -> &SheetConfig {
        &self.sheet
    }

    pub fn place_item(&mut self, idx: usize, x: f64, y: f64, rotation: f64) -> bool {
        if idx >= self.pieces.len() {
            return false;
        }

        let candidate = Placement {
            piece_id: self.pieces[idx].id,
            x,
            y,
            rotation,
        };

        if self.collision_cost(idx, x, y, rotation) > 0.0 {
            return false;
        }

        self.placements[idx] = Some(candidate);
        true
    }

    /// Place item ignoring collisions. Used by GLS which manages overlaps itself.
    pub fn force_place_item(&mut self, idx: usize, x: f64, y: f64, rotation: f64) {
        if idx >= self.pieces.len() {
            return;
        }
        self.placements[idx] = Some(Placement {
            piece_id: self.pieces[idx].id,
            x,
            y,
            rotation,
        });
    }

    pub fn remove_item(&mut self, idx: usize) {
        if idx < self.pieces.len() {
            self.placements[idx] = None;
        }
    }

    pub fn placement_of(&self, idx: usize) -> Option<&Placement> {
        self.placements.get(idx).and_then(|p| p.as_ref())
    }

    pub fn collision_cost(&self, idx: usize, x: f64, y: f64, rotation: f64) -> f64 {
        if idx >= self.pieces.len() {
            return f64::INFINITY;
        }

        let placement = Placement {
            piece_id: self.pieces[idx].id,
            x,
            y,
            rotation,
        };
        let poly = transformed_polygon(&self.inflated_items[idx].polygon, &placement);

        let mut cost = 0.0;
        if !within_bounds(&poly, self.container) {
            cost += 1.0;
        }

        for (other_idx, p) in self.placements.iter().enumerate() {
            if other_idx == idx {
                continue;
            }
            if let Some(other_placement) = p {
                let other_poly = transformed_polygon(&self.inflated_items[other_idx].polygon, other_placement);
                if polygons_overlap(&poly, &other_poly) {
                    cost += 1.0;
                }
            }
        }

        cost
    }

    pub fn current_result(&self) -> NestingResult {
        let placements = self
            .placements
            .iter()
            .filter_map(|p| p.clone())
            .collect::<Vec<_>>();
        let unplaced = self
            .pieces
            .iter()
            .enumerate()
            .filter(|(idx, _)| self.placements[*idx].is_none())
            .map(|(_, p)| p.id)
            .collect::<Vec<_>>();

        NestingResult {
            sheets: vec![Sheet {
                placements,
                utilization: self.utilization(),
            }],
            unplaced,
        }
    }

    pub fn utilization(&self) -> f64 {
        let used = self
            .pieces
            .iter()
            .enumerate()
            .filter(|(idx, _)| self.placements[*idx].is_some())
            .map(|(_, p)| polygon_ops::area(&p.polygon))
            .sum::<f64>();

        let total = self.sheet.width * self.sheet.height;
        if total <= f64::EPSILON {
            0.0
        } else {
            used / total
        }
    }
}

fn transformed_polygon(poly: &Polygon, placement: &Placement) -> Polygon {
    let rotated = polygon_ops::rotate(poly, placement.rotation, Point2D { x: 0.0, y: 0.0 });
    polygon_ops::translate(&rotated, placement.x, placement.y)
}

fn within_bounds(poly: &Polygon, container: JaguaContainer) -> bool {
    poly.outer
        .iter()
        .all(|p| p.x >= 0.0 && p.y >= 0.0 && p.x <= container.width && p.y <= container.height)
}

fn polygons_overlap(a: &Polygon, b: &Polygon) -> bool {
    let ab = polygon_ops::bounding_box(a);
    let bb = polygon_ops::bounding_box(b);
    if ab.max_x < bb.min_x || bb.max_x < ab.min_x || ab.max_y < bb.min_y || bb.max_y < ab.min_y {
        return false;
    }

    // Edge intersections.
    for i in 0..a.outer.len() {
        let a1 = a.outer[i];
        let a2 = a.outer[(i + 1) % a.outer.len()];
        for j in 0..b.outer.len() {
            let b1 = b.outer[j];
            let b2 = b.outer[(j + 1) % b.outer.len()];
            if segments_intersect(a1, a2, b1, b2) {
                return true;
            }
        }
    }

    // Containment.
    point_in_polygon(a.outer[0], &b.outer) || point_in_polygon(b.outer[0], &a.outer)
}

fn segments_intersect(p1: Point2D, p2: Point2D, q1: Point2D, q2: Point2D) -> bool {
    fn orient(a: Point2D, b: Point2D, c: Point2D) -> f64 {
        (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
    }

    fn on_seg(a: Point2D, b: Point2D, c: Point2D) -> bool {
        c.x >= a.x.min(b.x)
            && c.x <= a.x.max(b.x)
            && c.y >= a.y.min(b.y)
            && c.y <= a.y.max(b.y)
    }

    let o1 = orient(p1, p2, q1);
    let o2 = orient(p1, p2, q2);
    let o3 = orient(q1, q2, p1);
    let o4 = orient(q1, q2, p2);

    if (o1 > 0.0) != (o2 > 0.0) && (o3 > 0.0) != (o4 > 0.0) {
        return true;
    }

    let eps = 1e-9;
    (o1.abs() < eps && on_seg(p1, p2, q1))
        || (o2.abs() < eps && on_seg(p1, p2, q2))
        || (o3.abs() < eps && on_seg(q1, q2, p1))
        || (o4.abs() < eps && on_seg(q1, q2, p2))
}

fn point_in_polygon(p: Point2D, poly: &[Point2D]) -> bool {
    let mut inside = false;
    for i in 0..poly.len() {
        let j = (i + 1) % poly.len();
        let pi = poly[i];
        let pj = poly[j];
        let intersects = (pi.y > p.y) != (pj.y > p.y)
            && p.x < (pj.x - pi.x) * (p.y - pi.y) / (pj.y - pi.y + 1e-12) + pi.x;
        if intersects {
            inside = !inside;
        }
    }
    inside
}
