use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Polygon {
    /// Outer boundary (CCW winding)
    pub outer: Vec<Point2D>,
    /// Holes (CW winding)
    #[serde(default)]
    pub holes: Vec<Vec<Point2D>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PieceId(pub u64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Piece {
    pub id: PieceId,
    pub polygon: Polygon,
    #[serde(default)]
    pub source_file: PathBuf,
    #[serde(default = "default_quantity")]
    pub quantity: u32,
}

fn default_quantity() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Placement {
    pub piece_id: PieceId,
    pub x: f64,
    pub y: f64,
    pub rotation: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl BoundingBox {
    pub fn width(&self) -> f64 {
        self.max_x - self.min_x
    }

    pub fn height(&self) -> f64 {
        self.max_y - self.min_y
    }

    pub fn contains(&self, p: Point2D) -> bool {
        p.x >= self.min_x && p.x <= self.max_x && p.y >= self.min_y && p.y <= self.max_y
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetConfig {
    pub width: f64,
    pub height: f64,
    pub separation: f64,
}

impl Default for SheetConfig {
    fn default() -> Self {
        Self {
            width: 2440.0,
            height: 1220.0,
            separation: 2.0,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NestingResult {
    pub sheets: Vec<Sheet>,
    #[serde(default)]
    pub unplaced: Vec<PieceId>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Sheet {
    pub placements: Vec<Placement>,
    pub utilization: f64,
    #[serde(default)]
    pub strip_length: f64,
}
