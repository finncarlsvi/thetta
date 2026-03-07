use std::sync::Arc;

use anyhow::Result;

use crate::collision_detection::{CDEConfig, CDEngine};
use crate::collision_detection::hazards::{Hazard, HazardEntity};
use crate::geometry::primitives::{Point, Rect, SPolygon};
use crate::geometry::shape_modification::ShapeModifyConfig;

#[derive(Clone, Debug)]
pub struct Container {
    pub id: usize,
    pub shape_modify_config: ShapeModifyConfig,
    pub bbox: Rect,
    pub base_cde: Arc<CDEngine>,
}

impl Container {
    pub fn new_strip(
        id: usize,
        width: f32,
        height: f32,
        cde_config: CDEConfig,
        shape_modify_config: ShapeModifyConfig,
    ) -> Result<Self> {
        let bbox = Rect::try_new(0.0, 0.0, width, height)?;
        let shape = SPolygon::new(vec![
            Point(0.0, 0.0),
            Point(width, 0.0),
            Point(width, height),
            Point(0.0, height),
        ])?;

        let exterior = Hazard::new(HazardEntity::Exterior, shape, false);
        let base_cde = Arc::new(CDEngine::new(bbox, vec![exterior], cde_config));

        Ok(Self {
            id,
            shape_modify_config,
            bbox,
            base_cde,
        })
    }

    pub fn area(&self) -> f32 {
        self.bbox.width() * self.bbox.height()
    }
}
