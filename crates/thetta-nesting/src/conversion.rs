use thetta_geometry::polygon_ops;
use thetta_geometry::types::{Piece, Placement, Polygon, SheetConfig};

#[derive(Debug, Clone)]
pub struct JaguaItem {
    pub id: usize,
    pub polygon: Polygon,
}

#[derive(Debug, Clone, Copy)]
pub struct JaguaContainer {
    pub width: f64,
    pub height: f64,
    pub separation: f64,
}

#[derive(Debug, Clone, Default)]
pub struct JaguaLayout {
    pub placements: Vec<Placement>,
}

pub fn piece_to_jagua_item(piece: &Piece, config: &SheetConfig) -> JaguaItem {
    let inflated = if config.separation > 0.0 {
        polygon_ops::offset(&piece.polygon, config.separation * 0.5)
    } else {
        piece.polygon.clone()
    };

    JaguaItem {
        id: piece.id.0 as usize,
        polygon: inflated,
    }
}

pub fn sheet_to_jagua_container(config: &SheetConfig) -> JaguaContainer {
    JaguaContainer {
        width: config.width,
        height: config.height,
        separation: config.separation,
    }
}

pub fn extract_placements(layout: &JaguaLayout) -> Vec<Placement> {
    layout.placements.clone()
}
