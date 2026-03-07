use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone)]
pub struct Polygon {
    /// Outer boundary (CCW winding)
    pub outer: Vec<Point2D>,
    /// Holes (CW winding)
    pub holes: Vec<Vec<Point2D>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PieceId(pub u64);

#[derive(Debug, Clone)]
pub struct Piece {
    pub id: PieceId,
    pub polygon: Polygon,
    pub source_file: PathBuf,
    pub quantity: u32,
}

#[derive(Debug, Clone)]
pub struct Placement {
    pub piece_id: PieceId,
    pub x: f64,
    pub y: f64,
    pub rotation: f64,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct NestingResult {
    pub sheets: Vec<Sheet>,
}

#[derive(Debug, Clone)]
pub struct Sheet {
    pub placements: Vec<Placement>,
    pub utilization: f64,
}
