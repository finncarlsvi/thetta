use crate::geometry::DTransformation;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExtItem {
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_orientations: Option<Vec<f32>>,
    pub shape: ExtShape,
    pub min_quality: Option<usize>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum ExtShape {
    Rectangle {
        x_min: f32,
        y_min: f32,
        width: f32,
        height: f32,
    },
    SimplePolygon(ExtSPolygon),
    Polygon(ExtPolygon),
    MultiPolygon(Vec<ExtPolygon>),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExtPolygon {
    pub outer: ExtSPolygon,
    #[serde(default)]
    pub inner: Vec<ExtSPolygon>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExtSPolygon(pub Vec<(f32, f32)>);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExtTransformation {
    pub rotation: f32,
    pub translation: (f32, f32),
}

impl From<DTransformation> for ExtTransformation {
    fn from(dt: DTransformation) -> Self {
        Self {
            rotation: dt.rotation().to_degrees(),
            translation: dt.translation(),
        }
    }
}

impl From<ExtTransformation> for DTransformation {
    fn from(ext_dt: ExtTransformation) -> Self {
        DTransformation::new(ext_dt.rotation.to_radians(), ext_dt.translation)
    }
}
