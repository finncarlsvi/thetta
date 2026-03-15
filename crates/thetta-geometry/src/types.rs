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
            separation: 12.0,
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

/// Material type for tab sizing heuristics (RESEARCH-005 Section 4.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaterialType {
    WoodSoft,
    WoodHard,
    Plastic,
    Composite,
}

/// A single holding tab on a piece contour.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tab {
    /// World-space position on the contour edge.
    pub position: Point2D,
    /// Index of the edge in the polygon outer ring.
    pub edge_index: usize,
    /// Parameter along the edge (0.0 = start, 1.0 = end).
    pub t: f64,
    /// Tab width in mm along cutting direction.
    pub width: f64,
    /// Tab height in mm from cut bottom.
    pub height: f64,
}

/// Configuration for automatic tab placement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TabConfig {
    /// Target distance between tabs in mm.
    pub tab_distance: f64,
    /// Absolute minimum tab count per part.
    pub min_tabs: usize,
    /// Absolute maximum tab count per part.
    pub max_tabs: usize,
    /// Tab width in mm (manual mode), or computed from material.
    pub tab_width: f64,
    /// Tab height in mm (manual mode), or computed from material.
    pub tab_height: f64,
    /// Minimum perimeter in mm for a part to receive tabs.
    pub size_threshold: f64,
    /// Minimum distance from any polygon vertex in mm.
    pub corner_exclusion: f64,
    /// Material thickness in mm (used for auto sizing).
    pub material_thickness: f64,
    /// Material type (used for auto sizing).
    pub material_type: MaterialType,
    /// True: derive width/height from material settings. False: use manual values.
    pub auto_size: bool,
}

impl Default for TabConfig {
    fn default() -> Self {
        Self {
            tab_distance: 50.0,
            min_tabs: 3,
            max_tabs: 20,
            tab_width: 10.0,
            tab_height: 6.0,
            size_threshold: 30.0,
            corner_exclusion: 5.0,
            material_thickness: 18.0,
            material_type: MaterialType::WoodSoft,
            auto_size: true,
        }
    }
}

impl TabConfig {
    /// Compute effective tab height based on material properties.
    pub fn effective_tab_height(&self) -> f64 {
        if !self.auto_size {
            return self.tab_height;
        }

        let ratio = match self.material_type {
            MaterialType::WoodSoft => 0.40,
            MaterialType::WoodHard => 0.33,
            MaterialType::Plastic => 0.33,
            MaterialType::Composite => 0.50,
        };
        ratio * self.material_thickness
    }

    /// Compute effective tab width based on material properties.
    pub fn effective_tab_width(&self) -> f64 {
        if !self.auto_size {
            return self.tab_width;
        }
        (1.5 * self.effective_tab_height()).clamp(5.0, 25.0)
    }

    /// Validated tab distance (minimum 1.0 mm).
    pub fn effective_tab_distance(&self) -> f64 {
        self.tab_distance.max(1.0)
    }
}
