use std::sync::Arc;

use anyhow::Result;

use crate::geometry::OriginalShape;
use crate::geometry::fail_fast::SPSurrogateConfig;
use crate::geometry::geo_enums::RotationRange;
use crate::geometry::primitives::SPolygon;

#[derive(Clone, Debug)]
pub struct Item {
    pub id: usize,
    pub shape_orig: Arc<OriginalShape>,
    pub shape_cd: Arc<SPolygon>,
    pub allowed_rotation: RotationRange,
    pub min_quality: Option<usize>,
    pub surrogate_config: SPSurrogateConfig,
}

impl Item {
    pub fn new(
        id: usize,
        original_shape: OriginalShape,
        allowed_rotation: RotationRange,
        min_quality: Option<usize>,
        surrogate_config: SPSurrogateConfig,
    ) -> Result<Self> {
        let shape_orig = Arc::new(original_shape);
        let shape_cd = Arc::new(shape_orig.convert_to_internal()?);

        Ok(Self {
            id,
            shape_orig,
            shape_cd,
            allowed_rotation,
            min_quality,
            surrogate_config,
        })
    }

    pub fn area(&self) -> f32 {
        self.shape_orig.area()
    }
}
