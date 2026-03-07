pub mod collector;
pub mod filter;

use crate::entities::PItemKey;
use crate::geometry::primitives::SPolygon;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HazKey(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HazardEntity {
    Exterior,
    PlacedItem { pk: PItemKey, item_id: usize },
}

#[derive(Clone, Debug)]
pub struct Hazard {
    pub entity: HazardEntity,
    pub shape: SPolygon,
    pub dynamic: bool,
}

impl Hazard {
    pub fn new(entity: HazardEntity, shape: SPolygon, dynamic: bool) -> Self {
        Self {
            entity,
            shape,
            dynamic,
        }
    }
}
