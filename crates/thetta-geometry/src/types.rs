use serde::{Deserialize, Serialize};
use std::collections::HashSet;
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

/// A cutting tool in the persistent tool database.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tool {
    /// LinuxCNC tool number (T1, T2, ...).
    pub number: u32,
    /// Human-readable tool name.
    pub name: String,
    /// Tool diameter in mm.
    pub diameter: f64,
    /// Number of cutting flutes.
    pub flute_count: u32,
    /// Horizontal feed rate in mm/min.
    pub feed_rate: f64,
    /// Vertical plunge rate in mm/min.
    pub plunge_rate: f64,
    /// Spindle speed in RPM.
    pub spindle_speed: u32,
}

impl Tool {
    pub fn validate(&self) -> Result<(), String> {
        if self.diameter <= 0.0 || !self.diameter.is_finite() {
            return Err(format!("T{}: diameter must be positive", self.number));
        }
        if self.feed_rate <= 0.0 || !self.feed_rate.is_finite() {
            return Err(format!("T{}: feed rate must be positive", self.number));
        }
        if self.plunge_rate <= 0.0 || !self.plunge_rate.is_finite() {
            return Err(format!("T{}: plunge rate must be positive", self.number));
        }
        if self.spindle_speed == 0 {
            return Err(format!("T{}: spindle speed must be positive", self.number));
        }
        Ok(())
    }
}

/// Persistent collection of tools.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ToolTable {
    pub tools: Vec<Tool>,
}

impl ToolTable {
    pub fn by_number(&self, number: u32) -> Option<&Tool> {
        self.tools.iter().find(|tool| tool.number == number)
    }

    pub fn next_number(&self) -> u32 {
        self.tools.iter().map(|tool| tool.number).max().unwrap_or(0) + 1
    }

    pub fn validate(&self) -> Result<(), String> {
        let mut seen = HashSet::new();
        for tool in &self.tools {
            if !seen.insert(tool.number) {
                return Err(format!("Duplicate tool number: T{}", tool.number));
            }
            tool.validate()?;
        }
        Ok(())
    }
}

/// Parameters shared across G-code generation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GcodeConfig {
    /// Safe Z height for rapid moves, in mm.
    pub safe_z: f64,
    /// Full profile depth into material, in mm.
    pub cut_depth: f64,
    /// Selected tool number for profile operations.
    pub tool_number: Option<u32>,
    /// Active machine coordinate system (`G54`, `G55`, ...).
    pub coordinate_system: String,
}

impl Default for GcodeConfig {
    fn default() -> Self {
        Self {
            safe_z: 5.0,
            cut_depth: 18.0,
            tool_number: None,
            coordinate_system: "G54".to_string(),
        }
    }
}

impl GcodeConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.safe_z <= 0.0 || !self.safe_z.is_finite() {
            return Err("Safe Z must be positive".to_string());
        }
        if self.cut_depth <= 0.0 || !self.cut_depth.is_finite() {
            return Err("Cut depth must be positive".to_string());
        }
        if self.tool_number.is_none() {
            return Err("No tool selected".to_string());
        }
        if self.coordinate_system.trim().is_empty() {
            return Err("Coordinate system must not be empty".to_string());
        }
        Ok(())
    }
}

/// Cutting strategy direction for profile cuts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CutDirection {
    Conventional,
    Climb,
}

/// Parameters for profile cut generation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProfileCutConfig {
    /// Radius of lead-in / lead-out tangential arc, in mm.
    pub lead_in_radius: f64,
    /// Direction around contour.
    pub cutting_direction: CutDirection,
}

impl Default for ProfileCutConfig {
    fn default() -> Self {
        Self {
            lead_in_radius: 3.0,
            cutting_direction: CutDirection::Conventional,
        }
    }
}

/// 3D point used by CAM motion planning.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// A single planned tool motion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ToolpathMove {
    Rapid {
        to: Point3D,
    },
    LinearCut {
        to: Point3D,
        feed: f64,
    },
    ArcCut {
        to: Point3D,
        center: Point2D,
        clockwise: bool,
        feed: f64,
    },
    TabLift {
        to: Point3D,
        tab_height: f64,
    },
    Retract {
        to_z: f64,
    },
    Plunge {
        to_z: f64,
        feed: f64,
    },
}

/// Complete toolpath for one operation.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Toolpath {
    pub moves: Vec<ToolpathMove>,
    pub config: GcodeConfig,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tool(number: u32) -> Tool {
        Tool {
            number,
            name: "6mm 2F".to_string(),
            diameter: 6.0,
            flute_count: 2,
            feed_rate: 1500.0,
            plunge_rate: 500.0,
            spindle_speed: 18000,
        }
    }

    #[test]
    fn tool_table_next_number_and_lookup() {
        let mut table = ToolTable::default();
        assert_eq!(table.next_number(), 1);
        table.tools.push(sample_tool(2));
        table.tools.push(sample_tool(10));
        assert_eq!(table.next_number(), 11);
        assert_eq!(table.by_number(2).map(|t| t.number), Some(2));
        assert!(table.by_number(3).is_none());
    }

    #[test]
    fn tool_table_validate_rejects_duplicate_numbers() {
        let table = ToolTable {
            tools: vec![sample_tool(1), sample_tool(1)],
        };
        let err = table.validate().unwrap_err();
        assert!(err.contains("Duplicate tool number"));
    }

    #[test]
    fn tool_table_validate_rejects_invalid_tool_fields() {
        let mut bad = sample_tool(1);
        bad.diameter = 0.0;
        let table = ToolTable { tools: vec![bad] };
        assert!(table.validate().is_err());
    }

    #[test]
    fn gcode_config_validate_rejects_invalid_values() {
        let cfg = GcodeConfig {
            safe_z: -1.0,
            ..GcodeConfig::default()
        };
        assert!(cfg.validate().is_err());

        let cfg = GcodeConfig {
            cut_depth: 0.0,
            ..GcodeConfig::default()
        };
        assert!(cfg.validate().is_err());

        let cfg = GcodeConfig::default();
        assert!(cfg.validate().is_err());

        let cfg = GcodeConfig {
            tool_number: Some(1),
            ..GcodeConfig::default()
        };
        assert!(cfg.validate().is_ok());
    }
}
