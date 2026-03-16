# Sprint 008: Hole Drilling and Pocket Clearing

## Overview

Complete the 2D CAM pipeline by adding hole drilling (G83 peck drill cycles) and pocket clearing (contour-parallel offset milling) to the existing `thetta-cam` crate. After Sprint 007 established profile cuts with tool radius compensation, tab-aware toolpaths, and the LinuxCNC G-code emitter, the remaining two CAM operations from IDEA.md are hole drilling and pocket clearing. Both operate on inner contours (`Polygon.holes`) that are already parsed from STEP import but currently unused in the CAM pipeline.

The key challenge is classifying each hole contour: circular or near-circular holes that fit the assigned drill bit become drill operations (canned cycles), while non-circular or oversized holes become pocket operations (area-clearing toolpaths). Classification uses a dual check: circularity metric (`4*pi*area / perimeter^2 > 0.95`) AND tool-fit check (hole effective diameter <= drill tool diameter). The drill path emits G83 peck drill cycles with nearest-neighbor ordering. The pocket path generates concentric inward offsets at configurable step-over, entered via helical ramp at the pocket center. Both produce `ToolpathMove` sequences that feed into the existing G-code emitter.

Operation ordering follows IDEA.md: drill first, then pockets, then profile cuts. This per-sheet global ordering ensures pieces remain attached to the sheet (profile cuts with tabs are last) while holes and pockets are machined into still-rigid stock. Each operation type can use a different tool, with automatic tool changes (T/M6) emitted between operation groups. Same-tool optimization suppresses redundant tool changes when consecutive operations share a tool number.

**Scope:** Hole classification (drill vs pocket). Drill cycle planning and G83 emission. Pocket toolpath planning (contour-parallel offset, helical ramp entry). Multi-operation orchestration with tool changes. Per-operation tool assignment. Drill and pocket config types and UI panels. Tool diameter vs hole/pocket size safety validation. All three operations in per-sheet G-code output.
**Prerequisites:** Sprint 007 complete. `thetta-cam` crate with profile toolpath planner, G-code emitter, tool DB, and `generate.rs` orchestration. `Polygon.holes` populated from STEP import.
**Deferred:** Zigzag/raster pocketing, adaptive/trochoidal clearing, pocket island detection (raised areas within pockets), 3D toolpaths, in-app toolpath visualization.

## Use Cases

### UC1: Drill Through-Holes
User imports a STEP file with parts containing circular through-holes. After nesting, the CAM pipeline detects circular holes, determines their center coordinates, and generates G83 peck drill canned cycles. The user assigns a drill bit from the tool database. The G-code drills all holes on the sheet before any other operation.

### UC2: Clear Pockets
User imports parts with non-circular enclosed pockets (slots, rectangular cutouts, irregular shapes). The CAM pipeline classifies these as pockets and generates contour-parallel clearing toolpaths that step inward from the pocket boundary. The user assigns an endmill for pocket clearing. Pocket clearing occurs after drilling and before profile cuts.

### UC3: Mixed Operations on One Part
A part has two 5mm drill holes, one rectangular slot pocket, and an outer contour. The G-code output drills both holes (G83), clears the slot pocket (helical ramp entry + contour-parallel rings), then cuts the outer profile with tabs. Tool changes occur between operation groups if different tools are assigned.

### UC4: Assign Per-Operation Tools
User assigns T1 (4mm drill) to drilling, T2 (6mm endmill) to pocket clearing, and T3 (6mm endmill) to profile cuts. The G-code emits `T1 M6` before drill cycles, `T2 M6` before pockets, and `T3 M6` before profiles. If consecutive operations use the same tool, the redundant tool change is suppressed.

### UC5: Configure Drill Parameters
User configures peck depth for G83 peck drilling (default 5mm). Retract height between pecks is the safe-Z value from `GcodeConfig`.

### UC6: Configure Pocket Parameters
User sets step-over as a percentage of tool diameter (default 50%). These parameters are in `PocketConfig`.

### UC7: Pieces with No Holes
Parts with no inner contours produce only profile cuts. The drill and pocket phases are empty -- no tool changes are emitted for unused operations. The G-code is identical to Sprint 007 output for hole-free parts.

### UC8: Disable Operations
User can disable drilling or pocket clearing via toggle in the UI. Disabled operations are skipped during G-code generation. Profile cuts cannot be disabled (they are the primary operation).

### UC9: Multi-Sheet with All Operations
For multi-sheet nesting results, each sheet's G-code file includes all three operation types in the correct order. Each sheet file is self-contained.

### UC10: Tool Diameter Safety Validation
If the assigned drill bit diameter exceeds a classified drill hole's diameter, the system rejects the drill operation for that hole (prevents tool crash). If the pocket tool diameter exceeds a pocket's minimum dimension, the system skips that pocket with a warning.

## Architecture

### Component Overview

```
+-------------------------------------------------------------------+
|                          thetta-app                                |
|  +-----------+  +----------+  +---------------------------------+ |
|  | panels.rs |  |viewport.rs| |          state.rs               | |
|  | Drill cfg |  | (unchanged|  | DrillConfig, PocketConfig      | |
|  | Pocket cfg|  |  this     |  | OperationConfig (per-op tools) | |
|  | Operation |  |  sprint)  |  | Multi-op G-code generation     | |
|  | toggles   |  |          |  | Background thread + channel    | |
|  +-----+-----+  +----------+  +---------------+-----------------+ |
|        |                                       |                   |
+--------+---------------------------------------+-------------------+
         |                                       |
         |                       +---------------v------------------+
         |                       |          thetta-cam               |
         |                       |  +----------------------------+  |
         |                       |  |  hole_classify.rs (NEW)    |  |
         |                       |  |  classify_holes()          |  |
         |                       |  |  Circularity + tool-fit    |  |
         |                       |  +----------------------------+  |
         |                       |  +----------------------------+  |
         |                       |  |  drill.rs (NEW)            |  |
         |                       |  |  plan_drill_toolpath()     |  |
         |                       |  |  G83 peck cycle planning   |  |
         |                       |  |  Nearest-neighbor ordering |  |
         |                       |  +----------------------------+  |
         |                       |  +----------------------------+  |
         |                       |  |  pocket.rs (NEW)           |  |
         |                       |  |  plan_pocket_toolpath()    |  |
         |                       |  |  Contour-parallel offsets  |  |
         |                       |  |  Helical ramp entry        |  |
         |                       |  +----------------------------+  |
         |                       |  +----------------------------+  |
         |                       |  |  toolpath.rs (existing)    |  |
         |                       |  |  plan_profile_toolpath()   |  |
         |                       |  |  offset_contour() (reused) |  |
         |                       |  +----------------------------+  |
         |                       |  +----------------------------+  |
         |                       |  |  gcode.rs (extended)       |  |
         |                       |  |  emit_gcode() extended     |  |
         |                       |  |  + drill cycle emission    |  |
         |                       |  |  + multi-tool preamble     |  |
         |                       |  +----------------------------+  |
         |                       |  +----------------------------+  |
         |                       |  |  generate.rs (extended)    |  |
         |                       |  |  generate_sheet_gcode()    |  |
         |                       |  |  Multi-op orchestration:   |  |
         |                       |  |  drill -> pocket -> profile|  |
         |                       |  |  Per-operation tool changes|  |
         |                       |  +----------------------------+  |
         |                       +------------------+---------------+
         |                                          |
         |                       +------------------v---------------+
         |                       |        thetta-geometry            |
         +-------------------+-->|  types.rs                         |
                                 |  + DrillConfig, PocketConfig      |
                                 |  + HoleClassification enum        |
                                 |  + OperationConfig                |
                                 |  + New ToolpathMove variants      |
                                 |  + (existing) Polygon, Tool, etc  |
                                 +----------------------------------+
```

### New Module: `hole_classify.rs`

Analyzes `Polygon.holes` to classify each inner contour as either a drill hole or a pocket. Classification uses a dual check:

1. **Circularity**: `4 * pi * area / perimeter^2`. A perfect circle scores 1.0. Threshold default: 0.95.
2. **Tool fit**: The hole's effective diameter must be <= the drill tool diameter. A circular hole larger than the drill bit cannot be drilled (it becomes a pocket).

For drill holes, the classifier computes the center (centroid) and effective diameter (2 * average distance from center to vertices). For pockets, the original contour vertices are returned for toolpath planning.

All holes must be transformed to world space using the placement transform before classification, since the circularity metric is scale-invariant but drill center coordinates must be in machine coordinates.

```rust
/// A hole classified for drilling.
pub struct DrillHole {
    pub center: Point2D,       // World-space center
    pub diameter: f64,         // Effective diameter
}

/// A hole classified for pocket clearing.
pub struct PocketRegion {
    pub contour: Vec<Point2D>, // World-space contour (CW winding from Polygon.holes)
}

/// Classification result for a single inner contour.
pub enum HoleClassification {
    Drill(DrillHole),
    Pocket(PocketRegion),
}

/// Classify all holes in a polygon.
///
/// `drill_tool_diameter`: diameter of the assigned drill tool.
/// If `None`, all holes are classified as pockets.
pub fn classify_holes(
    holes: &[Vec<Point2D>],
    circularity_threshold: f64,
    drill_tool_diameter: Option<f64>,
) -> Vec<HoleClassification>
```

**Algorithm pseudocode:**
```
for each hole contour in holes:
    area = abs(signed_ring_area(contour))
    perimeter = ring_perimeter(contour)
    circularity = 4 * PI * area / (perimeter * perimeter)

    if circularity > circularity_threshold AND drill_tool_diameter is Some(d):
        center = ring_centroid(contour)
        effective_diameter = 2.0 * avg_distance_from_center(contour, center)
        if effective_diameter <= d:
            classify as Drill { center, diameter: effective_diameter }
        else:
            classify as Pocket { contour }  // too large for drill
    else:
        classify as Pocket { contour }
```

### New Module: `drill.rs`

Plans drill toolpaths for classified drill holes. Produces a three-part `ToolpathMove` sequence that maps directly to LinuxCNC canned cycle semantics:

1. `DrillCycleStart` -- sets cycle parameters (G83, depth, retract plane, peck depth, feed)
2. `DrillHole` -- one per hole (X, Y coordinates only; controller handles Z motion)
3. `DrillCycleEnd` -- cancels canned cycle mode (G80)

```rust
/// New ToolpathMove variants for drill cycles:
pub enum ToolpathMove {
    // ... existing variants (Rapid, LinearCut, ArcCut, TabLift, Retract, Plunge) ...

    /// Start a G83 peck drill canned cycle. Sets modal parameters.
    DrillCycleStart {
        depth: f64,       // Total drill depth (positive, from Z=0 down)
        peck_depth: f64,  // Per-peck increment in mm
        retract_z: f64,   // R-plane: retract height between pecks
        feed: f64,        // Plunge feed rate
    },

    /// A single drill hole location within an active canned cycle.
    DrillHole {
        position: Point2D, // XY center of the hole
    },

    /// Cancel canned cycle mode (G80).
    DrillCycleEnd,

    /// Helical ramp entry for pocket clearing.
    HelicalRamp {
        center: Point2D,  // Center of helix in XY
        radius: f64,      // Helix radius
        start_z: f64,     // Z at start of helix (safe_z or 0)
        end_z: f64,       // Z at bottom of helix (cut depth)
        clockwise: bool,  // Helix direction
        feed: f64,        // Feed rate during ramp
    },
}
```

```rust
pub fn plan_drill_toolpath(
    holes: &[DrillHole],
    tool: &Tool,
    drill_config: &DrillConfig,
    gcode_config: &GcodeConfig,
) -> Toolpath
```

**Algorithm pseudocode:**
```
if holes is empty:
    return empty Toolpath

// Nearest-neighbor ordering to minimize rapid travel
ordered_holes = nearest_neighbor_sort(holes)

moves = []

// Rapid to first hole at safe-Z
moves.push(Rapid { to: (ordered_holes[0].center, safe_z) })

// Start canned cycle
moves.push(DrillCycleStart {
    depth: gcode_config.cut_depth,
    peck_depth: drill_config.peck_depth,
    retract_z: gcode_config.safe_z,
    feed: tool.plunge_rate,
})

// Emit each hole
for hole in ordered_holes:
    moves.push(DrillHole { position: hole.center })

// Cancel cycle
moves.push(DrillCycleEnd)
moves.push(Retract { to_z: gcode_config.safe_z })

return Toolpath { moves, config }
```

**Nearest-neighbor sort:**
```
Start from first hole. Repeatedly pick the unvisited hole closest to the current
position. This is a greedy O(n^2) heuristic that works well for typical hole counts
(< 100 per sheet).
```

### New Module: `pocket.rs`

Plans pocket clearing toolpaths using contour-parallel (offset inward) strategy with helical ramp entry.

```rust
pub fn plan_pocket_toolpath(
    pocket: &PocketRegion,
    tool: &Tool,
    pocket_config: &PocketConfig,
    gcode_config: &GcodeConfig,
) -> Toolpath
```

**Algorithm pseudocode:**
```
contour = pocket.contour
// Reverse CW hole winding to CCW for offset_contour() compatibility
if signed_ring_area(contour) < 0:
    contour.reverse()

tool_radius = tool.diameter / 2.0
step = tool.diameter * pocket_config.step_over

// Generate offset rings (inward from pocket boundary)
rings = []
offset_distance = tool_radius   // first ring: tool radius compensation
loop:
    offset_polygon = offset_contour(contour_as_polygon, -offset_distance)
    if offset_polygon.outer.len() < 3 or polygon_area(offset_polygon) < tool.diameter^2:
        break
    rings.push(offset_polygon.outer)
    offset_distance += step

if rings is empty:
    // Pocket too small for tool -- skip with warning
    return empty Toolpath

// Validate: tool diameter vs pocket minimum dimension
pocket_bb = bounding_box(contour)
min_dimension = min(pocket_bb.width(), pocket_bb.height())
if tool.diameter > min_dimension:
    log warning "Tool diameter exceeds pocket minimum dimension"
    return empty Toolpath

moves = []

// Helical ramp entry at center of innermost ring (or pocket centroid)
ramp_center = centroid(rings.last())  // innermost ring center
ramp_radius = tool.diameter / 4.0     // small helix radius
moves.push(Rapid { to: (ramp_center.x + ramp_radius, ramp_center.y, safe_z) })
moves.push(HelicalRamp {
    center: ramp_center,
    radius: ramp_radius,
    start_z: 0.0,          // material surface
    end_z: -cut_depth,     // full depth
    clockwise: true,
    feed: tool.plunge_rate,
})

// Cut offset rings from outermost to innermost
for ring in rings (outermost first):
    // Rapid to ring start at safe-Z, plunge to cut depth
    moves.push(Rapid { to: (ring[0], safe_z) })
    moves.push(Plunge { to_z: -cut_depth, feed: tool.plunge_rate })

    // Walk ring contour
    for point in ring[1..]:
        moves.push(LinearCut { to: (point, -cut_depth), feed: tool.feed_rate })

    // Close ring
    moves.push(LinearCut { to: (ring[0], -cut_depth), feed: tool.feed_rate })

moves.push(Retract { to_z: safe_z })
return Toolpath { moves, config }
```

**Pocket entry (helical ramp):**
The tool enters the pocket via a helical ramp at the pocket center. The helix radius is `tool_diameter / 4` and the tool descends one full revolution to reach cut depth. This avoids vertical plunging into material, which is safer for tools and provides better chip evacuation. The helical ramp produces a G2/G3 arc with simultaneous Z descent.

**Ring traversal order:**
Rings are cut from outermost to innermost. This clears material progressively and keeps the tool engaged in previously-cleared areas. Between rings, a retract-rapid-plunge sequence repositions the tool.

### Extended: `gcode.rs`

The G-code emitter gains handlers for new `ToolpathMove` variants:

**Drill cycle emission:**
```
G83 Z{-depth} R{retract_z} Q{peck_depth} F{feed}  (start peck cycle)
X{x1} Y{y1}                                         (first hole)
X{x2} Y{y2}                                         (subsequent holes)
G80                                                   (cancel cycle)
```

The `DrillCycleStart` sets modal parameters. Each `DrillHole` emits only XY coordinates (the controller handles Z motion within the active cycle). `DrillCycleEnd` emits `G80`.

**Helical ramp emission:**
```
G0 X{cx+r} Y{cy} Z{start_z}     (position at helix start)
G2 X{cx+r} Y{cy} Z{end_z} I{-r} J{0} F{feed}  (full-circle helix)
```

A single full-circle arc (G2 or G3) with Z descent from `start_z` to `end_z`. LinuxCNC supports helical motion natively as an arc with a Z component.

**Multi-tool emission structure:**
```rust
pub fn emit_multi_op_gcode(
    drill_toolpaths: &[Toolpath],
    pocket_toolpaths: &[Toolpath],
    profile_toolpaths: &[Toolpath],
    drill_tool: Option<&Tool>,
    pocket_tool: Option<&Tool>,
    profile_tool: &Tool,
    config: &GcodeConfig,
    sheet_index: usize,
) -> String
```

Tool change logic:
```
last_tool_number = None

for each operation group (drill, pocket, profile) that has toolpaths:
    if group.tool.number != last_tool_number:
        if last_tool_number.is_some():
            emit "M5" (spindle off)
        emit "T{n} M6" (tool change)
        emit "S{rpm} M3" (spindle on)
        last_tool_number = group.tool.number
    emit group toolpaths
```

### Extended: `generate.rs`

The orchestrator changes from single-operation to multi-operation per sheet:

```
For each Sheet:
  1. For each Placement:
     a. Look up Piece polygon
     b. Apply placement transform -> world-space polygon (outer + holes)
     c. classify_holes(world_holes, circularity_threshold, drill_tool_diameter)
     d. Collect DrillHoles and PocketRegions

  2. DRILL PHASE (if drill enabled and any drill holes exist):
     a. plan_drill_toolpath(all_drill_holes, drill_tool, drill_config, gcode_config)

  3. POCKET PHASE (if pocket enabled and any pockets exist):
     a. For each pocket: plan_pocket_toolpath(pocket, pocket_tool, pocket_config, gcode_config)

  4. PROFILE PHASE:
     a. For each piece: plan_profile_toolpath(contour, tabs, profile_tool, ...) (existing S007 code)

  5. emit_multi_op_gcode(drill_paths, pocket_paths, profile_paths, tools, config)
     a. Preamble: %, G21, G90, G17, coordinate system
     b. Drill section: tool change + drill cycles + G80
     c. Pocket section: tool change (if different) + pocket moves
     d. Profile section: tool change (if different) + profile moves
     e. Postamble: M5, retract, home, M2, %

  6. Write to {output_dir}/sheet_{N}.ngc
```

Updated `SheetGcodeInput`:

```rust
pub struct SheetGcodeInput<'a> {
    pub sheet_index: usize,
    pub placements: &'a [Placement],
    pub pieces: &'a [Piece],
    pub tabs: &'a HashMap<(usize, usize), Vec<Tab>>,
    pub gcode_config: &'a GcodeConfig,
    pub profile_tool: &'a Tool,
    pub profile_config: &'a ProfileCutConfig,
    // New fields for Sprint 008
    pub drill_tool: Option<&'a Tool>,
    pub drill_config: &'a DrillConfig,
    pub pocket_tool: Option<&'a Tool>,
    pub pocket_config: &'a PocketConfig,
}
```

### New Types in `thetta-geometry/types.rs`

```rust
/// Configuration for drill operations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DrillConfig {
    /// Peck depth per cycle in mm (G83 Q value). Default 5.0mm.
    #[serde(default = "default_peck_depth")]
    pub peck_depth: f64,
    /// Tool number assigned to drill operations.
    #[serde(default)]
    pub tool_number: Option<u32>,
    /// Enable/disable drill operations.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Circularity threshold for classifying holes as drillable (0.0-1.0).
    /// Default 0.95. A perfect circle scores 1.0.
    #[serde(default = "default_circularity_threshold")]
    pub circularity_threshold: f64,
}

fn default_peck_depth() -> f64 { 5.0 }
fn default_true() -> bool { true }
fn default_circularity_threshold() -> f64 { 0.95 }

impl Default for DrillConfig {
    fn default() -> Self {
        Self {
            peck_depth: 5.0,
            tool_number: None,
            enabled: true,
            circularity_threshold: 0.95,
        }
    }
}

impl DrillConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.peck_depth <= 0.0 || !self.peck_depth.is_finite() {
            return Err("Peck depth must be positive".to_string());
        }
        if self.circularity_threshold <= 0.0 || self.circularity_threshold > 1.0 {
            return Err("Circularity threshold must be in (0.0, 1.0]".to_string());
        }
        if self.enabled && self.tool_number.is_none() {
            return Err("Drill operations enabled but no tool assigned".to_string());
        }
        Ok(())
    }
}

/// Configuration for pocket clearing operations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PocketConfig {
    /// Step-over as fraction of tool diameter (0.0-1.0). Default 0.5 (50%).
    #[serde(default = "default_step_over")]
    pub step_over: f64,
    /// Tool number assigned to pocket operations.
    #[serde(default)]
    pub tool_number: Option<u32>,
    /// Enable/disable pocket operations.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Whether to add a finish pass along the pocket wall at full depth.
    #[serde(default)]
    pub finish_pass: bool,
}

fn default_step_over() -> f64 { 0.5 }

impl Default for PocketConfig {
    fn default() -> Self {
        Self {
            step_over: 0.5,
            tool_number: None,
            enabled: true,
            finish_pass: false,
        }
    }
}

impl PocketConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.step_over <= 0.0 || self.step_over > 1.0 || !self.step_over.is_finite() {
            return Err("Step-over must be in (0.0, 1.0]".to_string());
        }
        if self.enabled && self.tool_number.is_none() {
            return Err("Pocket operations enabled but no tool assigned".to_string());
        }
        Ok(())
    }
}

/// Groups all operation configs and tool assignments for multi-operation G-code.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperationConfig {
    pub drill: DrillConfig,
    pub pocket: PocketConfig,
    pub profile: ProfileCutConfig,
}

impl Default for OperationConfig {
    fn default() -> Self {
        Self {
            drill: DrillConfig::default(),
            pocket: PocketConfig::default(),
            profile: ProfileCutConfig::default(),
        }
    }
}

/// Classification result for a single inner contour.
#[derive(Debug, Clone, PartialEq)]
pub enum HoleClassification {
    /// Circular hole suitable for drilling.
    Drill(DrillHole),
    /// Non-circular or oversized hole requiring pocket clearing.
    Pocket(PocketRegion),
}

#[derive(Debug, Clone, PartialEq)]
pub struct DrillHole {
    /// World-space center of the hole.
    pub center: Point2D,
    /// Effective diameter of the hole.
    pub diameter: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PocketRegion {
    /// World-space contour of the pocket boundary.
    pub contour: Vec<Point2D>,
}
```

### Design Decisions

1. **Dual classification criteria (circularity + tool fit).** A hole is drillable only if it is near-circular AND the drill bit fits. This prevents plunging oversized tools into small holes (safety) and avoids treating large circular cutouts as single drill operations when they should be pocketed (correctness).

2. **Three-variant drill model (DrillCycleStart/DrillHole/DrillCycleEnd).** Maps directly to LinuxCNC canned cycle semantics. The start variant sets modal parameters once, each hole is just XY coordinates, and the end variant cancels the cycle. This produces idiomatic G-code and is more faithful to the machine model than a single monolithic drill variant.

3. **G83 peck drill as default.** Peck drilling retracts between pecks for chip evacuation. Safer than simple drilling (G81) for through-holes in wood/composite, which is the target use case. Configurable peck depth defaults to 5mm.

4. **Helical ramp pocket entry.** The tool enters pockets via a helical ramp at the pocket center rather than a direct vertical plunge. The helix radius is `tool_diameter / 4` and descends one full revolution. This is safer for tools (reduced axial load) and provides better chip evacuation than plunging.

5. **Contour-parallel pocketing.** Repeated inward offsets of the pocket boundary produce better surface finish on pocket walls than zigzag/raster strategies. Reuses the existing `offset_contour()` from `toolpath.rs` (wrapping `geo-buffer`), minimizing new geometric code.

6. **Per-sheet global operation ordering.** All drill holes on the sheet, then all pockets, then all profiles. This minimizes tool changes (at most 3 per sheet) and matches IDEA.md requirement. Per-piece ordering would cause excessive tool changes.

7. **Same-tool optimization.** When consecutive operation groups use the same tool number, the redundant `M5`/`T/M6`/`S/M3` sequence is suppressed. This saves time without affecting correctness.

8. **Three separate modules.** `hole_classify.rs`, `drill.rs`, `pocket.rs` keep concerns isolated and prevent `toolpath.rs` from becoming a monolithic file. Each module has a single responsibility and clear public interface.

9. **Serde backward compatibility.** All new config fields use `#[serde(default)]` with explicit default functions. Existing saved configs that lack drill/pocket fields will deserialize correctly with sensible defaults.

10. **Tool diameter safety validation.** Reject drill if tool diameter > hole diameter. Skip pocket if tool diameter > pocket minimum dimension. Log warnings for skipped operations. This prevents physical tool crashes.

## Implementation

### Phase 1: Types + Hole Classification (~20%)

**Goal:** Add drill and pocket config types, implement hole classification logic.

#### Task 1.1: New Config Types

**File:** `crates/thetta-geometry/src/types.rs`

- Add `DrillConfig` with `peck_depth`, `tool_number`, `enabled`, `circularity_threshold`
- Add `PocketConfig` with `step_over`, `tool_number`, `enabled`, `finish_pass`
- Add `OperationConfig` grouping all three operation configs
- Add `HoleClassification` enum with `DrillHole` and `PocketRegion` structs
- Add new `ToolpathMove` variants: `DrillCycleStart`, `DrillHole`, `DrillCycleEnd`, `HelicalRamp`
- Add `Default` impls with sensible values
- Add `validate()` methods with parameter clamping/rejection
- All new fields use `#[serde(default)]` for backward compatibility

#### Task 1.2: Hole Classification Module

**File:** `crates/thetta-cam/src/hole_classify.rs` (new)

- Implement `circularity(contour: &[Point2D]) -> f64` using `4 * PI * area / perimeter^2`
  - Use `signed_ring_area()` from `toolpath.rs` (make pub or re-implement)
  - Perimeter: sum of edge lengths around the contour ring
- Implement `contour_center(contour: &[Point2D]) -> Point2D` (centroid = average of all vertices)
- Implement `contour_effective_diameter(contour: &[Point2D], center: Point2D) -> f64`
  - `2.0 * (sum of distances from center to each vertex) / vertex_count`
- Implement `classify_holes(holes, circularity_threshold, drill_tool_diameter) -> Vec<HoleClassification>`
  - For each hole: compute circularity, check threshold, check tool fit, classify
- Unit tests:
  - Perfect circle (32-vertex approximation) -> Drill with circularity > 0.99
  - Rectangle -> Pocket with circularity ~0.785
  - Near-circle (8-vertex approximation) -> depends on threshold
  - Circular hole larger than drill tool -> Pocket (tool fit fails)
  - Empty holes list -> empty result
  - Degenerate contour (< 3 vertices) -> skip

#### Task 1.3: Register New Modules

**File:** `crates/thetta-cam/src/lib.rs`

- Add `pub mod hole_classify;`
- Add `pub mod drill;`
- Add `pub mod pocket;`

### Phase 2: Drill Cycle Planning + Emission (~25%)

**Goal:** Generate G83 peck drill canned cycles from classified drill holes.

#### Task 2.1: Drill Toolpath Planner

**File:** `crates/thetta-cam/src/drill.rs` (new)

- Implement `plan_drill_toolpath(holes, tool, drill_config, gcode_config) -> Toolpath`
- Nearest-neighbor sort: start from first hole, greedily visit closest unvisited
- Emit: `Rapid` to first hole XY at safe-Z, `DrillCycleStart`, `DrillHole` per hole, `DrillCycleEnd`, `Retract`
- Tool diameter validation: skip holes where `tool.diameter > hole.diameter`, log warning
- Unit tests:
  - Single hole produces Rapid + DrillCycleStart + DrillHole + DrillCycleEnd + Retract
  - Multiple holes are nearest-neighbor ordered
  - Empty holes -> empty toolpath
  - Hole smaller than tool -> skipped

#### Task 2.2: G-code Drill Cycle Emission

**File:** `crates/thetta-cam/src/gcode.rs`

- Add `emit_move()` handler for `DrillCycleStart`:
  - Emit `G83 Z{-depth} R{retract_z} Q{peck_depth} F{feed}`
- Add handler for `DrillHole`:
  - Emit `X{x} Y{y}` (XY only, controller handles Z within active cycle)
- Add handler for `DrillCycleEnd`:
  - Emit `G80`
  - Reset modal motion state (canned cycle is a separate motion group)
- Add handler for `HelicalRamp`:
  - Emit `G2/G3 X{cx+r} Y{cy} Z{end_z} I{-r} J{0} F{feed}` (full-circle helix)
- Unit tests:
  - DrillCycleStart produces valid G83 line
  - DrillHole produces XY-only line
  - DrillCycleEnd produces G80
  - HelicalRamp produces G2/G3 with Z descent
  - Full sequence: no NaN/Inf in output

### Phase 3: Pocket Toolpath Planning (~25%)

**Goal:** Generate contour-parallel clearing toolpaths for pockets with helical ramp entry.

#### Task 3.1: Pocket Toolpath Planner

**File:** `crates/thetta-cam/src/pocket.rs` (new)

- Implement `plan_pocket_toolpath(pocket, tool, pocket_config, gcode_config) -> Toolpath`
- Reverse CW pocket contour to CCW (check signed area, reverse if negative)
- Generate offset rings:
  - First offset: inward by `tool.diameter / 2` (tool radius compensation for pocket wall)
  - Subsequent offsets: inward by `step_over * tool.diameter` from previous
  - Use `offset_contour()` from `toolpath.rs` with negative distance
  - Stop when offset polygon area < `tool.diameter^2` or fewer than 3 vertices
- Tool diameter validation: skip pocket if `tool.diameter > min(bb.width, bb.height)`, log warning
- Helical ramp entry:
  - Compute ramp center from centroid of innermost ring (or pocket centroid if no rings)
  - Ramp radius = `tool.diameter / 4`
  - Emit `Rapid` to ramp start, then `HelicalRamp` from Z=0 to Z=-cut_depth
- Ring traversal:
  - Cut rings from outermost to innermost
  - For each ring: `Rapid` to ring start at safe-Z, `Plunge` to cut depth, `LinearCut` along ring, close ring
  - Between rings: retract-rapid-plunge sequence
- `Retract` to safe-Z after last ring
- Unit tests:
  - Rectangular pocket (40x20mm with 6mm tool) produces multiple rings
  - Small pocket (smaller than tool) -> empty toolpath with warning
  - Square pocket produces correct ring count: `floor((side/2 - tool_radius) / step + 1)`
  - Output contains HelicalRamp move
  - All LinearCut Z values are at -cut_depth

#### Task 3.2: Pocket Finish Pass (P1)

**File:** `crates/thetta-cam/src/pocket.rs`

- If `pocket_config.finish_pass` is true, after clearing all offset rings:
  - Re-trace the first offset ring (outermost, at tool radius compensation) at full depth
  - This produces a clean wall finish by removing any scallops from step-over

### Phase 4: Multi-Operation Orchestration (~20%)

**Goal:** Extend `generate.rs` to orchestrate all three operations with correct ordering and tool changes.

#### Task 4.1: Extend `generate.rs`

**File:** `crates/thetta-cam/src/generate.rs`

- Update `SheetGcodeInput` with drill/pocket tool references, `DrillConfig`, `PocketConfig`
- For each placement:
  - Apply placement transform to polygon (existing code handles outer; extend to holes)
  - Call `classify_holes()` on world-space holes
  - Collect all `DrillHole`s and `PocketRegion`s across all placements
- Call `plan_drill_toolpath()` with all drill holes (if drill enabled and holes exist)
- Call `plan_pocket_toolpath()` per pocket (if pocket enabled and pockets exist)
- Call `plan_profile_toolpath()` per piece (existing logic, unchanged)
- Pass grouped toolpaths to `emit_multi_op_gcode()`

#### Task 4.2: Multi-Tool G-code Emission

**File:** `crates/thetta-cam/src/gcode.rs`

- New function `emit_multi_op_gcode()` (signature above)
- Preamble: `%`, `G21`, `G90`, `G17`, coordinate system
- For each non-empty operation group:
  - Tool change (if tool number differs from previous): `M5` + `T{n} M6` + `S{rpm} M3`
  - Emit operation toolpaths with piece-level comments
- Postamble: `M5`, `G0 Z{safe_z}`, `G0 X0 Y0`, `M2`, `%`
- Skip empty operation groups entirely (no tool change, no comments)
- Unit tests:
  - Drill-only sheet -> single tool change, G83 block, G80, postamble
  - Profile-only sheet (no holes) -> identical to Sprint 007 output
  - All three operations -> correct ordering, tool changes only when number changes
  - Same tool for drill and pocket -> single tool change for both

#### Task 4.3: Update `generate_all_sheets()`

**File:** `crates/thetta-cam/src/generate.rs`

- Update signature to accept `OperationConfig` and per-operation tool lookups
- Maintain backward compatibility: existing profile-only path still works
- Empty sheets produce no output (unchanged)

### Phase 5: App Integration (~10%)

**Goal:** Wire drill and pocket configuration into the UI and state management.

#### Task 5.1: State Extension

**File:** `crates/thetta-app/src/state.rs`

- Add `DrillConfig` and `PocketConfig` to `AppState` (or `OperationConfig` wrapper)
- Extend `gcode_export_precondition_error()` to validate:
  - Drill tool exists in tool table when drill enabled
  - Pocket tool exists in tool table when pocket enabled
  - Config validation passes for all enabled operations
- Update background thread to call multi-operation `generate_sheet_gcode()`
- Backward compatibility: existing app state loads cleanly with serde defaults

#### Task 5.2: UI Panels

**File:** `crates/thetta-app/src/panels.rs`

- Add drill configuration section in CAM panel:
  - Enable/disable toggle
  - Tool selection dropdown (from tool table)
  - Peck depth slider/input (1.0 - 50.0 mm range)
  - Circularity threshold slider (advanced, collapsed by default, 0.5 - 1.0 range)
- Add pocket configuration section in CAM panel:
  - Enable/disable toggle
  - Tool selection dropdown
  - Step-over percentage slider (10% - 100%)
  - Finish pass checkbox (P1)
- Show operation order summary: "Drill (T1) -> Pocket (T2) -> Profile (T3)"
- Disabled operations show grayed-out summary

## Files Summary

| File | Action | Description |
|------|--------|-------------|
| `crates/thetta-geometry/src/types.rs` | Modify | Add `DrillConfig`, `PocketConfig`, `OperationConfig`, `HoleClassification`, `DrillHole`, `PocketRegion`, new `ToolpathMove` variants |
| `crates/thetta-cam/src/lib.rs` | Modify | Register `hole_classify`, `drill`, `pocket` modules |
| `crates/thetta-cam/src/hole_classify.rs` | New | Hole circularity classification (drill vs pocket) with dual criteria |
| `crates/thetta-cam/src/drill.rs` | New | Drill cycle toolpath planning (G83 peck, nearest-neighbor ordering) |
| `crates/thetta-cam/src/pocket.rs` | New | Contour-parallel pocket clearing with helical ramp entry |
| `crates/thetta-cam/src/gcode.rs` | Modify | Add DrillCycleStart/DrillHole/DrillCycleEnd/HelicalRamp emission, multi-tool support |
| `crates/thetta-cam/src/generate.rs` | Modify | Multi-operation orchestration (drill -> pocket -> profile), extended SheetGcodeInput |
| `crates/thetta-cam/src/toolpath.rs` | Modify | Make `signed_ring_area`, `ring_centroid` public for reuse by `hole_classify.rs` |
| `crates/thetta-app/src/state.rs` | Modify | Add drill/pocket configs to state, update generation trigger, extend validation |
| `crates/thetta-app/src/panels.rs` | Modify | Add drill and pocket config UI panels with tool selectors and toggles |

## Definition of Done

### P0 (Must Ship)

- [ ] `classify_holes()` correctly distinguishes circular holes from non-circular pockets using circularity + tool-fit dual check
- [ ] Tool diameter vs hole diameter safety check: reject drill if tool > hole
- [ ] Tool diameter vs pocket minimum dimension safety check: skip pocket if tool too large
- [ ] Drill holes produce `DrillCycleStart` / `DrillHole` / `DrillCycleEnd` toolpath moves
- [ ] G-code emitter outputs valid G83 peck drill canned cycles with correct parameters
- [ ] G80 cancels canned cycle mode after drill section
- [ ] Drill holes sorted by nearest-neighbor to minimize rapid travel
- [ ] Pocket clearing produces contour-parallel toolpaths with configurable step-over
- [ ] Pocket toolpaths respect tool radius compensation (first offset = tool radius)
- [ ] Pocket entry via helical ramp (not vertical plunge)
- [ ] Operation ordering in G-code: drill -> pocket -> profile (IDEA.md requirement)
- [ ] Tool changes (T/M6) emitted between operation groups when tool number changes
- [ ] Same-tool optimization: skip redundant T/M6 when consecutive operations share tool
- [ ] Multi-sheet export includes all three operations per sheet
- [ ] Pieces with no holes produce profile-only G-code (no spurious tool changes)
- [ ] `DrillConfig` and `PocketConfig` types with validation and serde backward compatibility
- [ ] UI panels for drill and pocket configuration with tool assignment dropdowns
- [ ] Enable/disable toggles for drill and pocket operations
- [ ] World-space hole transformation (placement rotation + translation applied before classification)
- [ ] Unit tests for: classification, drill planning, pocket planning, orchestration ordering, G-code emission
- [ ] `cargo test --workspace` passes

### P1 (Should Ship)

- [ ] Pocket finish pass (re-trace pocket boundary at full depth for clean wall finish)
- [ ] Nearest-neighbor pocket ordering (minimize rapid travel between pockets)
- [ ] Multi-pass step-down for pockets (configurable `step_down` depth for deep pockets)
- [ ] Pocket island detection (raised areas within pockets -- skip them instead of milling into them)
- [ ] Circularity threshold configurable in UI (advanced section, collapsed by default)
- [ ] Drill-to-pocket automatic fallback: circular holes larger than drill tool auto-reclassified as pockets
- [ ] Duplicate hole center deduplication (messy CAD imports)

### P2 (Stretch)

- [ ] Zigzag/raster pocketing alternative strategy
- [ ] Adaptive/trochoidal clearing
- [ ] Estimated machining time per operation
- [ ] Operation summary display in UI ("3 holes, 1 pocket, 4 profiles")
- [ ] Spiral connect between pocket rings (continuous cut, no retract between rings)

### Cut if Behind

If the sprint falls behind schedule, cut these features in order (first cut = least impact):

1. Helical ramp entry -> direct center plunge (simpler, still functional)
2. Nearest-neighbor hole ordering -> natural order (holes processed as encountered)
3. Same-tool optimization -> always emit T/M6 (redundant but correct)
4. Pocket contour-parallel -> single-offset rough pass only (one ring instead of multiple)

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| `geo-buffer` produces degenerate polygons on concave or narrow pockets | Medium | Medium | Check polygon validity (area, vertex count) after each offset. Skip degenerate rings. Log warning for skipped pockets. |
| Circularity threshold misclassifies near-circular shapes (rounded rectangles, tessellation artifacts) | Low | Medium | Default 0.95 is conservative. User-configurable. Dual check (circularity + tool-fit) adds safety layer. |
| World-space transformation errors (forgotten rotation/mirroring on holes) | Medium | High | Holes transform with same `rotate` + `translate` as outer contour. Unit test with rotated piece verifies hole centers match expected world positions. |
| Tool diameter larger than hole/pocket triggers unsafe G-code | Low | Critical | P0 validation rejects oversized tools. No toolpath generated for undersized features. |
| Pocket offset rings fail to connect smoothly | Medium | Low | Simple retract-rapid-plunge between rings for P0. Spiral connect is P2. |
| Multi-tool G-code confuses LinuxCNC tool changer | Low | High | Follow standard LinuxCNC M5/T/M6/S/M3 conventions exactly. Test with LinuxCNC simulator (CAMotics). |
| Backward config compatibility breakage | Medium | Medium | All new serde fields use `#[serde(default)]`. Unit test: deserialize Sprint 007 config JSON into Sprint 008 types. |
| Canned cycle modal state leaks into pocket/profile moves | Low | High | `DrillCycleEnd` always emits `G80`. Emitter resets modal motion state on cycle end. Unit test: G80 appears before any non-drill motion. |

## Security Considerations

- **File system access:** G-code output writes to user-selected directory only (via native save dialog). No arbitrary path construction.
- **Input validation:** All user-configurable parameters (peck depth, step-over, circularity threshold) validated and clamped to safe ranges. Non-finite values rejected before emission.
- **G-code safety:** Z values clamped to prevent cutting above Z=0. Spindle and feed rate values from validated tool database entries. ASCII sanitization on user strings in G-code comments.
- **No network access:** All computation is local. No external service calls.
- **Deterministic output:** Same inputs produce identical G-code. No randomness or timing-dependent behavior.

## Dependencies

- **`thetta-geometry`**: Existing. Extended with new config types, classification types, and `ToolpathMove` variants.
- **`thetta-cam`**: Existing. Extended with three new modules. Depends on `thetta-geometry`.
- **`geo-buffer`**: Existing workspace dependency (used by `toolpath.rs` for polygon offset). Reused for pocket offset ring generation.
- **`geo-types`**: Existing workspace dependency. Used for polygon conversion in offset operations.
- **No new external dependencies.** All polygon offset work uses `geo-buffer`. Classification uses basic geometry (area, perimeter) computed from vertex data.

## Open Questions

1. **Pocket depth vs profile depth.** Should pockets always cut to `gcode_config.cut_depth` (same as profiles, through-cut), or support independent pocket depth? For this sprint, use `cut_depth` for all operations. Independent pocket depth is a future enhancement.

2. **Nested holes (hole within a hole).** If a pocket contour contains its own inner contour (island), the current `Polygon.holes` model stores both as separate entries in the holes list. For this sprint, each hole is classified independently. True island detection (hole-inside-pocket) is deferred to P1. The risk is milling into a raised island, but this geometry is rare in typical 2D CNC work.

3. **Empty pocket after offset.** If a pocket is too small for even one offset ring (pocket width < tool diameter), the pocket is skipped with a log warning. The user can switch to a smaller tool.

4. **Winding normalization.** `Polygon.holes` should be CW-wound (standard for holes). The classifier verifies winding via signed area and normalizes before computing circularity metrics. If winding is wrong (from unusual STEP imports), it is corrected silently.
