use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct ExtSPInstance {
    pub name: String,
    pub items: Vec<ExtItem>,
    pub strip_height: f32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ExtItem {
    #[serde(flatten)]
    pub base: crate::io::ext_repr::ExtItem,
    pub demand: u64,
}
