# Sprint 007: G-code Profile Cuts with Tool Database

## Overview

Implement G-code generation for profile cuts with tab-aware toolpaths, tool radius compensation, tangential arc lead-in/lead-out, and a persistent tool database, targeting LinuxCNC CNC routers. This is the first CAM sprint: after nesting and tab placement, the user selects a cutting tool, configures machining parameters, and exports one G-code file per sheet. The generated G-code cuts along each piece's outer contour at full depth, lifting the tool over holding tabs placed by Sprint 006.

The implementation introduces a new `thetta-cam` workspace crate that owns the toolpath planner, G-code emitter, and tool database persistence. Separating CAM from STEP I/O is deliberate: G-code generation has no dependency on OpenCASCADE, uses fundamentally different data structures (linear move sequences vs. solid geometry), and will grow to include hole drilling and pocket clearing in future sprints. The `thetta-cam` crate depends only on `thetta-geometry` for `Polygon`, `Placement`, `Tab`, and `Point2D` types.

The tool database stores tool definitions (number, diameter, flute count, feed/plunge rates, spindle speed) in a JSON file that persists across sessions. The UI provides tool CRUD, tool assignment to the profile operation, and a G-code configuration panel (safe-Z, cut depth). G-code generation runs on a background thread and produces one `.ngc` file per sheet, following LinuxCNC conventions: `G21` (metric), `G90` (absolute), `G54` (work coordinate system), `T/M6` tool changes, `G17` (XY plane), and proper preamble/postamble.

**Scope:** `Tool`, `ToolTable`, `GcodeConfig`, `ProfileCutConfig`, `ToolpathMove`, `Toolpath` types in `thetta-geometry`. `thetta-cam` crate with toolpath planner, G-code emitter, and tool database persistence. Tool database UI, G-code config panel, and export trigger in `thetta-app`. One G-code file per sheet. Profile cuts only (no holes/pockets). Hole contours (`Polygon.holes`) are explicitly skipped in this sprint.
**Prerequisites:** Sprint 006 complete. `NestingResult` with placed pieces. `TabState` with `HashMap<(usize, usize), Vec<Tab>>` per placement. Tab positions in world space. STEP/JSON export pipeline operational.
**Deferred:** Hole drilling, pocket clearing, cutter radius compensation via G41/G42, in-app G-code preview/simulation, tool wear compensation, LinuxCNC `.tbl` tool table export.

## Use Cases

### UC1: Create and Manage Tools
User opens the Tool Database panel, clicks "Add Tool", and enters parameters: tool number (T1), name ("6mm 2-flute upcut"), diameter (6mm), flute count (2), feed rate (1500 mm/min), plunge rate (500 mm/min), spindle speed (18000 RPM). The tool appears in the tool list. User can edit or delete tools. The tool database saves to disk automatically and loads on next app launch.

### UC2: Configure G-code Parameters
User sets machining parameters in the G-code panel: cut depth (18mm, matching material thickness), safe-Z height (5mm for rapid traverses above material). These parameters combine with the selected tool's feed/speed to fully define the machining operation.

### UC3: Assign Tool to Profile Operation
User selects a tool from the database for profile cuts. The tool's diameter determines the outward offset of the cutting contour (software tool radius compensation).

### UC4: Generate G-code for Single Sheet
User clicks "Generate G-code", selects an output directory via native save dialog. The app generates one `.ngc` file per sheet. The file contains a LinuxCNC-compatible program: tool change, spindle on, then for each piece on the sheet, the profile cut toolpath with tangential arc lead-in, tab-aware Z heights, and arc lead-out. Rapid moves between pieces at safe-Z.

### UC5: Tab-Aware Profile Cuts
At each tab position along a piece's offset contour, the tool ramps up from full cut depth to `cut_depth - tab_height` (leaving the tab uncut), traverses the tab width at the shallower depth, then ramps back down to full depth. Between tabs, the tool cuts at full depth.

### UC6: Tool Radius Compensation
The toolpath planner offsets each piece's outer contour outward by `tool.diameter / 2` using proper parallel offset (via `geo-buffer` crate or equivalent). Tab positions from the original contour are projected onto the offset contour using nearest-point projection. The generated G-code cuts along the offset contour so the finished part matches the original geometry.

### UC7: Tangential Arc Lead-In/Lead-Out
At each piece's entry point, the tool approaches the offset contour from outside via a small arc (G2/G3) tangent to the first contour edge. This prevents tool marks at the entry point. At contour completion, a mirror arc leads out before retracting.

### UC8: Multi-Sheet G-code Export
For multi-sheet nesting results, each sheet produces an independent G-code file (`sheet_1.ngc`, `sheet_2.ngc`, etc.). Each file is self-contained with its own preamble, tool change, and postamble. Empty sheets (no placements) produce no output file.

### UC9: Tool Database Persistence
Tool database saves to `~/.config/thetta/tools.json` (or platform equivalent via `dirs` crate). On first launch with no tool file, the database starts empty. Corrupt tool files produce a warning and start with an empty default.

### UC10: Validation Errors
User cannot export G-code without a tool selected, without a nesting result, or with invalid parameters (zero cut depth, negative safe-Z). The "Generate G-code" button is disabled with an actionable message explaining what's missing.

## Architecture

### Component Overview

```
+------------------------------------------------------------------+
|                          thetta-app                                |
|  +-----------+  +----------+  +--------------------------------+ |
|  | panels.rs |  |viewport.rs| |          state.rs              | |
|  | Tool DB   |  | (unchanged|  | ToolTable (loaded from disk)  | |
|  | G-code    |  |  this     |  | GcodeConfig, ProfileCutConfig | |
|  | config    |  |  sprint)  |  | G-code generation trigger     | |
|  | panel     |  |          |  | Background thread + channel   | |
|  +-----+-----+  +----------+  +---------------+----------------+ |
|        |                                       |                  |
+--------+---------------------------------------+------------------+
         |                                       |
         |                       +---------------v-----------------+
         |                       |          thetta-cam              |
         |                       |  +---------------------------+  |
         |                       |  |    toolpath.rs             |  |
         |                       |  | plan_profile_toolpath()    |  |
         |                       |  | Polygon offset, contour    |  |
         |                       |  | walk, tab projection,      |  |
         |                       |  | lead-in/out arcs           |  |
         |                       |  +---------------------------+  |
         |                       |  +---------------------------+  |
         |                       |  |    gcode.rs                |  |
         |                       |  | emit_gcode()               |  |
         |                       |  | Toolpath -> G-code text    |  |
         |                       |  | LinuxCNC dialect            |  |
         |                       |  +---------------------------+  |
         |                       |  +---------------------------+  |
         |                       |  |    generate.rs             |  |
         |                       |  | generate_sheet_gcode()     |  |
         |                       |  | Orchestrates per-sheet     |  |
         |                       |  | toolpath + emission         |  |
         |                       |  +---------------------------+  |
         |                       |  +---------------------------+  |
         |                       |  |    tool_db.rs              |  |
         |                       |  | load/save ToolTable JSON   |  |
         |                       |  +---------------------------+  |
         |                       +-----------------+---------------+
         |                                         |
         |                       +-----------------v---------------+
         |                       |        thetta-geometry           |
         +-------------------+-->|  types.rs                        |
                                 |  + Tool, ToolTable, GcodeConfig  |
                                 |  + ProfileCutConfig, ToolpathMove|
                                 |  + Toolpath                      |
                                 |  + (existing) Polygon, Tab, etc  |
                                 +---------------------------------+
```

### New Crate: `thetta-cam`

A new workspace crate at `crates/thetta-cam/` with dependencies on `thetta-geometry` only (plus `geo-buffer` or equivalent for polygon offset, `serde_json` for tool DB, and `dirs` for config paths). This crate has no UI, no OCCT, no nesting logic. Its sole job is: given placed piece contours, tabs, tool parameters, and G-code config, produce G-code text.

Rationale for a separate crate rather than extending `thetta-step`:
1. **No OCCT dependency.** G-code is plain text output. Putting it in `thetta-step` would couple CAM code with OCCT FFI, creating confusion about what requires the `occt` feature.
2. **Different domain.** STEP I/O deals with 3D solid geometry. G-code deals with 2.5D toolpath motion. The data models and concerns are different.
3. **Growth path.** Future sprints add hole drilling, pocket clearing, and operation ordering. These are CAM concepts that belong together, not mixed into an I/O crate.
4. **Testability.** `thetta-cam` can be tested independently with simple polygon fixtures, no STEP files needed.

### Data Flow

```
NestingResult + TabState + ToolTable + GcodeConfig + ProfileCutConfig
         |
         v
For each Sheet (sheet_index):
  |
  +-- 1. Collect placements and their tabs
  |
  +-- 2. For each Placement:
  |       |
  |       +-- a. Look up Piece polygon
  |       +-- b. Apply placement transform -> world-space contour
  |       +-- c. Offset contour outward by tool.diameter / 2
  |       +-- d. Look up tabs from TabState[(sheet_idx, placement_idx)]
  |       +-- e. Project tab positions onto offset contour (nearest-point)
  |       +-- f. Determine entry point (furthest from center, on straight edge, away from tabs)
  |       +-- g. Call plan_profile_toolpath(offset_contour, projected_tabs, tool, config)
  |       |       |
  |       |       +-- Generate tangential arc lead-in (G2/G3)
  |       |       +-- Walk offset contour edge-by-edge
  |       |       +-- At each tab: ramp up to (cut_depth - tab_height), traverse, ramp down
  |       |       +-- Generate tangential arc lead-out
  |       |       +-- Retract to safe-Z
  |       |       +-- Return Toolpath { moves: Vec<ToolpathMove> }
  |       |
  |       +-- h. Append toolpath to sheet program
  |
  +-- 3. Call emit_gcode(sheet_toolpaths, tool, config) -> String
  |       |
  |       +-- Preamble: %, G21 G90 G54, T/M6 tool change, S/M3 spindle on
  |       +-- For each piece toolpath:
  |       |     Rapid to approach XY at safe-Z -> arc lead-in with plunge ->
  |       |     profile moves with tab lifts -> arc lead-out -> retract
  |       +-- Postamble: M5 spindle off, G0 Z safe, G0 X0 Y0, M2, %
  |
  +-- 4. Write String to file: {output_dir}/sheet_{N}.ngc
```

### Coordinate System

The nesting coordinate origin is the bottom-left corner of the sheet. This maps directly to the G54 work coordinate system origin on the CNC machine. The user is responsible for setting G54 on the machine to match the physical sheet corner. Z=0 is the top surface of the stock material. Negative Z values cut into the material. Positive Z values are above the material (safe-Z for rapids).

### Intermediate Toolpath Representation

Rather than emitting G-code directly from contour geometry, the pipeline uses a `Toolpath` intermediate representation. This separation provides:
1. **Testability.** Toolpath generation can be unit-tested by inspecting move sequences without parsing G-code text.
2. **Flexibility.** The same toolpath can be emitted to different G-code dialects or post-processors in the future.
3. **Clarity.** Tab-aware Z height logic and arc lead-in geometry live in the toolpath planner, while G-code formatting lives in the emitter. Each module has a single concern.

### Threading Model

```
Main Thread (egui)                    Background Thread
+--------------------+               +---------------------------+
| - Click "Generate" |               | - Receives pieces, tabs,  |
| - Spawn thread     | -- inputs --> |   tool, config            |
| - Poll receiver    |               | - Calls generate_gcode()  |
|   each frame       | <-- mpsc --   | - Sends Result<Vec<Path>> |
| - Show status      |               |   or error                |
+--------------------+               +---------------------------+
```

Same `mpsc::channel` pattern as STEP export. The generation is fast (text output), but runs on a background thread to keep the UI responsive and consistent with existing patterns.

### Design Decisions

1. **New `thetta-cam` crate.** G-code has no relationship to STEP I/O. Separate crate keeps concerns clean and avoids polluting `thetta-step` with unrelated CAM logic.

2. **Intermediate `Toolpath` representation.** Decouples toolpath planning from G-code text emission. Enables unit testing of tab-aware height changes without parsing G-code strings.

3. **Software tool radius compensation (polygon offset).** The contour is offset outward by `tool.diameter / 2` in the planner before generating moves. This produces dimensionally correct parts without relying on LinuxCNC's G41/G42 cutter compensation, which has its own quirks (requires lead-in of at least one tool radius, varying behavior across versions). The existing `offset()` in `polygon_ops.rs` is a centroid-radial approximation and is not suitable. Use `geo-buffer` (already in workspace dependencies) or implement proper Clipper-style parallel offset.

4. **Tangential arc lead-in/lead-out (P0).** The tool approaches the contour from outside via a small arc (G2/G3) tangent to the first contour edge at the entry point. This prevents tool marks at the entry point and is standard CNC practice. The lead-in radius is configurable in `ProfileCutConfig`.

5. **Single full-depth pass for P0.** Multi-pass step-down adds significant complexity to tab-lift interactions (each pass must handle tabs differently). P0 cuts at full `cut_depth` in one pass. Step-down is P1.

6. **Conventional milling direction default (CCW for outer contour).** The outer contour is stored CCW. Following this winding produces conventional milling. Climb milling (CW) would require reversing the contour. Conventional is safer for CNC routers and is the default. Cutting direction option is P1.

7. **Entry point selection.** The profile cut starts at the point on the offset contour that is furthest from the part center, lies on a straight (long) edge, and is away from tab positions. This avoids tool marks on visible surfaces and prevents interference with tabs.

8. **Tab projection via nearest-point.** Tab positions are defined on the original polygon (from Sprint 006). After polygon offset, the contour geometry changes. For each tab: find the nearest point on the offset contour to the original tab world-space position. This is an approximation that works well for small tool radii relative to feature size. A sanity check warns if the projection distance exceeds the tool radius by more than 50%.

9. **Tool table as JSON with `dirs` crate.** Simple, human-readable, editable outside the app. The `dirs` crate provides `config_dir()` for cross-platform paths.

10. **One `.ngc` file per sheet.** Each file is self-contained. No multi-file coordination needed. Filenames include sheet index.

## Implementation

### Phase 1: Core Types + Tool DB (~25%)

**Goal:** Add CAM types to `thetta-geometry`, create the `thetta-cam` crate scaffold, and implement tool database persistence.

#### Task 1.1: CAM Types in `thetta-geometry`

**File:** `crates/thetta-geometry/src/types.rs`

```rust
/// A cutting tool in the tool database.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tool {
    /// LinuxCNC tool number (T1, T2, etc.).
    pub number: u32,
    /// Human-readable name (e.g., "6mm 2-flute upcut").
    pub name: String,
    /// Tool diameter in mm.
    pub diameter: f64,
    /// Number of cutting flutes.
    pub flute_count: u32,
    /// Feed rate in mm/min (horizontal cutting speed).
    pub feed_rate: f64,
    /// Plunge rate in mm/min (vertical entry speed).
    pub plunge_rate: f64,
    /// Spindle speed in RPM.
    pub spindle_speed: u32,
}

/// Persistent collection of cutting tools.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolTable {
    pub tools: Vec<Tool>,
}

impl ToolTable {
    /// Find a tool by its number.
    pub fn by_number(&self, number: u32) -> Option<&Tool> {
        self.tools.iter().find(|t| t.number == number)
    }

    /// Next available tool number.
    pub fn next_number(&self) -> u32 {
        self.tools.iter().map(|t| t.number).max().unwrap_or(0) + 1
    }

    /// Validate: no duplicate tool numbers, all diameters/feeds/speeds positive.
    pub fn validate(&self) -> Result<(), String> {
        let mut seen = std::collections::HashSet::new();
        for tool in &self.tools {
            if !seen.insert(tool.number) {
                return Err(format!("Duplicate tool number: T{}", tool.number));
            }
            if tool.diameter <= 0.0 {
                return Err(format!("T{}: diameter must be positive", tool.number));
            }
            if tool.feed_rate <= 0.0 {
                return Err(format!("T{}: feed rate must be positive", tool.number));
            }
            if tool.plunge_rate <= 0.0 {
                return Err(format!("T{}: plunge rate must be positive", tool.number));
            }
            if tool.spindle_speed == 0 {
                return Err(format!("T{}: spindle speed must be positive", tool.number));
            }
        }
        Ok(())
    }
}

/// Parameters for G-code generation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GcodeConfig {
    /// Safe Z height for rapid moves above material (mm, positive).
    pub safe_z: f64,
    /// Total cut depth into material (mm, positive value).
    pub cut_depth: f64,
    /// Tool number assigned to profile cuts.
    pub tool_number: Option<u32>,
    /// Coordinate system (default: "G54").
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

/// Configuration for profile cut operations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProfileCutConfig {
    /// Radius of the tangential arc lead-in/lead-out (mm).
    pub lead_in_radius: f64,
    /// Cutting direction.
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

/// Cutting direction for profile operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CutDirection {
    /// Tool moves opposite to feed direction at cutting edge. Safer for routers.
    Conventional,
    /// Tool moves in same direction as feed at cutting edge. Better finish.
    Climb,
}

/// A single motion in a toolpath.
#[derive(Debug, Clone, PartialEq)]
pub enum ToolpathMove {
    /// Rapid traverse (G0) to position. No cutting.
    Rapid { to: Point3D },
    /// Linear cutting move (G1) to position at given feed rate.
    LinearCut { to: Point3D, feed: f64 },
    /// Arc cutting move (G2/G3) to position via center point.
    ArcCut {
        to: Point3D,
        center: Point2D,
        clockwise: bool,
        feed: f64,
    },
    /// Tab lift: ramp Z up to tab height while moving to position.
    TabLift { to: Point3D, tab_height: f64 },
    /// Retract to safe Z height.
    Retract { to_z: f64 },
    /// Plunge to cutting depth.
    Plunge { to_z: f64, feed: f64 },
}

/// A 3D point (XYZ) for toolpath positions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// A complete toolpath for one piece profile or one operation.
#[derive(Debug, Clone, Default)]
pub struct Toolpath {
    pub moves: Vec<ToolpathMove>,
    pub config: GcodeConfig,
}
```

Validation helpers:
- `GcodeConfig::validate()` -> `Result<(), String>`: `safe_z > 0`, `cut_depth > 0`, `tool_number.is_some()`.
- `Tool` fields: `diameter > 0`, `feed_rate > 0`, `plunge_rate > 0`, `spindle_speed > 0`.

#### Task 1.2: Crate Scaffold

**New files:**
- `crates/thetta-cam/Cargo.toml`
- `crates/thetta-cam/src/lib.rs`
- `crates/thetta-cam/src/toolpath.rs`
- `crates/thetta-cam/src/gcode.rs`
- `crates/thetta-cam/src/generate.rs`
- `crates/thetta-cam/src/tool_db.rs`

**`Cargo.toml`:**
```toml
[package]
name = "thetta-cam"
version = "0.1.0"
edition = "2021"

[dependencies]
thetta-geometry = { path = "../thetta-geometry" }
serde_json = "1"
dirs = "5"
geo-buffer = "0.3"
geo = "0.29"
```

Add `"crates/thetta-cam"` to workspace members in root `Cargo.toml`.

**`lib.rs`:**
```rust
pub mod toolpath;
pub mod gcode;
pub mod generate;
pub mod tool_db;
```

#### Task 1.3: Tool Database Persistence

**File:** `crates/thetta-cam/src/tool_db.rs`

```rust
use std::path::PathBuf;
use thetta_geometry::types::ToolTable;

/// Default tool database file path.
pub fn tool_db_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("thetta").join("tools.json"))
}

/// Load tool table from disk. Returns default (empty) on any error.
pub fn load_tool_table() -> ToolTable {
    let Some(path) = tool_db_path() else {
        return ToolTable::default();
    };
    match std::fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => ToolTable::default(),
    }
}

/// Save tool table to disk using atomic write (temp file + rename).
/// Creates parent directory if needed.
pub fn save_tool_table(table: &ToolTable) -> Result<(), std::io::Error> {
    let Some(path) = tool_db_path() else {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "no config dir"));
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(table)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, &json)?;
    std::fs::rename(&tmp, &path)
}
```

#### Task 1.4: Unit Tests for Types and Tool DB

- `ToolTable::validate()` catches duplicate tool numbers, zero/negative parameters.
- `ToolTable::by_number()` finds correct tool or returns None.
- `ToolTable::next_number()` returns max+1 or 1 for empty table.
- `GcodeConfig::validate()` rejects zero cut depth, negative safe-Z, missing tool number.
- Tool DB roundtrip: save and load produces identical `ToolTable`.
- Tool DB missing file: returns empty default.
- Tool DB corrupt JSON: returns empty default.

### Phase 2: Toolpath Planner (~35%)

**Goal:** Implement polygon offset, contour walking, tab projection, lead-in/out arcs, and `ToolpathMove` generation.

#### Task 2.1: Polygon Offset

**File:** `crates/thetta-cam/src/toolpath.rs`

Implement `offset_contour(contour: &Polygon, distance: f64) -> Polygon` that offsets the outer ring outward by `distance` (tool radius). Use the `geo-buffer` crate to perform proper parallel offset that handles concave polygons correctly.

Conversion helpers: `Polygon` (thetta) <-> `geo::Polygon<f64>` (geo crate). Only the outer ring is offset; `Polygon.holes` are ignored in this sprint.

If the offset produces a multi-polygon (e.g., self-intersecting result from very tight concavities), take the largest polygon by area.

If `distance` is 0.0 (tool diameter 0, which is invalid but handled), return the original contour unchanged.

#### Task 2.2: Tab Projection onto Offset Contour

**File:** `crates/thetta-cam/src/toolpath.rs`

Implement `project_tabs(original_tabs: &[Tab], offset_contour: &Polygon) -> Vec<ProjectedTab>` where:

```rust
struct ProjectedTab {
    /// Position on the offset contour.
    position: Point2D,
    /// Edge index on the offset contour.
    edge_index: usize,
    /// Parameter along the offset edge.
    t: f64,
    /// Original tab width and height.
    width: f64,
    height: f64,
}
```

Algorithm:
1. For each original tab, compute its world-space position (already stored in `Tab.position`).
2. For each edge of the offset contour, compute the nearest point on that edge to the tab position.
3. Select the edge with the minimum distance.
4. Compute the `t` parameter and `edge_index` on the offset contour.
5. Sanity check: if the projection distance exceeds `tool_radius * 1.5`, log a warning (tab may be misplaced due to large offset on tight geometry).

#### Task 2.3: Entry Point Selection

**File:** `crates/thetta-cam/src/toolpath.rs`

Implement `select_entry_point(offset_contour: &Polygon, projected_tabs: &[ProjectedTab]) -> EntryPoint`:

```rust
struct EntryPoint {
    position: Point2D,
    edge_index: usize,
    t: f64,
    /// Unit normal pointing outward at the entry point (for lead-in direction).
    outward_normal: Point2D,
}
```

Algorithm:
1. Compute the centroid of the offset contour.
2. For each edge, compute its midpoint and the distance from centroid.
3. Score each edge: `distance_from_centroid * edge_length`. Prefer long edges far from center.
4. Penalize edges that contain a projected tab (subtract a large penalty).
5. Select the highest-scoring edge. The entry point is at the edge midpoint.
6. Compute the outward-facing normal at the entry point (perpendicular to the edge, pointing away from the polygon interior).

#### Task 2.4: Tangential Arc Lead-In/Lead-Out

**File:** `crates/thetta-cam/src/toolpath.rs`

Implement `compute_lead_in(entry: &EntryPoint, lead_in_radius: f64, cut_depth: f64, plunge_rate: f64) -> Vec<ToolpathMove>`:

The lead-in is a quarter-circle arc from an approach point to the contour entry point:
1. The approach point is offset from the entry point along the outward normal by `lead_in_radius`.
2. The arc center is at the entry point offset by `lead_in_radius` perpendicular to the edge direction (on the outward side).
3. The arc starts at the approach point (at safe-Z or clearance height) and ends at the entry point on the contour.
4. During the arc, Z ramps from clearance down to `-cut_depth` (helical lead-in), using `plunge_rate`.
5. The arc direction (G2/G3, clockwise) depends on the contour winding and approach geometry.

Lead-out is the mirror: arc from contour exit back to an approach point, ramping Z up, then retract.

Output moves:
```
Rapid { to: approach_point at safe_z }
ArcCut { to: entry_point at -cut_depth, center, clockwise, feed: plunge_rate }
```

#### Task 2.5: Contour Walking with Tab Awareness

**File:** `crates/thetta-cam/src/toolpath.rs`

Implement `plan_profile_toolpath(offset_contour: &Polygon, projected_tabs: &[ProjectedTab], tool: &Tool, gcode_config: &GcodeConfig, profile_config: &ProfileCutConfig) -> Toolpath`:

Algorithm:

**Step 1: Validate inputs.**
- Assert contour has >= 3 vertices.
- Validate CCW winding (signed area > 0). If CW, reverse the contour.
- If no vertices, return empty toolpath.

**Step 2: Build sorted tab intervals along contour.**

Convert each projected tab to an arc-distance interval `(start_dist, end_dist)` along the offset contour:
```
contour_length = perimeter(offset_contour)
for tab in projected_tabs:
    center_dist = arc_position_of(tab, offset_contour)
    half_width = tab.width / 2.0
    start = (center_dist - half_width).max(0.0)
    end = (center_dist + half_width).min(contour_length)
    tab_intervals.push((start, end, tab.height))
sort tab_intervals by start_dist
```

Handle wrap-around: if a tab straddles the contour start (distance near 0 or near perimeter), split into two intervals.

**Step 3: Select entry point and compute lead-in.**

Call `select_entry_point()`. The contour walk starts from this entry point, not from `contour.outer[0]`.

**Step 4: Walk contour edges from entry point.**

Starting from the entry edge at parameter `t`, walk edges in contour order (wrapping around). For each position along the contour:
- Compute arc distance from entry point.
- Check if inside any tab interval.
- Outside tab: emit `LinearCut { to: edge_endpoint at -cut_depth, feed: tool.feed_rate }`.
- Entering tab: emit `LinearCut` ramping Z from `-cut_depth` to `-(cut_depth - tab.height)`, clamped so Z never goes above 0.0.
- Inside tab: emit `LinearCut` at `-(cut_depth - tab.height)`.
- Exiting tab: emit `LinearCut` ramping Z from `-(cut_depth - tab.height)` back to `-cut_depth`.

The ramp distance for tab entry/exit is `min(2.0mm, remaining_edge_length)`.

**Step 5: Close contour and lead-out.**

After walking back to the entry point, emit the lead-out arc (mirror of lead-in) and retract to safe-Z.

**Complete move sequence for one piece:**
```
Rapid { to: approach_point at safe_z }
ArcCut { to: entry_point at -cut_depth, center, clockwise, feed: plunge_rate }  // lead-in
[LinearCut moves along offset contour with tab lifts]
ArcCut { to: departure_point, center, clockwise, feed: feed_rate }  // lead-out
Retract { to_z: safe_z }
```

#### Task 2.6: Unit Tests for Toolpath Planner

**File:** `crates/thetta-cam/src/toolpath.rs` (inline tests) and `crates/thetta-cam/tests/toolpath_tests.rs`

| Test | Input | Expected |
|------|-------|----------|
| `test_simple_rectangle_no_tabs` | 100x50mm rectangle, tool 6mm, 18mm depth | Offset contour is 103x53mm (3mm outward). Moves include lead-in arc, 4 linear cuts, lead-out arc, retract. |
| `test_rectangle_with_one_tab` | 100x50mm rect, 1 tab on edge 0 at t=0.5, 7mm height | At projected tab location: Z lifts from -18 to -11 (18-7), then back to -18. |
| `test_tab_z_clamped_above_zero` | Tab height > cut depth | Z at tab clamped to 0.0 (never above surface). |
| `test_no_contour` | Empty polygon | Empty toolpath. |
| `test_start_and_end_at_safe_z` | Any valid input | First move is Rapid to safe_z, last is Retract to safe_z. |
| `test_multiple_tabs_on_one_edge` | Long edge with 2 tabs | Two Z lifts on that edge, in correct order. |
| `test_polygon_offset_rectangle` | 100x50mm rect, 3mm radius | Offset result is 106x56mm (outer ring). |
| `test_tab_projection` | Tab at (50, 0) on original, offset by 3mm | Projected tab near (50, -3) on offset contour. |
| `test_entry_point_away_from_tabs` | Rectangle with tabs on all 4 edges | Entry point is on longest tab-free edge. |
| `test_lead_in_arc_geometry` | Known entry point and normal | Arc start is offset outward by lead_in_radius, arc is tangent to edge. |
| `test_ccw_winding_enforced` | CW polygon | Either reversed automatically or error returned. |
| `test_holes_ignored` | Polygon with holes | Only outer ring used for toolpath. |
| `test_zero_tool_radius` | Tool diameter = 0 | Validation error (tool diameter must be positive). |
| `test_deterministic_output` | Same input twice | Identical toolpath output. |

### Phase 3: G-code Emitter (~15%)

**Goal:** Convert toolpath moves into LinuxCNC-compatible G-code text.

#### Task 3.1: G-code Emitter

**File:** `crates/thetta-cam/src/gcode.rs`

```rust
/// Emit a complete G-code program for one sheet.
pub fn emit_gcode(
    toolpaths: &[Toolpath],
    tool: &Tool,
    config: &GcodeConfig,
    sheet_index: usize,
) -> String
```

Output structure per sheet:
```gcode
%
(Sheet 1 - Generated by Thetta)
(Tool: T1 6mm 2-flute upcut)
G21 (Metric)
G90 (Absolute positioning)
G54 (Work coordinate system)
T1 M6 (Tool change)
S18000 M3 (Spindle on CW)
G0 Z5.000 (Safe Z)

(Piece 1)
G0 X47.000 Y-3.000 (Rapid to approach)
G0 Z2.000 (Lower to clearance)
G2 X50.000 Y0.000 I3.000 J0.000 Z-18.000 F500 (Arc lead-in with helical plunge)
G1 X103.000 Y0.000 F1500 (Profile cut)
G1 X103.000 Y53.000
... (contour with tab lifts)
G2 X47.000 Y-3.000 I-3.000 J0.000 F1500 (Arc lead-out)
G0 Z5.000 (Retract)

(Piece 2)
G0 X200.000 Y30.000
...

M5 (Spindle off)
G0 Z5.000 (Retract to safe Z)
G0 X0.000 Y0.000 (Return to origin)
M2 (Program end)
%
```

Formatting rules:
- Coordinates formatted to 3 decimal places, locale-independent (always `.` decimal separator).
- No NaN or Inf values in output. Validate all coordinates before formatting; if any are non-finite, skip the move and log a warning.
- Feed rates as integers (F1500, not F1500.000).
- Comments in parentheses on the same line as the command. Tool names and comments must be ASCII-only; strip non-ASCII characters.
- Blank line between pieces for readability.
- G0 for rapids (no feed rate), G1 for linear cuts (with feed rate), G2/G3 for arcs.
- Modal optimization: only emit G0/G1/G2/G3 when motion mode changes, only emit F when feed rate changes.
- G17 (XY plane) set in preamble for arc interpretation.

#### Task 3.2: ToolpathMove to G-code Mapping

| ToolpathMove | G-code | Notes |
|---|---|---|
| `Rapid { to }` | `G0 X{to.x} Y{to.y} Z{to.z}` | Omit unchanged axes |
| `LinearCut { to, feed }` | `G1 X{to.x} Y{to.y} Z{to.z} F{feed}` | F only when changed |
| `ArcCut { to, center, clockwise, feed }` | `G2/G3 X{to.x} Y{to.y} Z{to.z} I{cx} J{cy} F{feed}` | G2=CW, G3=CCW. I/J are center offsets from current position |
| `TabLift { to, tab_height }` | `G1 X{to.x} Y{to.y} Z{-(depth-tab_height)}` | Z clamped to 0.0 |
| `Retract { to_z }` | `G0 Z{to_z}` | Always rapid |
| `Plunge { to_z, feed }` | `G1 Z{to_z} F{feed}` | Vertical plunge |

#### Task 3.3: Unit Tests for G-code Emitter

| Test | Input | Expected |
|------|-------|----------|
| `test_preamble_format` | Any valid input | Output contains `%`, `G21`, `G90`, `G54`, `T{n} M6`, `S{rpm} M3` |
| `test_postamble_format` | Any valid input | Output ends with `M5`, `G0 Z{safe}`, `G0 X0.000 Y0.000`, `M2`, `%` |
| `test_coordinate_precision` | Point at (1.0/3.0, 2.0/3.0, ...) | Coordinates have exactly 3 decimal places |
| `test_feed_rate_format` | Feed 1500.0 | Emitted as `F1500` (integer, no decimals) |
| `test_empty_toolpaths` | Empty slice | Valid G-code with preamble + postamble only |
| `test_modal_gcode` | Consecutive LinearCut moves | `G1` emitted once, subsequent cuts omit `G1` |
| `test_arc_move_format` | ArcCut clockwise | `G2 X... Y... I... J...` with correct center offsets |
| `test_no_nan_in_output` | Toolpath with normal values | Output contains no "NaN" or "Inf" strings |
| `test_z_never_positive_during_cut` | Any profile | No G1 Z value > 0.0 |
| `test_tool_change` | Tool T3 | Output contains `T3 M6` |
| `test_ascii_comments` | Tool name with unicode | Comments contain only ASCII |

### Phase 4: App Integration + UI (~25%)

**Goal:** Wire tool database, G-code config, and export trigger into the app. Build Tool DB panel and G-code config panel.

#### Task 4.1: Sheet G-code Orchestrator

**File:** `crates/thetta-cam/src/generate.rs`

```rust
use std::collections::HashMap;
use thetta_geometry::types::*;
use thetta_geometry::polygon_ops;

/// Input for generating G-code for one sheet.
pub struct SheetGcodeInput<'a> {
    pub sheet_index: usize,
    pub placements: &'a [Placement],
    pub pieces: &'a [Piece],
    pub tabs: &'a HashMap<(usize, usize), Vec<Tab>>,
    pub tool: &'a Tool,
    pub gcode_config: &'a GcodeConfig,
    pub profile_config: &'a ProfileCutConfig,
}

/// Generate G-code for all pieces on a single sheet.
pub fn generate_sheet_gcode(input: &SheetGcodeInput) -> String {
    let mut toolpaths = Vec::new();

    for (placement_idx, placement) in input.placements.iter().enumerate() {
        let Some(piece) = input.pieces.iter().find(|p| p.id == placement.piece_id) else {
            continue;
        };

        // Apply placement transform to get world-space contour
        let center = Point2D { x: 0.0, y: 0.0 };
        let rotated = polygon_ops::rotate(&piece.polygon, placement.rotation, center);
        let placed = polygon_ops::translate(&rotated, placement.x, placement.y);

        // Get tabs for this placement (may be empty)
        let empty_tabs = Vec::new();
        let tabs = input.tabs
            .get(&(input.sheet_index, placement_idx))
            .unwrap_or(&empty_tabs);

        let toolpath = crate::toolpath::plan_profile_toolpath(
            &placed, tabs, input.tool, input.gcode_config, input.profile_config,
        );
        toolpaths.push(toolpath);
    }

    crate::gcode::emit_gcode(&toolpaths, input.tool, input.gcode_config, input.sheet_index)
}

/// Generate G-code for all sheets, returning one String per sheet.
/// Skips empty sheets.
pub fn generate_all_sheets(
    result: &NestingResult,
    pieces: &[Piece],
    tabs: &HashMap<(usize, usize), Vec<Tab>>,
    tool: &Tool,
    gcode_config: &GcodeConfig,
    profile_config: &ProfileCutConfig,
) -> Vec<(usize, String)> {
    result.sheets.iter().enumerate()
        .filter(|(_, sheet)| !sheet.placements.is_empty())
        .map(|(sheet_idx, sheet)| {
            let input = SheetGcodeInput {
                sheet_index: sheet_idx,
                placements: &sheet.placements,
                pieces,
                tabs,
                tool,
                gcode_config,
                profile_config,
            };
            (sheet_idx, generate_sheet_gcode(&input))
        })
        .collect()
}
```

#### Task 4.2: App State Additions

**File:** `crates/thetta-app/src/state.rs`

Add to `AppState`:
```rust
pub tool_table: ToolTable,
pub gcode_config: GcodeConfig,
pub profile_config: ProfileCutConfig,
gcode_rx: Option<Receiver<Result<Vec<PathBuf>, String>>>,
gcode_status: Option<String>,
```

In `AppState::new()`, load tool table from disk:
```rust
tool_table: thetta_cam::tool_db::load_tool_table(),
gcode_config: GcodeConfig::default(),
profile_config: ProfileCutConfig::default(),
```

Add G-code generation method:
```rust
impl AppState {
    pub fn start_gcode_export(&mut self, output_dir: PathBuf) {
        // Validate config
        let Some(tool_num) = self.gcode_config.tool_number else { return };
        let Some(tool) = self.tool_table.by_number(tool_num).cloned() else { return };
        let Some(result) = self.current_result.clone() else { return };

        let pieces = self.pieces.clone();
        let tabs = self.tab_state.results.clone();
        let gcode_config = self.gcode_config.clone();
        let profile_config = self.profile_config.clone();
        let (tx, rx) = mpsc::channel();
        self.gcode_rx = Some(rx);

        thread::spawn(move || {
            let sheets = thetta_cam::generate::generate_all_sheets(
                &result, &pieces, &tabs, &tool, &gcode_config, &profile_config,
            );
            let mut paths = Vec::new();
            for (sheet_idx, gcode) in &sheets {
                let filename = format!("sheet_{}.ngc", sheet_idx + 1);
                let path = output_dir.join(&filename);
                if let Err(e) = std::fs::write(&path, gcode) {
                    let _ = tx.send(Err(format!("Failed to write {}: {}", filename, e)));
                    return;
                }
                paths.push(path);
            }
            let _ = tx.send(Ok(paths));
        });
    }

    pub fn poll_gcode_export(&mut self) {
        if let Some(rx) = &self.gcode_rx {
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok(paths) => {
                        self.gcode_status = Some(
                            format!("Exported {} G-code file(s)", paths.len())
                        );
                    }
                    Err(e) => {
                        self.gcode_status = Some(format!("G-code error: {}", e));
                    }
                }
                self.gcode_rx = None;
            }
        }
    }
}
```

#### Task 4.3: UI Panels

**File:** `crates/thetta-app/src/panels.rs`

Add two new UI sections after the Tabs panel:

**Tool Database section:**
- List of tools (table: number, name, diameter, feed, RPM)
- "Add Tool" button -> inline form with fields for all `Tool` properties
- Edit button per tool row -> same form, pre-filled
- Delete button per tool row (with confirmation)
- Changes auto-save via `save_tool_table()`

**G-code section:**
- `tool_number` dropdown populated from `ToolTable.tools`
- `safe_z` slider (1.0-20.0mm, default 5.0)
- `cut_depth` slider (1.0-50.0mm, default 18.0)
- `lead_in_radius` slider (1.0-10.0mm, default 3.0)
- "Generate G-code" button (disabled with message if: no tool selected, no nesting result, no pieces)
- Status line showing export result or error

Return `PanelAction::GenerateGcode` when "Generate G-code" clicked. Return `PanelAction::SaveToolTable` when tool database changes.

#### Task 4.4: Main Loop Integration

**File:** `crates/thetta-app/src/main.rs`

Handle new `PanelAction` variants:
- `GenerateGcode`: open native save-directory dialog, then call `app_state.start_gcode_export(dir)`
- `SaveToolTable`: call `thetta_cam::tool_db::save_tool_table(&app_state.tool_table)`

Add `app_state.poll_gcode_export()` to the frame update loop.

#### Task 4.5: Add `thetta-cam` Dependency

**File:** `crates/thetta-app/Cargo.toml`

```toml
thetta-cam = { path = "../thetta-cam" }
```

#### Task 4.6: Integration and End-to-End Tests

**File:** `crates/thetta-cam/tests/integration_tests.rs`

| Test | Scenario | Verification |
|------|----------|-------------|
| `test_full_rectangle_profile` | 100x50mm rect, tool 6mm, 18mm depth, no tabs | G-code contains lead-in arc, 4 profile cuts on offset contour, lead-out arc, retract. |
| `test_rectangle_with_tabs` | 100x50mm rect, 2 tabs on long edges | Tab positions produce Z lifts in G-code output. |
| `test_multi_piece_sheet` | 3 pieces on one sheet | 3 independent tool approach/retract sequences, rapids between each. |
| `test_empty_sheet` | Sheet with no placements | No output file / empty result. |
| `test_gcode_parseable` | Any generated G-code | Every non-comment line matches expected G-code patterns. |
| `test_z_safety` | Multi-piece sheet | Z is at safe_z between all piece operations. Never at cut depth during rapid traverse. |
| `test_deterministic` | Same input twice | Identical G-code output byte-for-byte. |
| `test_offset_contour_dimensions` | 100x50mm rect, 3mm tool radius | Profile cuts trace 106x56mm offset contour. |

## Files Summary

### New Files

| File | Purpose | Est. LOC |
|------|---------|----------|
| `crates/thetta-cam/Cargo.toml` | Crate manifest | 15 |
| `crates/thetta-cam/src/lib.rs` | Module declarations | 10 |
| `crates/thetta-cam/src/toolpath.rs` | Profile toolpath planner: offset, tab projection, entry point, lead-in/out, contour walk | 450 |
| `crates/thetta-cam/src/gcode.rs` | G-code emitter: LinuxCNC dialect, modal output, formatting | 200 |
| `crates/thetta-cam/src/generate.rs` | Per-sheet orchestration | 100 |
| `crates/thetta-cam/src/tool_db.rs` | Tool database JSON persistence with atomic writes | 50 |
| `crates/thetta-cam/tests/toolpath_tests.rs` | Toolpath planner unit/integration tests | 200 |
| `crates/thetta-cam/tests/gcode_tests.rs` | G-code output tests | 120 |
| `crates/thetta-cam/tests/integration_tests.rs` | End-to-end sheet generation tests | 150 |
| **Total new** | | **~1295** |

### Modified Files

| File | Changes | Est. Delta |
|------|---------|------------|
| `Cargo.toml` (root) | Add `crates/thetta-cam` to workspace members | +1 |
| `crates/thetta-geometry/src/types.rs` | Add `Tool`, `ToolTable`, `GcodeConfig`, `ProfileCutConfig`, `CutDirection`, `ToolpathMove`, `Point3D`, `Toolpath` | +120 |
| `crates/thetta-app/Cargo.toml` | Add `thetta-cam` dependency | +1 |
| `crates/thetta-app/src/state.rs` | Add `tool_table`, `gcode_config`, `profile_config`, G-code export lifecycle | +70 |
| `crates/thetta-app/src/panels.rs` | Add Tool Database and G-code config UI sections | +160 |
| `crates/thetta-app/src/main.rs` | Handle new `PanelAction` variants, poll G-code export | +25 |
| **Total modified** | | **~377** |

### Estimated Total: ~1672 LOC

## Definition of Done

### P0 -- Must Ship

- [ ] `Tool` type with fields: `number`, `name`, `diameter`, `flute_count`, `feed_rate`, `plunge_rate`, `spindle_speed` in `thetta-geometry/src/types.rs`
- [ ] `ToolTable` with `tools: Vec<Tool>`, `by_number()`, `next_number()`, `validate()` methods
- [ ] `GcodeConfig` with `safe_z`, `cut_depth`, `tool_number`, `coordinate_system` fields and defaults
- [ ] `ProfileCutConfig` with `lead_in_radius`, `cutting_direction` fields and defaults
- [ ] `ToolpathMove` enum with `Rapid`, `LinearCut`, `ArcCut`, `TabLift`, `Retract`, `Plunge` variants
- [ ] `Point3D` struct for 3D toolpath positions
- [ ] `Toolpath` struct with `moves: Vec<ToolpathMove>`, `config: GcodeConfig`
- [ ] `thetta-cam` crate exists in workspace with `thetta-geometry` dependency
- [ ] `thetta-cam` added to workspace members in root `Cargo.toml`
- [ ] `thetta-app` depends on `thetta-cam`
- [ ] Polygon offset: `offset_contour()` offsets outer ring outward by tool radius using `geo-buffer` or proper parallel offset
- [ ] Tab projection: `project_tabs()` maps original tab positions to nearest point on offset contour
- [ ] Entry point selection: furthest from part center, on straight edge, away from tabs
- [ ] Tangential arc lead-in: G2/G3 arc from approach point to contour entry, with helical plunge to cut depth
- [ ] Tangential arc lead-out: mirror arc from contour exit to departure point
- [ ] Profile contour walking: edge-by-edge `LinearCut` moves along offset contour at `-cut_depth`
- [ ] Tab-aware height changes: Z ramps to `-(cut_depth - tab_height)` at tab positions, clamped to 0.0
- [ ] Tab ramp distance: `min(2.0mm, remaining_edge_length)` for entry/exit transitions
- [ ] CCW winding validated as precondition (consistent with `tabs.rs` approach)
- [ ] Hole contours (`Polygon.holes`) explicitly skipped -- only outer ring used
- [ ] G-code emitter produces LinuxCNC-compatible output with `%`, `G21`, `G90`, `G54`, `G17` preamble
- [ ] G-code includes `T{n} M6` tool change and `S{rpm} M3` spindle start
- [ ] G-code includes `M5`, safe-Z retract, `G0 X0 Y0`, `M2`, `%` postamble
- [ ] G-code coordinates formatted to 3 decimal places, locale-independent (`.` separator)
- [ ] G-code feed rates formatted as integers
- [ ] No NaN or Inf values in G-code output
- [ ] G-code comments are ASCII-only
- [ ] G-code Z values never positive during G1 cut moves
- [ ] Modal G-code: G0/G1/G2/G3 and F only emitted when mode or feed changes
- [ ] `generate_sheet_gcode()` combines multiple piece toolpaths with safe-Z rapids between pieces
- [ ] `generate_all_sheets()` produces one G-code string per non-empty sheet
- [ ] Tool database saves to and loads from JSON file on disk (atomic write)
- [ ] Tool database handles missing/corrupt file gracefully (returns empty default)
- [ ] `ToolTable::validate()` catches duplicate tool numbers and invalid parameters
- [ ] UI: Tool Database panel with add/edit/delete tool functionality
- [ ] UI: G-code config panel with safe-Z, cut depth, lead-in radius, tool selection
- [ ] UI: "Generate G-code" button triggers background export (disabled when preconditions not met)
- [ ] UI: G-code export status shown after completion or error
- [ ] One `.ngc` file written per non-empty sheet to user-selected directory
- [ ] Unit tests for: toolpath planner (offset, tabs, lead-in, contour walk), G-code emitter (format, preamble, postamble), tool DB (roundtrip, corrupt file)
- [ ] `cargo test --workspace` passes with no regressions
- [ ] `cargo clippy --workspace` reports no new warnings

### P1 -- Should Ship

- [ ] Multi-pass step-down: when `step_down < cut_depth`, generate multiple passes at increasing depths with per-pass tab handling
- [ ] Cutting direction option: `Climb` vs `Conventional` (reverse contour walk for climb)
- [ ] G-code preview/visualization in viewport: toolpath lines overlay showing rapid (dashed) and cut (solid) moves
- [ ] G-code file naming includes descriptive info: `sheet_1_T1_6mm.ngc`
- [ ] Feed rate changes correctly between plunge (`plunge_rate`) and cut (`feed_rate`) moves with modal tracking
- [ ] Integration test: rectangle with known tabs produces G-code with verifiable Z heights at tab positions

### P2 -- Stretch

- [ ] LinuxCNC `.tbl` tool table export
- [ ] G-code dry-run validator: check for out-of-bounds moves (exceeding sheet dimensions)
- [ ] Feed rate override per operation
- [ ] Estimated machining time displayed in UI (total path length / feed rate)
- [ ] Tool library presets (common endmill sizes with typical feeds/speeds)
- [ ] G-code output preview in a scrollable text widget

### Cut if Behind Schedule

- Tangential arc lead-in/lead-out -> direct plunge (simpler but leaves marks)
- Tab projection onto offset contour -> use original tab positions directly (slightly inaccurate)
- Entry point selection -> use `contour.outer[0]` (suboptimal but functional)
- Tool database persistence -> tools stored only in memory, lost on restart
- Modal G-code optimization -> always emit G0/G1/F on every line

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Polygon offset fails on concave polygons (self-intersection, degenerate result) | Medium | High | Use `geo-buffer` crate (Clipper-style offset). If multi-polygon results, take largest by area. Test with L-shapes and concave fixtures. |
| Tab projection from original to offset contour misplaces tabs for large tool radii or tight concavities | Medium | Medium | Nearest-point projection with sanity check: warn if projection distance exceeds `tool_radius * 1.5`. Works well for typical tool sizes (3-6mm radius on features > 20mm). |
| Arc lead-in geometry incorrect (tangent point computation, arc direction) | Medium | Medium | Perpendicular offset from entry edge for approach point. Test with known geometry. G-code simulator (CAMotics) verification. |
| Tab Z-height calculation incorrect, tool cuts through tabs | Medium | High | Unit test with known tab positions verifying exact Z values at tab entry, center, and exit. Visual verification in CAMotics. |
| Contour winding direction wrong, tool cuts on wrong side | Medium | High | Assert CCW outer winding (same guard as `tabs.rs`). Conventional milling follows CCW naturally. |
| Tab interval wraps around contour start/entry point | Medium | Medium | Handle wrap-around: split tab interval at contour seam. Test with tab placed near entry point. |
| Float formatting locale issues (comma decimal separator) | Low | High | Use `format!("{:.3}", val)` which is locale-independent in Rust. Add test verifying `.` separator. |
| `dirs::config_dir()` returns None on unusual system | Low | Low | Fallback: log warning and keep tool table in memory only. User can still use the app. |
| G-code dialect incompatible with specific LinuxCNC version | Low | Medium | Target LinuxCNC 2.8+. Use only standard G-codes (G0, G1, G2, G3, G17, G21, G54, G90, T, M2, M3, M5, M6). |
| Very short edges (< tool diameter) cause offset contour to collapse | Low | Medium | `geo-buffer` handles this; collapsed features are removed from offset result. Test with small polygon features. |

## Security Considerations

- **No network access.** G-code generation is pure local computation. Tool database is a local file.
- **File writes limited to user-selected directory.** G-code export uses native save dialog. No writes outside user-chosen path except tool database in config dir.
- **No unsafe code.** All new code is safe Rust. No FFI.
- **New dependencies are minimal and well-audited.** `dirs` (platform paths), `serde_json` (JSON), `geo-buffer`/`geo` (geometry) are widely used crates.
- **Input validation.** Tool parameters validated via `ToolTable::validate()`. `GcodeConfig` values validated before export. Invalid configs produce clear error messages rather than panicking.
- **G-code output safety.** No NaN/Inf in output. Coordinates are fixed-precision. Z values validated to never be positive during cut moves. Safe-Z rapids between all piece operations.
- **No arbitrary file path construction.** Output filenames are hardcoded patterns (`sheet_N.ngc`), not user-controlled strings.
- **Tool database corruption.** Atomic writes (temp file + rename) prevent partial writes. Corrupt JSON handled by returning empty default.
- **Deterministic output.** Same inputs always produce identical G-code. No HashMap iteration for output-affecting logic.

## Dependencies

### New External Dependencies

| Crate | Version | Purpose | Justification |
|-------|---------|---------|---------------|
| `dirs` | 5.x | Platform config directory path | Needed for tool database file location. Small, widely used. |
| `serde_json` | 1.x | Tool table JSON serialization | Already a transitive dependency in the workspace. |
| `geo-buffer` | 0.3.x | Proper parallel polygon offset | Already in workspace dependencies. Required for tool radius compensation. |
| `geo` | 0.29.x | Geometry types for `geo-buffer` interop | Transitive dependency of `geo-buffer`. |

### New Internal Dependencies

| Edge | Purpose |
|------|---------|
| `thetta-cam` -> `thetta-geometry` | Access to `Polygon`, `Tab`, `Tool`, `Toolpath`, `Point2D`, etc. |
| `thetta-app` -> `thetta-cam` | G-code generation, tool database I/O |

### Updated Dependency Graph

```
                   +----------------------+
                   |   External Systems   |
                   |----------------------|
                   | OpenCASCADE runtime  |
                   +----------+-----------+
                              ^
                              | [feature = "occt"]
                              |
+----------------+    +-------+--------+    +-------------------+
| thetta-geometry|    |  thetta-step   |    |     jagua-rs      |
| (leaf crate)   |    |  (STEP I/O)    |    | (vendored fork)   |
+---+----+-------+    +-------+--------+    +---------+---------+
    ^    ^                    ^                       ^
    |    |                    |                       |
    |    |         +----------+-----------+-----------+
    |    |         |                      |
    |    |  +------+--------+             |
    |    |  | thetta-nesting|             |
    |    |  | (optimizer)   |             |
    |    |  +------+--------+             |
    |    |         ^                      |
    |    |         |                      |
    |  +-+---------+---------+            |
    |  |    thetta-app       |------------+
    |  |   (eframe UI)       |
    |  +----+----------------+
    |       |
    +---+---+
        |
   +----+----------+
   |  thetta-cam    |
   |  (G-code CAM)  |
   +----------------+
```

Note: `thetta-cam` depends only on `thetta-geometry`. It has no dependency on `jagua-rs`, `thetta-nesting`, or `thetta-step`.

## Open Questions (Resolved)

1. **Should G-code generation live in `thetta-step` or a new `thetta-cam` crate?** -> New `thetta-cam` crate. CAM logic is distinct from STEP I/O.
2. **Should there be an intermediate `Toolpath` representation?** -> Yes. Decouples planning from emission. Enables unit testing without parsing G-code.
3. **How should lead-in/lead-out be handled?** -> Tangential arc entry (G2/G3) as P0. Prevents tool marks at entry point.
4. **Should step-down be supported in this sprint?** -> P1. Single full-depth pass for P0.
5. **Tool table persistence format?** -> JSON in `~/.config/thetta/tools.json` via `dirs` crate.
6. **G-code viewer/preview?** -> External only for P0 (CAMotics, ncviewer). In-app preview is P1.
7. **Cutting direction?** -> Conventional (CCW) as default. Climb option is P1.
8. **Tool radius compensation approach?** -> Software polygon offset (not G41/G42). Uses `geo-buffer` crate.
9. **Tab position mapping after offset?** -> Nearest-point projection from original tab position onto offset contour.
10. **Coordinate system?** -> Nesting origin = bottom-left of sheet = G54 work origin. Z=0 at stock surface.
