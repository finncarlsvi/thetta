# Sprint 007: G-code Profile Cuts with Tool Database

## Overview

Implement G-code generation for profile cuts with tab-aware toolpaths and a persistent tool database, targeting LinuxCNC CNC routers. This is the first CAM sprint: after nesting and tab placement, the user selects a cutting tool, configures machining parameters, and exports one G-code file per sheet. The generated G-code cuts along each piece's outer contour at full depth (with optional multi-pass step-down), lifting the tool over holding tabs placed by Sprint 006.

The implementation introduces a new `thetta-cam` crate that owns the toolpath intermediate representation and G-code emitter. Separating CAM from STEP I/O is deliberate: G-code generation has no dependency on OpenCASCADE, uses fundamentally different data structures (linear move sequences vs. solid geometry), and will grow to include hole drilling and pocket clearing in future sprints. The `thetta-cam` crate depends only on `thetta-geometry` for `Polygon`, `Placement`, `Tab`, and `Point2D` types.

The tool database stores tool definitions (number, diameter, flute count, feed/plunge rates, spindle speed) in a JSON file that persists across sessions. The UI provides tool CRUD, tool assignment to the profile operation, and a G-code configuration panel (safe-Z, cut depth, step-down increment). G-code generation runs synchronously on a background thread and produces one `.ngc` file per sheet, following LinuxCNC conventions: `G21` (metric), `G90` (absolute), `T/M6` tool changes, `G17` (XY plane), and proper preamble/postamble.

**Scope:** `Tool`, `ToolTable`, `GcodeConfig`, `Toolpath`, `ToolpathSegment` types in `thetta-geometry`. `thetta-cam` crate with toolpath builder and G-code emitter. Tool database UI, G-code config panel, and export trigger in `thetta-app`. One G-code file per sheet. Profile cuts only (no holes/pockets).
**Prerequisites:** Sprint 006 complete. `NestingResult` with placed pieces. `TabState` with `HashMap<(usize, usize), Vec<Tab>>` per placement. Tab positions in world space. STEP/JSON export pipeline operational.
**Deferred:** Hole drilling, pocket clearing, lead-in/lead-out arcs, climb vs. conventional milling selection, in-app G-code preview/simulation, tool wear compensation, cutter radius compensation (G41/G42), arc moves (G2/G3) for curved contours.

## Use Cases

### UC1: Create and Manage Tools
User opens the Tool Database panel, clicks "Add Tool", and enters parameters: tool number (T1), diameter (6mm), flute count (2), feed rate (1500 mm/min), plunge rate (500 mm/min), spindle speed (18000 RPM). The tool appears in the tool list. User can edit or delete tools. The tool database saves to disk automatically and loads on next app launch.

### UC2: Configure G-code Parameters
User sets machining parameters in the G-code panel: cut depth (18mm, matching material thickness), step-down per pass (6mm, so 3 passes), safe-Z height (5mm for rapid traverses above material). These parameters combine with the selected tool's feed/speed to fully define the machining operation.

### UC3: Assign Tool to Profile Operation
User selects a tool from the database for profile cuts. The tool's diameter determines the offset (cutter compensation is deferred — the toolpath follows the contour centerline in this sprint, so the user is expected to account for tool diameter in their nesting separation setting).

### UC4: Generate G-code for Single Sheet
User clicks "Generate G-code", selects an output directory. The app generates one `.ngc` file per sheet. The file contains a LinuxCNC-compatible program: tool change, spindle on, then for each piece on the sheet, the profile cut toolpath with tab-aware Z heights. Rapid moves between pieces at safe-Z.

### UC5: Tab-Aware Profile Cuts
At each tab position along a piece's contour, the tool ramps up from full cut depth to `cut_depth - tab_height` (leaving the tab uncut), traverses the tab width at the shallower depth, then ramps back down to full depth. Between tabs, the tool cuts at full depth.

### UC6: Multi-Pass Step-Down
When material is thicker than the tool can cut in one pass (step-down < cut depth), the toolpath makes multiple passes at increasing depths. Each pass follows the full contour with tab awareness. The final pass reaches full cut depth.

### UC7: Multi-Sheet G-code Export
For multi-sheet nesting results, each sheet produces an independent G-code file (`output_sheet_1.ngc`, `output_sheet_2.ngc`, etc.). Each file is self-contained with its own preamble, tool change, and postamble.

### UC8: Empty Sheet Handling
Sheets with no placements produce no G-code file. Sheets where all pieces have no tabs still generate valid G-code (full-depth profile cuts with no tab lifts).

### UC9: Tool Database Persistence
Tool database saves to `~/.config/thetta/tools.json` (or platform equivalent via `dirs` crate). On first launch with no tool file, the database starts empty. Corrupt tool files produce a warning and start fresh.

## Architecture

### Component Overview

```
+------------------------------------------------------------------+
|                          thetta-app                                |
|  +-----------+  +----------+  +--------------------------------+ |
|  | panels.rs |  |viewport.rs| |          state.rs              | |
|  | Tool DB   |  | (unchanged|  | ToolTable (loaded from disk)  | |
|  | G-code    |  |  this     |  | GcodeConfig                   | |
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
         |                       |  | build_profile_toolpath()   |  |
         |                       |  | Contour → Toolpath with    |  |
         |                       |  | tab-aware Z heights        |  |
         |                       |  +---------------------------+  |
         |                       |  +---------------------------+  |
         |                       |  |    gcode.rs                |  |
         |                       |  | emit_gcode()               |  |
         |                       |  | Toolpath → G-code text     |  |
         |                       |  | LinuxCNC dialect            |  |
         |                       |  +---------------------------+  |
         |                       |  +---------------------------+  |
         |                       |  |    generate.rs             |  |
         |                       |  | generate_sheet_gcode()     |  |
         |                       |  | Orchestrates per-sheet     |  |
         |                       |  | toolpath + emission         |  |
         |                       |  +---------------------------+  |
         |                       +-----------------+---------------+
         |                                         |
         |                       +-----------------v---------------+
         |                       |        thetta-geometry           |
         +-------------------+-->|  types.rs                        |
                                 |  + Tool, ToolTable, GcodeConfig  |
                                 |  + Toolpath, ToolpathSegment     |
                                 |  + (existing) Polygon, Tab, etc  |
                                 +---------------------------------+
```

### New Crate: `thetta-cam`

A new workspace crate at `crates/thetta-cam/` with dependencies on `thetta-geometry` only. This crate has no UI, no OCCT, no nesting logic. Its sole job is: given placed piece contours, tabs, tool parameters, and G-code config, produce G-code text.

Rationale for a separate crate rather than extending `thetta-step`:
1. **No OCCT dependency.** G-code is plain text output. Putting it in `thetta-step` would mean the CAM code lives alongside OCCT FFI, creating confusion about what requires the `occt` feature.
2. **Different domain.** STEP I/O deals with 3D solid geometry. G-code deals with 2.5D toolpath motion. The data models and concerns are different.
3. **Growth path.** Future sprints add hole drilling, pocket clearing, and operation ordering. These are CAM concepts that belong together, not mixed into an I/O crate.
4. **Testability.** `thetta-cam` can be tested independently with simple polygon fixtures, no STEP files needed.

### Data Flow

```
NestingResult + TabState + ToolTable + GcodeConfig
         |
         v
For each Sheet (sheet_index):
  |
  +-- 1. Collect placements and their tabs
  |
  +-- 2. For each Placement:
  |       |
  |       +-- a. Look up Piece polygon
  |       +-- b. Apply placement transform → world-space contour
  |       +-- c. Look up tabs from TabState[(sheet_idx, placement_idx)]
  |       +-- d. Call build_profile_toolpath(contour, tabs, tool, config)
  |       |       |
  |       |       +-- Walk contour edges in order
  |       |       +-- At each tab: ramp to (cut_depth - tab_height), traverse tab width, ramp back
  |       |       +-- Between tabs: cut at full depth
  |       |       +-- For multi-pass: repeat contour at each depth level
  |       |       +-- Return Toolpath { segments: Vec<ToolpathSegment> }
  |       |
  |       +-- e. Append toolpath to sheet program
  |
  +-- 3. Call emit_gcode(sheet_toolpaths, tool, config) → String
  |       |
  |       +-- Preamble: G21 G90 G17, T/M6 tool change, M3 spindle on
  |       +-- For each piece toolpath:
  |       |     Rapid to start XY at safe-Z → plunge → profile moves → retract
  |       +-- Postamble: M5 spindle off, G0 Z safe, M2 program end
  |
  +-- 4. Write String to file: {output_dir}/sheet_{N}.ngc
```

### Intermediate Toolpath Representation

Rather than emitting G-code directly from contour geometry, the pipeline uses a `Toolpath` intermediate representation. This separation provides:
1. **Testability.** Toolpath generation can be unit-tested by inspecting segment sequences without parsing G-code text.
2. **Flexibility.** The same toolpath can be emitted to different G-code dialects or post-processors in the future.
3. **Clarity.** Tab-aware Z height logic lives in the toolpath builder, while G-code formatting lives in the emitter. Each module has a single concern.

### Threading Model

```
Main Thread (egui)                    Background Thread
+--------------------+               +---------------------------+
| - Click "Generate" |               | - Receives pieces, tabs,  |
| - Spawn thread     | ── inputs ──> |   tool, config            |
| - Poll receiver    |               | - Calls generate_gcode()  |
|   each frame       | <── mpsc ──   | - Sends Result<Vec<Path>> |
| - Show status      |               |   or error                |
+--------------------+               +---------------------------+
```

Same `mpsc::channel` pattern as STEP export. The generation is fast (text output), but runs on a background thread to keep the UI responsive and consistent with existing patterns.

### Design Decisions

1. **New `thetta-cam` crate.** G-code has no relationship to STEP I/O. Separate crate keeps concerns clean and avoids polluting `thetta-step` with unrelated CAM logic.

2. **Intermediate `Toolpath` representation.** Decouples toolpath planning from G-code text emission. Enables unit testing of tab-aware height changes without parsing G-code strings.

3. **No cutter radius compensation (G41/G42) in this sprint.** Cutter comp requires the toolpath to be offset from the contour by the tool radius. This is complex for irregular polygons with concave regions. Instead, the user accounts for tool diameter via the nesting separation parameter. Cutter comp is deferred.

4. **Direct plunge, no lead-in/lead-out.** Tangential arc entry/exit (common in production CAM) requires arc geometry and careful handling of tab proximity. Direct plunge is simpler and sufficient for wood/plastic CNC routing. Lead-in/lead-out deferred.

5. **Multi-pass step-down included.** Essential for CNC routing — cutting 18mm plywood in one pass with a 6mm endmill would break the tool. Step-down is a must-have, not a stretch goal.

6. **Conventional milling direction (CCW for outer contour, matching polygon winding).** The outer contour is stored CCW. Following this winding produces conventional milling. Climb milling (CW) would require reversing the contour. Conventional is safer for CNC routers and is the default.

7. **Tool table as JSON file with `dirs` crate for platform paths.** Simple, human-readable, and editable outside the app. The `dirs` crate provides `config_dir()` for cross-platform paths.

8. **One `.ngc` file per sheet.** Each file is self-contained. No multi-file coordination needed. Filenames include sheet index.

## Implementation

### Phase 1: Core Types in `thetta-geometry`

**Goal:** Add tool, toolpath, and G-code config types to the shared geometry crate.

#### Task 1.1: Tool and ToolTable types

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
    pub spindle_rpm: u32,
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
}
```

#### Task 1.2: GcodeConfig type

**File:** `crates/thetta-geometry/src/types.rs`

```rust
/// Parameters for G-code generation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GcodeConfig {
    /// Safe Z height for rapid moves above material (mm).
    pub safe_z: f64,
    /// Total cut depth into material (mm, positive value).
    pub cut_depth: f64,
    /// Maximum depth per pass (mm). If less than cut_depth, multiple passes are made.
    pub step_down: f64,
    /// Tool number assigned to profile cuts.
    pub profile_tool_number: Option<u32>,
}

impl Default for GcodeConfig {
    fn default() -> Self {
        Self {
            safe_z: 5.0,
            cut_depth: 18.0,
            step_down: 6.0,
            profile_tool_number: None,
        }
    }
}
```

#### Task 1.3: Toolpath types

**File:** `crates/thetta-geometry/src/types.rs`

```rust
/// A single motion segment in a toolpath.
#[derive(Debug, Clone, PartialEq)]
pub enum ToolpathSegment {
    /// Rapid move (G0) to position. Used for traverses above material.
    Rapid { x: f64, y: f64, z: f64 },
    /// Linear cut move (G1) to position at the given feed rate.
    Cut { x: f64, y: f64, z: f64, feed: f64 },
    /// Linear plunge move (G1) to Z depth at plunge feed rate.
    Plunge { z: f64, feed: f64 },
}

/// A complete toolpath for one piece profile or one operation.
#[derive(Debug, Clone, Default)]
pub struct Toolpath {
    pub segments: Vec<ToolpathSegment>,
}
```

### Phase 2: `thetta-cam` Crate — Toolpath Builder

**Goal:** Create the `thetta-cam` crate with the core toolpath generation logic.

#### Task 2.1: Crate scaffold

**New files:**
- `crates/thetta-cam/Cargo.toml`
- `crates/thetta-cam/src/lib.rs`
- `crates/thetta-cam/src/toolpath.rs`
- `crates/thetta-cam/src/gcode.rs`
- `crates/thetta-cam/src/generate.rs`

**`Cargo.toml`:**
```toml
[package]
name = "thetta-cam"
version = "0.1.0"
edition = "2021"

[dependencies]
thetta-geometry = { path = "../thetta-geometry" }
```

Add `"crates/thetta-cam"` to workspace members in root `Cargo.toml`.

**`lib.rs`:**
```rust
pub mod toolpath;
pub mod gcode;
pub mod generate;
```

#### Task 2.2: Profile toolpath builder

**File:** `crates/thetta-cam/src/toolpath.rs`

```rust
use thetta_geometry::polygon_ops::{edge_length, point_on_edge};
use thetta_geometry::types::*;

/// Build a tab-aware profile toolpath for a single placed piece.
///
/// The contour must be in world space (after placement transform).
/// Tabs must correspond to this contour's edges.
/// Returns a Toolpath with rapid approach, multi-pass profile cuts
/// with tab lifts, and retract.
pub fn build_profile_toolpath(
    contour: &Polygon,
    tabs: &[Tab],
    tool: &Tool,
    config: &GcodeConfig,
) -> Toolpath
```

Algorithm:

**Step 1: Compute depth passes.**
```
passes = []
remaining = config.cut_depth
current_depth = 0.0
while remaining > 0.0:
    step = min(config.step_down, remaining)
    current_depth += step
    passes.push(current_depth)
    remaining -= step
```

**Step 2: Build sorted tab intervals along contour.**

Convert each tab to an arc-distance interval `(start_dist, end_dist)` along the contour. Tab center is at `arc_position(tab)`. Tab extends `tab.width / 2` in each direction along the contour.

```
contour_length = perimeter(contour)
tab_intervals: Vec<(f64, f64, f64)> = []  // (start_dist, end_dist, tab_height)
for tab in tabs:
    center_dist = arc_position_of(tab, contour)
    half_width = tab.width / 2.0
    start = (center_dist - half_width).max(0.0)
    end = (center_dist + half_width).min(contour_length)
    tab_intervals.push((start, end, tab.height))
sort tab_intervals by start_dist
```

**Step 3: For each depth pass, walk contour edges with tab awareness.**

```
for pass_depth in passes:
    // Rapid to contour start point at safe_z
    segments.push(Rapid { x: contour.outer[0].x, y: contour.outer[0].y, z: config.safe_z })
    // Plunge to pass depth
    segments.push(Plunge { z: -pass_depth, feed: tool.plunge_rate })

    // Walk contour edges
    running_dist = 0.0
    for each edge (a → b) in contour.outer:
        edge_len = edge_length(a, b)

        // Check if any tab interval overlaps this edge segment
        // Split the edge at tab boundaries, adjusting Z:
        //   - Outside tab: z = -pass_depth
        //   - Inside tab: z = -(pass_depth - tab_height), clamped to 0.0
        // Emit Cut segments for each sub-segment

        edge_start_dist = running_dist
        edge_end_dist = running_dist + edge_len

        // Collect cut points along this edge with Z levels
        walk_edge_with_tabs(a, b, edge_start_dist, edge_end_dist,
                           &tab_intervals, pass_depth, tool.feed_rate,
                           &mut segments)

        running_dist = edge_end_dist

    // Close contour back to start point
    segments.push(Cut { x: contour.outer[0].x, y: contour.outer[0].y,
                        z: current_z_at(0.0, ...), feed: tool.feed_rate })

// Retract to safe Z after all passes
segments.push(Rapid { x: last_x, y: last_y, z: config.safe_z })
```

**`walk_edge_with_tabs` detail:**

For a single edge from `a` to `b` spanning arc-distance `[edge_start, edge_end]`, find all tab intervals that overlap this range. Split the edge into segments at tab boundaries, emitting `Cut` moves with appropriate Z:
- Outside any tab: `z = -pass_depth`
- Inside a tab: `z = -(pass_depth - tab.height)`, clamped so Z never goes above 0.0
- At tab entry/exit boundaries: ramp Z linearly over a short distance (2mm or tab edge, whichever is shorter)

#### Task 2.3: Unit tests for toolpath builder

**File:** `crates/thetta-cam/src/toolpath.rs` (inline tests)

| Test | Input | Expected |
|------|-------|----------|
| `test_simple_rectangle_no_tabs` | 100×50mm rectangle, no tabs, 6mm depth, 6mm step-down | 1 pass, 4 cut segments (one per edge) + close, rapid approach + retract |
| `test_rectangle_with_step_down` | 100×50mm rectangle, no tabs, 18mm depth, 6mm step-down | 3 passes, each with 4 cuts + close |
| `test_rectangle_with_one_tab` | 100×50mm rect, 1 tab on edge 0 at t=0.5, 7mm height | At tab: Z lifts from -18 to -11 (18-7), then back to -18 |
| `test_tab_z_clamped_above_zero` | Tab height > pass depth on first pass | Z at tab clamped to 0.0 (never above surface) |
| `test_no_contour` | Empty polygon | Empty toolpath |
| `test_start_and_end_at_safe_z` | Any valid input | First segment is Rapid to safe_z, last is Rapid to safe_z |
| `test_multiple_tabs_on_one_edge` | Long edge with 2 tabs | Two Z lifts on that edge, in correct order |

### Phase 3: `thetta-cam` Crate — G-code Emitter

**Goal:** Convert toolpath segments into LinuxCNC-compatible G-code text.

#### Task 3.1: G-code emitter

**File:** `crates/thetta-cam/src/gcode.rs`

```rust
/// Emit a complete G-code program for one sheet.
///
/// Returns the G-code as a String.
pub fn emit_gcode(
    toolpaths: &[Toolpath],
    tool: &Tool,
    config: &GcodeConfig,
    sheet_index: usize,
) -> String
```

Output format:
```gcode
(Sheet 1 - Generated by Thetta)
(Tool: T1 6mm 2-flute upcut)
G21 (Metric)
G90 (Absolute positioning)
G17 (XY plane)
T1 M6 (Tool change)
S18000 M3 (Spindle on CW)
G0 Z5.000 (Safe Z)

(Piece 1)
G0 X10.000 Y20.000 (Rapid to start)
G0 Z5.000
G1 Z-6.000 F500 (Plunge pass 1)
G1 X110.000 Y20.000 F1500
G1 X110.000 Y70.000
... (contour)
G1 Z-12.000 F500 (Plunge pass 2)
... (contour)
G1 Z-18.000 F500 (Plunge pass 3)
... (contour with tab lifts)
G0 Z5.000 (Retract)

(Piece 2)
G0 X200.000 Y30.000
...

M5 (Spindle off)
G0 Z5.000 (Retract to safe Z)
G0 X0.000 Y0.000 (Return to origin)
M2 (Program end)
```

Formatting rules:
- Coordinates formatted to 3 decimal places
- Feed rates as integers (F1500, not F1500.000)
- Comments in parentheses on the same line as the command
- Blank line between pieces for readability
- G0 for rapids (no feed rate), G1 for cuts (with feed rate)
- Modal: only emit G0/G1 when mode changes, only emit F when feed changes

#### Task 3.2: Unit tests for G-code emitter

| Test | Input | Expected |
|------|-------|----------|
| `test_preamble_format` | Any valid input | Output starts with G21, G90, G17, T/M6, S/M3 |
| `test_postamble_format` | Any valid input | Output ends with M5, G0 Z safe, G0 X0 Y0, M2 |
| `test_coordinate_precision` | Point at (1.0/3.0, 2.0/3.0, ...) | Coordinates have exactly 3 decimal places |
| `test_feed_rate_format` | Feed 1500.0 | Emitted as F1500 (integer, no decimals) |
| `test_empty_toolpaths` | Empty slice | Valid G-code with preamble + postamble only |
| `test_modal_gcode` | Consecutive Cut segments | G1 emitted once, subsequent cuts omit G1 |

### Phase 4: `thetta-cam` — Sheet Orchestrator

**Goal:** Top-level function that combines toolpath building and G-code emission for a complete sheet.

#### Task 4.1: Sheet G-code generator

**File:** `crates/thetta-cam/src/generate.rs`

```rust
/// Input for generating G-code for one sheet.
pub struct SheetGcodeInput<'a> {
    pub sheet_index: usize,
    pub placements: &'a [Placement],
    pub pieces: &'a [Piece],
    pub tabs: &'a HashMap<(usize, usize), Vec<Tab>>,
    pub tool: &'a Tool,
    pub config: &'a GcodeConfig,
}

/// Generate G-code for all pieces on a single sheet.
/// Returns the G-code program as a String.
pub fn generate_sheet_gcode(input: &SheetGcodeInput) -> String {
    let mut toolpaths = Vec::new();

    for (placement_idx, placement) in input.placements.iter().enumerate() {
        let Some(piece) = input.pieces.iter().find(|p| p.id == placement.piece_id) else {
            continue;
        };

        // Apply placement transform
        let center = Point2D { x: 0.0, y: 0.0 };
        let rotated = polygon_ops::rotate(&piece.polygon, placement.rotation, center);
        let placed = polygon_ops::translate(&rotated, placement.x, placement.y);

        // Get tabs for this placement
        let empty_tabs = Vec::new();
        let tabs = input.tabs
            .get(&(input.sheet_index, placement_idx))
            .unwrap_or(&empty_tabs);

        let toolpath = toolpath::build_profile_toolpath(&placed, tabs, input.tool, input.config);
        toolpaths.push(toolpath);
    }

    gcode::emit_gcode(&toolpaths, input.tool, input.config, input.sheet_index)
}

/// Generate G-code for all sheets, returning one String per sheet.
pub fn generate_all_sheets(
    result: &NestingResult,
    pieces: &[Piece],
    tabs: &HashMap<(usize, usize), Vec<Tab>>,
    tool: &Tool,
    config: &GcodeConfig,
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
                config,
            };
            (sheet_idx, generate_sheet_gcode(&input))
        })
        .collect()
}
```

### Phase 5: Tool Database Persistence

**Goal:** Save/load tool table to disk as JSON.

#### Task 5.1: Tool table file I/O

**File:** `crates/thetta-cam/src/tool_db.rs` (new)

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

/// Save tool table to disk. Creates parent directory if needed.
pub fn save_tool_table(table: &ToolTable) -> Result<(), std::io::Error> {
    let Some(path) = tool_db_path() else {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "no config dir"));
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(table)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    std::fs::write(&path, json)
}
```

**`Cargo.toml` update:** Add `serde_json` and `dirs` to `thetta-cam` dependencies.

```toml
[dependencies]
thetta-geometry = { path = "../thetta-geometry" }
serde_json = "1"
dirs = "5"
```

### Phase 6: App Integration — State, UI, and Export

**Goal:** Wire tool database, G-code config, and export trigger into the app.

#### Task 6.1: App state additions

**File:** `crates/thetta-app/src/state.rs`

Add to `AppState`:
```rust
pub tool_table: ToolTable,
pub gcode_config: GcodeConfig,
gcode_rx: Option<Receiver<Result<Vec<PathBuf>, String>>>,
gcode_status: Option<String>,
```

In `AppState::new()`, load tool table from disk:
```rust
tool_table: thetta_cam::tool_db::load_tool_table(),
```

Add G-code generation method:
```rust
impl AppState {
    pub fn start_gcode_export(&mut self, output_dir: PathBuf) {
        let Some(result) = self.current_result.clone() else { return };
        let Some(tool_num) = self.gcode_config.profile_tool_number else { return };
        let Some(tool) = self.tool_table.by_number(tool_num).cloned() else { return };

        let pieces = self.pieces.clone();
        let tabs = self.tab_state.results.clone();
        let config = self.gcode_config.clone();
        let (tx, rx) = mpsc::channel();
        self.gcode_rx = Some(rx);

        thread::spawn(move || {
            let sheets = thetta_cam::generate::generate_all_sheets(
                &result, &pieces, &tabs, &tool, &config,
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
                        self.gcode_status = Some(format!("Exported {} G-code file(s)", paths.len()));
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

#### Task 6.2: UI panels

**File:** `crates/thetta-app/src/panels.rs`

Add two new UI sections after the Tabs panel:

**Tool Database section:**
- List of tools (table: number, name, diameter, feed, RPM)
- "Add Tool" button → inline form with fields for all `Tool` properties
- Edit button per tool row → same form, pre-filled
- Delete button per tool row (with confirmation)
- Changes auto-save via `save_tool_table()`

**G-code section:**
- `profile_tool_number` dropdown populated from `ToolTable.tools`
- `safe_z` slider (1.0–20.0mm, default 5.0)
- `cut_depth` slider (1.0–50.0mm, default 18.0)
- `step_down` slider (1.0–cut_depth mm, default 6.0)
- "Generate G-code" button (disabled if no tool selected or no nesting result)
- Status line showing export result or error

Return `PanelAction::GenerateGcode` when "Generate G-code" clicked. Return `PanelAction::SaveToolTable` when tool database changes.

#### Task 6.3: Main loop integration

**File:** `crates/thetta-app/src/main.rs`

Handle new `PanelAction` variants:
- `GenerateGcode`: open native save-directory dialog, then call `app_state.start_gcode_export(dir)`
- `SaveToolTable`: call `thetta_cam::tool_db::save_tool_table(&app_state.tool_table)`

Add `app_state.poll_gcode_export()` to the frame update loop.

#### Task 6.4: Add `thetta-cam` dependency

**File:** `crates/thetta-app/Cargo.toml`

```toml
thetta-cam = { path = "../thetta-cam" }
```

### Phase 7: Integration Tests

**Goal:** End-to-end tests that verify G-code output correctness.

#### Task 7.1: Toolpath integration tests

**File:** `crates/thetta-cam/tests/toolpath_tests.rs`

| Test | Scenario | Verification |
|------|----------|-------------|
| `test_full_rectangle_profile` | 100×50mm rect, tool 6mm, 18mm depth, 6mm step-down, no tabs | 3 depth passes, each traces 4 edges, starts/ends at safe-Z |
| `test_rectangle_with_tabs` | 100×50mm rect, 2 tabs on long edges | Tab positions produce Z lifts in toolpath |
| `test_multi_piece_sheet` | 3 pieces on one sheet | 3 independent toolpaths, rapid between each |
| `test_empty_sheet` | Sheet with no placements | Empty result |

#### Task 7.2: G-code output tests

**File:** `crates/thetta-cam/tests/gcode_tests.rs`

| Test | Scenario | Verification |
|------|----------|-------------|
| `test_gcode_parseable` | Generate G-code for simple rect | Every non-comment line matches `G0|G1|T|S|M|G21|G90|G17` pattern |
| `test_gcode_z_never_positive_during_cut` | Any profile | No G1 Z value > 0.0 |
| `test_gcode_tab_z_heights` | Profile with known tab | G1 Z values at tab location match expected -(depth - tab_height) |
| `test_gcode_tool_change` | Tool T3 | Output contains "T3 M6" |

#### Task 7.3: Tool database tests

**File:** `crates/thetta-cam/tests/tool_db_tests.rs`

| Test | Scenario | Verification |
|------|----------|-------------|
| `test_roundtrip` | Save and load tool table | Loaded table matches saved |
| `test_missing_file` | Load from non-existent path | Returns empty default |
| `test_corrupt_file` | Load from file with invalid JSON | Returns empty default |

## Files Summary

### New Files

| File | Purpose | Est. LOC |
|------|---------|----------|
| `crates/thetta-cam/Cargo.toml` | Crate manifest | 15 |
| `crates/thetta-cam/src/lib.rs` | Module declarations | 10 |
| `crates/thetta-cam/src/toolpath.rs` | Profile toolpath builder + unit tests | 350 |
| `crates/thetta-cam/src/gcode.rs` | G-code emitter + unit tests | 200 |
| `crates/thetta-cam/src/generate.rs` | Per-sheet orchestration | 80 |
| `crates/thetta-cam/src/tool_db.rs` | Tool database persistence | 50 |
| `crates/thetta-cam/tests/toolpath_tests.rs` | Toolpath integration tests | 150 |
| `crates/thetta-cam/tests/gcode_tests.rs` | G-code output tests | 100 |
| `crates/thetta-cam/tests/tool_db_tests.rs` | Tool DB roundtrip tests | 60 |
| **Total new** | | **~1015** |

### Modified Files

| File | Changes | Est. Delta |
|------|---------|------------|
| `Cargo.toml` (root) | Add `crates/thetta-cam` to workspace members | +1 |
| `crates/thetta-geometry/src/types.rs` | Add `Tool`, `ToolTable`, `GcodeConfig`, `ToolpathSegment`, `Toolpath` | +80 |
| `crates/thetta-app/Cargo.toml` | Add `thetta-cam` dependency | +1 |
| `crates/thetta-app/src/state.rs` | Add `tool_table`, `gcode_config`, G-code export lifecycle | +60 |
| `crates/thetta-app/src/panels.rs` | Add Tool Database and G-code config UI sections | +150 |
| `crates/thetta-app/src/main.rs` | Handle new `PanelAction` variants, poll G-code export | +20 |
| **Total modified** | | **~312** |

### Estimated Total: ~1327 LOC

## Definition of Done

### P0 — Must Ship

- [ ] `Tool` type with fields: `number`, `name`, `diameter`, `flute_count`, `feed_rate`, `plunge_rate`, `spindle_rpm` in `thetta-geometry/src/types.rs`
- [ ] `ToolTable` with `tools: Vec<Tool>`, `by_number()`, `next_number()` methods
- [ ] `GcodeConfig` with `safe_z`, `cut_depth`, `step_down`, `profile_tool_number` fields and sensible defaults
- [ ] `ToolpathSegment` enum with `Rapid`, `Cut`, `Plunge` variants
- [ ] `Toolpath` struct with `segments: Vec<ToolpathSegment>`
- [ ] `thetta-cam` crate exists in workspace, depends only on `thetta-geometry`
- [ ] `build_profile_toolpath()` generates correct segments for rectangular contour with no tabs
- [ ] `build_profile_toolpath()` generates multi-pass depth passes when `step_down < cut_depth`
- [ ] `build_profile_toolpath()` lifts Z at tab positions: `z = -(pass_depth - tab_height)`, clamped to 0.0
- [ ] `build_profile_toolpath()` returns empty toolpath for empty polygon
- [ ] Toolpath starts with rapid to safe-Z and ends with retract to safe-Z
- [ ] `emit_gcode()` produces valid LinuxCNC G-code with G21, G90, G17 preamble
- [ ] `emit_gcode()` includes T/M6 tool change and S/M3 spindle start
- [ ] `emit_gcode()` includes M5, safe-Z retract, and M2 postamble
- [ ] G-code coordinates formatted to 3 decimal places
- [ ] G-code Z values never positive during G1 cut moves
- [ ] `generate_sheet_gcode()` combines multiple piece toolpaths with rapid traverses between pieces
- [ ] `generate_all_sheets()` produces one G-code string per non-empty sheet
- [ ] Tool database saves to and loads from JSON file on disk
- [ ] Tool database handles missing/corrupt file gracefully (returns empty default)
- [ ] UI: Tool Database panel with add/edit/delete tool functionality
- [ ] UI: G-code config panel with safe-Z, cut depth, step-down, tool selection
- [ ] UI: "Generate G-code" button triggers background export
- [ ] UI: G-code export status shown after completion
- [ ] One `.ngc` file written per sheet to user-selected directory
- [ ] `cargo test --workspace` passes with no regressions
- [ ] `cargo clippy --workspace` reports no new warnings

### P1 — Should Ship

- [ ] G-code includes comments: sheet number, tool description, piece number per profile
- [ ] Modal G-code output: G0/G1/F only emitted when mode or feed changes
- [ ] Feed rate changes correctly between plunge (plunge_rate) and cut (feed_rate) moves
- [ ] Tool table auto-saves on every change (add/edit/delete)
- [ ] G-code file naming includes descriptive info: `sheet_1_T1_6mm.ngc`
- [ ] Empty sheets (no placements) produce no output file
- [ ] "Generate G-code" button disabled when no tool selected or no nesting result
- [ ] Integration test: rectangle with known tabs produces G-code with verifiable Z heights at tab positions

### P2 — Stretch

- [ ] G-code dry-run validation: check for self-intersecting toolpaths
- [ ] Estimated machining time displayed in UI (total path length / feed rate)
- [ ] Tool library presets (common endmill sizes with typical feeds/speeds)
- [ ] G-code output preview in a scrollable text widget

### Cut if Behind Schedule

- Multi-pass step-down → cut full depth in one pass (simpler but less practical)
- G-code comments → bare G-code without annotations
- Tool database persistence → tools stored only in memory, lost on restart
- Modal G-code optimization → always emit G0/G1/F on every line

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Tab Z-height calculation incorrect, tool cuts through tabs | Medium | High | Unit test with known tab positions verifying exact Z values at tab entry, center, and exit. Visual verification in CAMotics simulator. |
| Contour winding direction wrong, tool cuts on wrong side | Medium | High | Assert CCW outer winding (same as tabs module). Conventional milling follows CCW naturally. |
| Multi-pass depth accumulation drift from floating-point | Low | Medium | Compute each pass depth from integer pass number: `depth = min(pass_num * step_down, cut_depth)`. Final pass always exactly `cut_depth`. |
| Tab interval wraps around contour start/end point | Medium | Medium | Handle wrap-around case: if tab straddles contour start (distance near 0 or near perimeter), split into two intervals. Test with tab placed near vertex 0. |
| Large contour produces excessive G-code (many small segments from high-resolution polygon) | Low | Low | G-code file size is proportional to polygon vertex count. Typical STEP-extracted profiles have <100 vertices. Not a concern for this sprint. |
| `dirs::config_dir()` returns None on unusual system | Low | Low | Fallback: log warning and keep tool table in memory only. User can still use the app, just tools won't persist. |
| G-code dialect incompatible with specific LinuxCNC version | Low | Medium | Target LinuxCNC 2.8+ (current stable). Use only standard G-codes (G0, G1, G17, G21, G90, T, M2, M3, M5, M6). Avoid extended codes. |
| No cutter compensation means parts are slightly oversized by tool radius | Medium | Medium | Document clearly in UI that nesting separation should account for tool diameter. Cutter comp (G41/G42) is a future sprint. |
| Step-down creates visible ridges on tab walls from different pass heights | Low | Low | Cosmetic issue only. Tabs are broken off after machining. Not a functional concern. |

## Security Considerations

- **No network access.** G-code generation is pure local computation. Tool database is a local file.
- **File writes limited to user-selected directory.** G-code export uses a native save dialog (same pattern as STEP export). No writes outside user-chosen path except tool database in config dir.
- **No unsafe code.** All new code is safe Rust. No FFI.
- **New dependencies are minimal and well-audited.** `dirs` (platform config paths) and `serde_json` (JSON serialization) are widely used crates. `serde_json` is already a transitive dependency.
- **Input validation.** Tool parameters validated in UI (sliders with min/max). `GcodeConfig` values clamped: `safe_z > 0`, `cut_depth > 0`, `step_down > 0`. Invalid configs produce empty output rather than panicking.
- **No arbitrary file path construction.** Output filenames are hardcoded patterns (`sheet_N.ngc`), not user-controlled strings. No path traversal risk.
- **Tool database corruption.** Corrupt JSON is handled by returning empty default. No deserialization of untrusted types — only `ToolTable` with primitive fields.

## Dependencies

### New External Dependencies

| Crate | Version | Purpose | Justification |
|-------|---------|---------|---------------|
| `dirs` | 5.x | Platform config directory path | Needed for tool database file location. Small, widely used, no transitive bloat. |
| `serde_json` | 1.x | Tool table JSON serialization | Already a transitive dependency in the workspace. |

### New Internal Dependencies

| Edge | Purpose |
|------|---------|
| `thetta-cam` → `thetta-geometry` | Access to `Polygon`, `Tab`, `Tool`, `Toolpath`, etc. |
| `thetta-app` → `thetta-cam` | G-code generation and tool database I/O |

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
    |    |                    |                       |
    |    |       +------------+----------+------------+
    |    |       |                       |
    |    |       |                       |
    |    | +-----+--------+    +--------+---------+
    |    | | thetta-nesting|   |                   |
    |    | | (optimizer)   |   |                   |
    |    | +-----+--------+   |                   |
    |    |       ^             |                   |
    |    |       |             |                   |
    |  +-+-------+--------+   |                   |
    |  |   thetta-app     |---+                   |
    |  |  (eframe UI)     |                       |
    |  +------------------+                       |
    |                                             |
    +---+---------------+                         |
        |  thetta-cam   |                         |
        |  (G-code CAM) |-------------------------+
        +---------------+
```

Note: `thetta-cam` depends only on `thetta-geometry`. It has no dependency on `jagua-rs`, `thetta-nesting`, or `thetta-step`.

## Open Questions

1. **Cutter radius compensation.** This sprint cuts on the contour centerline. Should the sprint document a recommended workflow (e.g., "set nesting separation to tool diameter + desired gap") or is that out of scope?

2. **Contour start point.** The profile cut starts at `contour.outer[0]`. Should there be logic to choose an optimal start point (e.g., on the longest edge, away from corners)? Or is vertex 0 acceptable for this sprint?

3. **Feed rate during tab ramp.** When the tool ramps up/down at a tab boundary, should the feed rate be the cut feed rate or a reduced ramp rate? Most CAM software uses cut feed rate for short Z ramps.

4. **Retract between depth passes.** When doing multi-pass step-down, should the tool retract to safe-Z between passes (safe but slow) or stay at current Z and re-enter at the contour start (faster but riskier if there's vibration)?

5. **Tool number validation.** Should the app warn if the selected tool number conflicts with the LinuxCNC tool table on the machine? Or is that out of scope (user's responsibility)?

6. **G-code file encoding.** LinuxCNC expects ASCII. Should we explicitly validate that tool names and comments contain only ASCII characters?

7. **`dirs` crate on Linux without XDG.** `dirs::config_dir()` returns `~/.config` on Linux. Is this the right location, or should we use the application's working directory as a fallback?
