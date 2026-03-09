use std::collections::BTreeMap;

use crate::collision_detection::hazards::collector::HazardCollector;
use crate::collision_detection::hazards::filter::HazardFilter;
use crate::collision_detection::hazards::{HazKey, Hazard, HazardEntity};
use crate::entities::PItemKey;
use crate::geometry::fail_fast::SPSurrogateConfig;
use crate::geometry::primitives::{Point, Rect, SPolygon};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct CDEConfig {
    pub quadtree_depth: u8,
    pub cd_threshold: u8,
    pub item_surrogate_config: SPSurrogateConfig,
}

#[derive(Clone, Debug)]
pub struct CDEngine {
    bbox: Rect,
    pub hazards_map: BTreeMap<HazKey, Hazard>,
    pub config: CDEConfig,
    hkey_exterior: HazKey,
    next_key: usize,
}

impl CDEngine {
    pub fn new(bbox: Rect, static_hazards: Vec<Hazard>, config: CDEConfig) -> Self {
        let mut hazards_map = BTreeMap::new();
        let mut next_key = 0;
        for haz in static_hazards {
            hazards_map.insert(HazKey(next_key), haz);
            next_key += 1;
        }

        let hkey_exterior = hazards_map
            .iter()
            .find(|(_, h)| matches!(h.entity, HazardEntity::Exterior))
            .map(|(hk, _)| *hk)
            .unwrap_or_else(|| {
                let hk = HazKey(next_key);
                next_key += 1;
                hazards_map.insert(
                    hk,
                    Hazard::new(
                        HazardEntity::Exterior,
                        SPolygon::new(vec![
                            Point(bbox.x_min, bbox.y_min),
                            Point(bbox.x_max, bbox.y_min),
                            Point(bbox.x_max, bbox.y_max),
                            Point(bbox.x_min, bbox.y_max),
                        ])
                        .expect("valid bbox polygon"),
                        false,
                    ),
                );
                hk
            });

        Self {
            bbox,
            hazards_map,
            config,
            hkey_exterior,
            next_key,
        }
    }

    pub fn register_hazard(&mut self, hazard: Hazard) {
        self.hazards_map.insert(HazKey(self.next_key), hazard);
        self.next_key += 1;
    }

    pub fn deregister_hazard_by_entity(&mut self, hazard_entity: HazardEntity) -> Hazard {
        let key = self
            .hazards_map
            .iter()
            .find(|(_, h)| h.entity == hazard_entity)
            .map(|(k, _)| *k)
            .expect("hazard not found");
        self.hazards_map.remove(&key).expect("hazard key missing")
    }

    pub fn detect_poly_collision(&self, shape: &SPolygon, filter: &impl HazardFilter) -> bool {
        if !self.bbox.surrounds(&shape.bbox) {
            return true;
        }

        self.hazards_map
            .iter()
            .filter(|(hk, _)| !filter.is_irrelevant(**hk))
            .any(|(_, h)| match h.entity {
                HazardEntity::Exterior => false,
                _ => polygons_overlap(shape, &h.shape),
            })
    }

    pub fn collect_poly_collisions(&self, shape: &SPolygon, collector: &mut impl HazardCollector) {
        if !self.bbox.surrounds(&shape.bbox) {
            collector.insert(self.hkey_exterior, HazardEntity::Exterior);
        }

        for (hk, hazard) in &self.hazards_map {
            if collector.contains_key(*hk) {
                continue;
            }
            if matches!(hazard.entity, HazardEntity::Exterior) {
                continue;
            }
            if polygons_overlap(shape, &hazard.shape) {
                collector.insert(*hk, hazard.entity);
            }
        }
    }

    pub fn haz_key_from_pi_key(&self, pik: PItemKey) -> Option<HazKey> {
        self.hazards_map
            .iter()
            .find(|(_, hazard)| match hazard.entity {
                HazardEntity::PlacedItem { pk, .. } => pk == pik,
                _ => false,
            })
            .map(|(key, _)| *key)
    }

    pub fn bbox(&self) -> Rect {
        self.bbox
    }
}

fn polygons_overlap(a: &SPolygon, b: &SPolygon) -> bool {
    if !a.bbox.intersects(&b.bbox) {
        return false;
    }

    for ea in a.edge_iter() {
        for eb in b.edge_iter() {
            if segments_intersect(ea.start, ea.end, eb.start, eb.end) {
                return true;
            }
        }
    }

    point_in_polygon(a.vertices[0], &b.vertices) || point_in_polygon(b.vertices[0], &a.vertices)
}

fn segments_intersect(p1: Point, p2: Point, q1: Point, q2: Point) -> bool {
    fn orient(a: Point, b: Point, c: Point) -> f32 {
        (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)
    }

    fn on_seg(a: Point, b: Point, c: Point) -> bool {
        c.0 >= a.0.min(b.0) && c.0 <= a.0.max(b.0) && c.1 >= a.1.min(b.1) && c.1 <= a.1.max(b.1)
    }

    let o1 = orient(p1, p2, q1);
    let o2 = orient(p1, p2, q2);
    let o3 = orient(q1, q2, p1);
    let o4 = orient(q1, q2, p2);

    if (o1 > 0.0) != (o2 > 0.0) && (o3 > 0.0) != (o4 > 0.0) {
        return true;
    }

    let eps = 1e-6;
    (o1.abs() < eps && on_seg(p1, p2, q1))
        || (o2.abs() < eps && on_seg(p1, p2, q2))
        || (o3.abs() < eps && on_seg(q1, q2, p1))
        || (o4.abs() < eps && on_seg(q1, q2, p2))
}

/// Ray-casting point-in-polygon test.
fn point_in_polygon(p: Point, vertices: &[Point]) -> bool {
    let mut inside = false;
    for i in 0..vertices.len() {
        let j = (i + 1) % vertices.len();
        let a = vertices[i];
        let b = vertices[j];
        let dy = b.1 - a.1;
        let intersects =
            (a.1 > p.1) != (b.1 > p.1) && p.0 < (b.0 - a.0) * (p.1 - a.1) / dy + a.0;
        if intersects {
            inside = !inside;
        }
    }
    inside
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pip_triangle_interior() {
        // Triangle (0,0), (10,0), (5,10) — point (3,5) is inside
        let verts = vec![Point(0.0, 0.0), Point(10.0, 0.0), Point(5.0, 10.0)];
        assert!(point_in_polygon(Point(3.0, 5.0), &verts));
        assert!(point_in_polygon(Point(5.0, 3.0), &verts));
    }

    #[test]
    fn pip_triangle_exterior() {
        let verts = vec![Point(0.0, 0.0), Point(10.0, 0.0), Point(5.0, 10.0)];
        assert!(!point_in_polygon(Point(-1.0, 5.0), &verts));
        assert!(!point_in_polygon(Point(15.0, 5.0), &verts));
    }

    #[test]
    fn pip_square() {
        let verts = vec![
            Point(0.0, 0.0),
            Point(10.0, 0.0),
            Point(10.0, 10.0),
            Point(0.0, 10.0),
        ];
        assert!(point_in_polygon(Point(5.0, 5.0), &verts));
        assert!(!point_in_polygon(Point(15.0, 5.0), &verts));
    }

    #[test]
    fn overlap_containment_detected() {
        // Small square entirely inside large square — no edge crossings,
        // overlap must be detected via point-in-polygon.
        let big = SPolygon::new(vec![
            Point(0.0, 0.0),
            Point(100.0, 0.0),
            Point(100.0, 100.0),
            Point(0.0, 100.0),
        ])
        .unwrap();
        let small = SPolygon::new(vec![
            Point(30.0, 30.0),
            Point(60.0, 30.0),
            Point(60.0, 60.0),
            Point(30.0, 60.0),
        ])
        .unwrap();
        assert!(polygons_overlap(&big, &small));
        assert!(polygons_overlap(&small, &big));
    }

    #[test]
    fn overlap_separated_not_detected() {
        let a = SPolygon::new(vec![
            Point(0.0, 0.0),
            Point(10.0, 0.0),
            Point(10.0, 10.0),
            Point(0.0, 10.0),
        ])
        .unwrap();
        let b = SPolygon::new(vec![
            Point(20.0, 0.0),
            Point(30.0, 0.0),
            Point(30.0, 10.0),
            Point(20.0, 10.0),
        ])
        .unwrap();
        assert!(!polygons_overlap(&a, &b));
    }

    #[test]
    fn overlap_triangle_containment() {
        // Small triangle inside large triangle — tests downward edges
        let big = SPolygon::new(vec![
            Point(0.0, 0.0),
            Point(100.0, 0.0),
            Point(50.0, 100.0),
        ])
        .unwrap();
        let small = SPolygon::new(vec![
            Point(40.0, 10.0),
            Point(60.0, 10.0),
            Point(50.0, 30.0),
        ])
        .unwrap();
        assert!(polygons_overlap(&big, &small));
        assert!(polygons_overlap(&small, &big));
    }
}
