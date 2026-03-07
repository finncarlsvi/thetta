use anyhow::Result;

use crate::geometry::d_transformation::DTransformation;
use crate::geometry::geo_traits::Transformable;
use crate::geometry::primitives::SPolygon;
use crate::geometry::shape_modification::ShapeModifyConfig;

#[derive(Clone, Debug)]
pub struct OriginalShape {
    pub shape: SPolygon,
    pub pre_transform: DTransformation,
    pub modify_config: ShapeModifyConfig,
}

impl OriginalShape {
    pub fn area(&self) -> f32 {
        self.shape.area
    }

    pub fn convert_to_internal(&self) -> Result<SPolygon> {
        let mut out = self.shape.clone();
        out.transform(&self.pre_transform.compose());

        if let Some(offset) = self.modify_config.offset
            && offset.abs() > f32::EPSILON
        {
            out.inflate(offset);
        }

        Ok(out)
    }
}
