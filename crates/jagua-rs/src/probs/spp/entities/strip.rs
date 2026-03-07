use anyhow::Result;

use crate::collision_detection::CDEConfig;
use crate::entities::Container;
use crate::geometry::shape_modification::ShapeModifyConfig;

#[derive(Clone, Copy, Debug)]
pub struct Strip {
    pub height: f32,
    pub width: f32,
    pub cde_config: CDEConfig,
    pub shape_modify_config: ShapeModifyConfig,
}

impl Strip {
    pub fn new(
        height: f32,
        cde_config: CDEConfig,
        shape_modify_config: ShapeModifyConfig,
        width: f32,
    ) -> Result<Self> {
        Ok(Self {
            height,
            width,
            cde_config,
            shape_modify_config,
        })
    }

    pub fn set_width(&mut self, width: f32) {
        self.width = width;
    }
}

impl From<Strip> for Container {
    fn from(value: Strip) -> Self {
        Container::new_strip(
            0,
            value.width,
            value.height,
            value.cde_config,
            value.shape_modify_config,
        )
        .expect("valid strip")
    }
}
