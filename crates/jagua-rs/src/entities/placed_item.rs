use crate::entities::Item;
use crate::geometry::DTransformation;
use crate::geometry::geo_traits::Transformable;
use crate::geometry::primitives::SPolygon;

pub type PItemKey = usize;

#[derive(Clone, Debug)]
pub struct PlacedItem {
    pub item_id: usize,
    pub d_transf: DTransformation,
    pub shape: SPolygon,
}

impl PlacedItem {
    pub fn new(item: &Item, d_transf: DTransformation) -> Self {
        let shape = item.shape_cd.transform_clone(&d_transf.compose());
        Self {
            item_id: item.id,
            d_transf,
            shape,
        }
    }
}
