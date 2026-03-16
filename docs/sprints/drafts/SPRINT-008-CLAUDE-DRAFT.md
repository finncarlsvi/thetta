# Sprint 008: Hole Drilling and Pocket Clearing

## Overview

Complete the 2D CAM pipeline by adding hole drilling (G81/G83 canned cycles) and pocket clearing (contour-parallel offset) to the existing `thetta-cam` crate. After Sprint 007 established profile cuts with tool radius compensation, tab-aware toolpaths, and the G-code emitter, the remaining two CAM operations from IDEA.md are hole drilling and pocket clearing. Both operate on the inner contours (`Polygon.holes`) that are already parsed from STEP import but currently unused in the CAM pipeline. After this sprint, Thetta can generate complete G-code programs with all three 2D operations for CNC router production.

The key challenge is classifying each hole contour: circular or near-circular holes become drill operations (single-point canned cycles), while non-circular or large enclosed regions become pocket operations (area-clearing toolpaths). The drill path is straightforward — detect center and depth, emit G81 or G83. The pocket path is more involved: generate concentric inward offsets of the pocket boundary at a configurable step-over, connect them into a continuous toolpath, and optionally add a finish pass along the pocket wall. Both operations produce `ToolpathMove` sequences that feed into the existing G-code emitter.

Operation ordering follows IDEA.md: drill first, then pockets, then profile cuts. This ordering ensures pieces remain attached to the sheet (profile cuts with tabs are last) while holes and pockets are machined into still-rigid stock. Each operation type can use a different tool from the tool database, with automatic tool changes (T/M6) emitted between operation groups. The per-sheet orchestration in `generate.rs` is extended from single-operation (profile only) to multi-operation with correct ordering and tool change management.

**Scope:** Hole classification (drill vs pocket). Drill cycle planning and G81/G83 emission. Pocket toolpath planning (contour-parallel offset strategy) and emission. Multi-operation orchestration with tool changes. Per-operation tool assignment in `GcodeConfig`. Drill and pocket config UI panels. All three operations in per-sheet G-code output.
**Prerequisites:** Sprint 007 complete. `thetta-cam` crate with profile toolpath planner, G-code emitter, tool DB, and `generate.rs` orchestration. `Polygon.holes` populated from STEP import.
**Deferred:** Zigzag/raster pocketing strategy, multi-pass step-down for deep pockets, helical ramp entry for pockets, pocket island detection (holes within holes), tool wear compensation, in-app toolpath visualization.

## Use Cases

### UC1: Drill Through-Holes
User imports a STEP file with parts containing circular through-holes. After nesting, the CAM pipeline detects circular holes, determines their center coordinates, and generates G81 (simple drill) or G83 (peck drill) canned cycles. The user can assign a drill bit from the tool database. The G-code drills all holes before any other operation.

### UC2: Clear Pockets
User imports parts with non-circular enclosed pockets (slots, rectangular cutouts, irregular shapes). The CAM pipeline classifies these as pockets and generates contour-parallel clearing toolpaths that step inward from the pocket boundary. The user assigns an endmill for pocket clearing. Pocket clearing occurs after drilling and before profile cuts.

### UC3: Mixed Operations on One Part
A part has two 5mm drill holes, one rectangular slot pocket, and an outer contour. The G-code output drills both holes (G83), clears the slot pocket (G1 linear moves with step-over), then cuts the outer profile with tabs. Tool changes occur between operation groups if different tools are assigned.

### UC4: Assign Per-Operation Tools
User assigns T1 (6mm drill) to hole drilling, T2 (6mm endmill) to pocket clearing, and T3 (6mm endmill) to profile cuts. The G-code emits `T1 M6` before drill cycles, `T2 M6` before pocket clearing, and `T3 M6` before profiles. If the same tool is used for multiple operations, tool changes are still emitted for clarity and LinuxCNC compatibility.

### UC5: Configure Drill Parameters
User configures peck depth for G83 peck drilling (e.g., 5mm per peck for an 18mm-deep hole). Simple drilling (G81) is used when peck depth is zero or exceeds material thickness. Retract height between pecks is the safe-Z value from `GcodeConfig`.

### UC6: Configure Pocket Parameters
User sets step-over as a percentage of tool diameter (default 50%). An optional finish pass re-traces the pocket boundary at full depth for a clean wall finish. These parameters are in `PocketConfig`.

### UC7: Pieces with No Holes
Parts with no inner contours produce only profile cuts. The drill and pocket phases are empty — no tool changes are emitted for unused operations. The G-code is identical to Sprint 007 output for hole-free parts.

### UC8: Disable Operations
User can disable drilling or pocket clearing entirely via toggle in the UI. Disabled operations are skipped during G-code generation. Profile cuts cannot be disabled (they are the primary operation).

### UC9: Multi-Sheet with All Operations
For multi-sheet nesting results, each sheet's G-code file includes all three operation types in the correct order: drill all holes on the sheet, clear all pockets, then cut all profiles. Each sheet file is self-contained.

### UC10: Circularity Threshold
User can adjust the circularity threshold that distinguishes drill holes from pockets. Default: 0.95 (where 1.0 is a perfect circle). A 4-point approximation of a circle (~0.90 circularity) would fall below the default threshold and be treated as a pocket.

## Architecture

### Component Overview

```
+-------------------------------------------------------------------+
|                          thetta-app                                |
|  +-----------+  +----------+  +---------------------------------+ |
|  | panels.rs |  |viewport.rs| |          state.rs               | |
|  | Drill cfg |  | (unchanged|  | DrillConfig, PocketConfig      | |
|  | Pocket cfg|  |  this     |  | Per-operation tool assignment   | |
|  | Operation |  |  sprint)  |  | Multi-op G-code generation     | |
|  | toggles   |  |          |  | Background thread + channel    | |
|  +-----+-----+  +----------+  +---------------+-----------------+ |
|        |                                       |                   |
+--------+---------------------------------------+-------------------+
         |                                       |
         |                       +---------------v------------------+
         |                       |          thetta-cam               |
         |                       |  +----------------------------+  |
         |                       |  |    hole_classify.rs (NEW)  |  |
         |                       |  | classify_holes()           |  |
         |                       |  | Circularity + center detect|  |
         |                       |  +----------------------------+  |
         |                       |  +----------------------------+  |
         |                       |  |    drill.rs (NEW)          |  |
         |                       |  | plan_drill_toolpath()      |  |
         |                       |  | G81/G83 cycle planning     |  |
         |                       |  +----------------------------+  |
         |                       |  +----------------------------+  |
         |                       |  |    pocket.rs (NEW)         |  |
         |                       |  | plan_pocket_toolpath()     |  |
         |                       |  | Contour-parallel offsets   |  |
         |                       |  | Optional finish pass       |  |
         |                       |  +----------------------------+  |
         |                       |  +----------------------------+  |
         |                       |  |    toolpath.rs (existing)  |  |
         |                       |  | plan_profile_toolpath()    |  |
         |                       |  | offset_contour() (reused)  |  |
         |                       |  +----------------------------+  |
         |                       |  +----------------------------+  |
         |                       |  |    gcode.rs (extended)     |  |
         |                       |  | emit_gcode() extended      |  |
         |                       |  | + emit_drill_cycles()      |  |
         |                       |  | + multi-tool preamble      |  |
         |                       |  +----------------------------+  |
         |                       |  +----------------------------+  |
         |                       |  |    generate.rs (extended)  |  |
         |                       |  | generate_sheet_gcode()     |  |
         |                       |  | Multi-op orchestration:    |  |
         |                       |  | drill -> pocket -> profile |  |
         |                       |  | Per-operation tool changes |  |
         |                       |  +----------------------------+  |
         |                       +------------------+---------------+
         |                                          |
         |                       +------------------v---------------+
         |                       |        thetta-geometry            |
         +-------------------+-->|  types.rs                         |
                                 |  + DrillConfig, PocketConfig      |
                                 |  + DrillCycle (new ToolpathMove)  |
                                 |  + OperationConfig (per-op tools) |
                                 |  + (existing) Polygon, Tool, etc  |
                                 +----------------------------------+
```

### New Module: `hole_classify.rs`

Analyzes `Polygon.holes` to classify each inner contour as either a drill hole or a pocket. Classification uses circularity: `4π × area / perimeter²`. A perfect circle has circularity 1.0. Contours above the configurable threshold (default 0.95) are classified as drill holes; below as pockets.

For drill holes, the classifier computes the center (centroid of the contour) and diameter (average distance from center to vertices × 2). These are returned in a `DrillHole` struct.

For pockets, the original contour vertices are returned as a `PocketRegion` struct for toolpath planning.

```rust
pub struct DrillHole {
    pub center: Point2D,
    pub diameter: f64,
}

pub struct PocketRegion {
    pub contour: Vec<Point2D>,  // CW winding from Polygon.holes
}

pub enum HoleClassification {
    Drill(DrillHole),
    Pocket(PocketRegion),
}

pub fn classify_holes(
    polygon: &Polygon,
    circularity_threshold: f64,
) -> Vec<HoleClassification>
```

### New Module: `drill.rs`

Plans drill toolpaths for classified drill holes. Each drill hole produces a sequence: rapid to XY above hole → canned drill cycle. The `ToolpathMove` enum gains a new `DrillCycle` variant:

```rust
pub enum ToolpathMove {
    // ... existing variants ...
    DrillCycle {
        position: Point2D,   // XY center
        depth: f64,          // total depth (positive, from Z=0 down)
        peck_depth: f64,     // per-peck increment (0 = simple G81)
        retract_z: f64,      // retract height between pecks
        feed: f64,           // plunge feed rate
    },
}
```

The `DrillCycle` variant encodes enough information for the G-code emitter to choose between G81 (when `peck_depth == 0` or `peck_depth >= depth`) and G83 (when `peck_depth < depth`).

```rust
pub fn plan_drill_toolpath(
    holes: &[DrillHole],
    tool: &Tool,
    drill_config: &DrillConfig,
    gcode_config: &GcodeConfig,
) -> Toolpath
```

### New Module: `pocket.rs`

Plans pocket clearing toolpaths using contour-parallel (offset inward) strategy. For each pocket contour:

1. Convert CW hole winding to CCW (pocket boundary viewed from inside).
2. Compute effective cut width: `tool.diameter * step_over_fraction`.
3. Generate concentric inward offsets using `offset_contour()` (from `toolpath.rs`, which wraps `geo-buffer`). First offset is inward by `tool.diameter / 2` (tool radius compensation). Subsequent offsets step inward by `step_over * tool.diameter`.
4. Stop when the offset polygon degenerates (area below threshold or fewer than 3 vertices).
5. Connect offset rings into a continuous toolpath: cut the outermost ring, step inward to the next ring with a connecting move, repeat.
6. Optional finish pass: after clearing, re-trace the innermost boundary (first offset) at full depth for a clean wall.

```rust
pub fn plan_pocket_toolpath(
    pocket: &PocketRegion,
    tool: &Tool,
    pocket_config: &PocketConfig,
    gcode_config: &GcodeConfig,
) -> Toolpath
```

### Extended: `gcode.rs`

The G-code emitter gains a new handler for `ToolpathMove::DrillCycle`:

```
G81 X{x} Y{y} Z{-depth} R{retract_z} F{feed}    (simple drill)
G83 X{x} Y{y} Z{-depth} R{retract_z} Q{peck}     (peck drill)
G80                                                 (cancel canned cycle)
```

Drill cycles use modal canned cycle syntax: the first hole sets the cycle, subsequent holes only need XY coordinates. `G80` cancels the canned cycle mode after the last hole.

The emitter also gains multi-tool support: when the tool changes between operation groups, it emits:
```
M5          (spindle off)
T{n} M6    (tool change)
S{rpm} M3  (spindle on)
```

### Extended: `generate.rs`

The orchestrator changes from single-operation to multi-operation per sheet:

```
For each Sheet:
  1. Classify holes for all placed pieces
  2. Group operations: drill holes, pocket regions, profile contours

  3. DRILL PHASE (if drill tool assigned and holes exist):
     a. Tool change to drill tool
     b. For each piece: emit drill cycles for all drill holes
     c. Cancel canned cycle (G80)

  4. POCKET PHASE (if pocket tool assigned and pockets exist):
     a. Tool change to pocket tool
     b. For each piece: emit pocket toolpaths for all pocket regions

  5. PROFILE PHASE (if profile tool assigned):
     a. Tool change to profile tool
     b. For each piece: emit profile toolpath (existing logic)

  6. Postamble
```

### Data Flow

```
NestingResult + ToolTable + GcodeConfig + DrillConfig + PocketConfig + ProfileCutConfig
        |
        v
For each Sheet (sheet_index):
  |
  +-- 1. For each Placement:
  |       |
  |       +-- a. Look up Piece polygon
  |       +-- b. Apply placement transform -> world-space polygon (with holes)
  |       +-- c. classify_holes(polygon, circularity_threshold)
  |       +-- d. Collect DrillHoles and PocketRegions
  |
  +-- 2. DRILL phase:
  |       +-- plan_drill_toolpath(all_drill_holes, drill_tool, drill_config, gcode_config)
  |       +-- Returns Vec<ToolpathMove> with DrillCycle variants
  |
  +-- 3. POCKET phase:
  |       +-- For each pocket: plan_pocket_toolpath(pocket, pocket_tool, pocket_config, gcode_config)
  |       +-- Returns Vec<Toolpath> with LinearCut/Rapid/Retract/Plunge moves
  |
  +-- 4. PROFILE phase (existing):
  |       +-- For each piece: plan_profile_toolpath(contour, tabs, profile_tool, ...)
  |       +-- Returns Vec<Toolpath> (unchanged from Sprint 007)
  |
  +-- 5. emit_multi_op_gcode(drill_paths, pocket_paths, profile_paths, tools, config)
  |       +-- Preamble (%, G21, G90, G17, coord system)
  |       +-- Drill section: tool change + drill cycles + G80
  |       +-- Pocket section: tool change + pocket moves
  |       +-- Profile section: tool change + profile moves
  |       +-- Postamble (M5, retract, home, M2, %)
  |
  +-- 6. Write to {output_dir}/sheet_{N}.ngc
```

### New Types in `thetta-geometry/types.rs`

```rust
/// Configuration for drill operations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DrillConfig {
    /// Peck depth per cycle in mm. 0 = simple drill (G81).
    pub peck_depth: f64,
    /// Tool number assigned to drill operations.
    pub tool_number: Option<u32>,
    /// Enable/disable drill operations.
    pub enabled: bool,
    /// Circularity threshold for classifying holes as drillable (0.0-1.0).
    pub circularity_threshold: f64,
}

/// Configuration for pocket clearing operations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PocketConfig {
    /// Step-over as fraction of tool diameter (0.0-1.0, default 0.5).
    pub step_over: f64,
    /// Tool number assigned to pocket operations.
    pub tool_number: Option<u32>,
    /// Enable/disable pocket operations.
    pub enabled: bool,
    /// Whether to add a finish pass along the pocket wall.
    pub finish_pass: bool,
}
```

### Design Decisions

1. **Circularity-based classification.** Simple, geometry-only heuristic that works for the common case (round drilled holes vs shaped pockets). No STEP semantic info needed — works with polygon data alone. Configurable threshold lets users tune for their geometry.

2. **`DrillCycle` as a `ToolpathMove` variant.** Canned cycles are semantically different from linear moves. A dedicated variant lets the emitter choose G81 vs G83 and handle modal canned cycle syntax correctly, rather than encoding drill motion as a series of plunge/retract moves that wouldn't benefit from the controller's built-in cycle logic.

3. **Contour-parallel pocketing (not zigzag).** Contour-parallel produces better surface finish on pocket walls and naturally respects pocket shape. Zigzag requires complex clipping against arbitrary pocket boundaries. Contour-parallel reuses the existing `offset_contour()` from `toolpath.rs` (which wraps `geo-buffer`), minimizing new geometric code.

4. **Per-operation tool assignment.** Each operation type (drill, pocket, profile) has its own `tool_number` field. This is the natural CNC workflow: drill bits for holes, endmills for pockets/profiles. The emitter handles tool changes between operation groups.

5. **Operation ordering: drill → pocket → profile.** Matches IDEA.md requirement. Drilling first is safest (point contact, least lateral force). Pockets second (enclosed, stock supports tool). Profiles last because they sever the part from the sheet (tabs keep it attached but rigidity decreases).

6. **Reuse of `offset_contour()` for pocket offsets.** The polygon offset function from `toolpath.rs` (wrapping `geo-buffer`) is already tested and handles the CCW→offset correctly. Pocket clearing simply calls it repeatedly with increasing inward offset until the polygon degenerates.

7. **Pocket entry via plunge.** Simple vertical plunge at the pocket center for the first ring, then step-over to subsequent rings. Helical ramp entry is deferred — it adds complexity and is only needed for harder materials or larger endmills.

8. **G80 canned cycle cancel.** Explicit G80 after the last drill hole prevents accidental drill cycles on subsequent rapid moves. Standard LinuxCNC practice.

9. **Multi-tool G-code structure.** Each operation group gets its own tool change block. Even if drill and pocket use the same tool, the tool change is emitted for clarity. This is a conservative choice that works on all LinuxCNC setups.

10. **Hole winding conversion.** `Polygon.holes` are CW-wound (standard for holes in the polygon model). For pocket clearing, the contour is reversed to CCW so that `offset_contour()` with negative distance produces inward offsets correctly.

## Implementation

### Phase 1: Types + Hole Classification (~20%)

**Goal:** Add drill and pocket config types, implement hole classification logic.

#### Task 1.1: New Config Types

**File:** `crates/thetta-geometry/src/types.rs`

Add `DrillConfig`, `PocketConfig` structs with defaults and validation. Add `DrillCycle` variant to `ToolpathMove`. Extend `GcodeConfig` with per-operation tool numbers or add a wrapper `CamConfig` that holds all three operation configs.

- Add `DrillConfig` with `peck_depth`, `tool_number`, `enabled`, `circularity_threshold`
- Add `PocketConfig` with `step_over`, `tool_number`, `enabled`, `finish_pass`
- Add `ToolpathMove::DrillCycle { position, depth, peck_depth, retract_z, feed }`
- Add `Default` impls with sensible values (peck_depth=5mm, step_over=0.5, circularity_threshold=0.95)
- Add `validate()` methods

#### Task 1.2: Hole Classification Module

**File:** `crates/thetta-cam/src/hole_classify.rs` (new)

- Implement `circularity(contour: &[Point2D]) -> f64` using `4π × area / perimeter²`
- Implement `contour_center(contour: &[Point2D]) -> Point2D` (centroid)
- Implement `contour_diameter(contour: &[Point2D], center: Point2D) -> f64` (2 × avg distance from center)
- Implement `classify_holes(polygon: &Polygon, threshold: f64) -> Vec<HoleClassification>`
- Unit tests: perfect circle polygon → Drill, rectangle → Pocket, near-circle (high vertex count) → Drill

#### Task 1.3: Register New Module

**File:** `crates/thetta-cam/src/lib.rs`

- Add `pub mod hole_classify;`
- Add `pub mod drill;`
- Add `pub mod pocket;`

### Phase 2: Drill Cycle Planning + Emission (~25%)

**Goal:** Generate G81/G83 canned cycles from classified drill holes.

#### Task 2.1: Drill Toolpath Planner

**File:** `crates/thetta-cam/src/drill.rs` (new)

- Implement `plan_drill_toolpath(holes, tool, drill_config, gcode_config) -> Toolpath`
- For each `DrillHole`: emit `Rapid` to XY at safe-Z, then `DrillCycle` with computed parameters
- Peck depth selection: if `drill_config.peck_depth > 0` and `peck_depth < gcode_config.cut_depth`, use G83 (peck); else G81 (simple)
- Sort holes by position (nearest-neighbor or simple X-then-Y) to minimize rapid travel
- Unit tests: single hole produces Rapid + DrillCycle, multiple holes sorted, peck vs simple selection

#### Task 2.2: G-code Drill Cycle Emission

**File:** `crates/thetta-cam/src/gcode.rs`

- Add handler for `ToolpathMove::DrillCycle` in `emit_move()`
- Emit `G81 X{x} Y{y} Z{z} R{r} F{f}` for simple drill or `G83 X{x} Y{y} Z{z} R{r} Q{q} F{f}` for peck
- Track modal canned cycle state to avoid repeating G81/G83 on subsequent holes
- Emit `G80` at end of drill section (called from `generate.rs`, not per-hole)
- Unit tests: verify G81 format, G83 format with Q word, modal behavior (second hole omits G83 code)

### Phase 3: Pocket Toolpath Planning (~30%)

**Goal:** Generate contour-parallel clearing toolpaths for pockets.

#### Task 3.1: Pocket Toolpath Planner

**File:** `crates/thetta-cam/src/pocket.rs` (new)

- Implement `plan_pocket_toolpath(pocket, tool, pocket_config, gcode_config) -> Toolpath`
- Reverse CW pocket contour to CCW
- Compute first offset: inward by `tool.diameter / 2` (tool radius compensation for pocket wall)
- Compute subsequent offsets: inward by `step_over * tool.diameter` from previous
- Stop when offset polygon has area < `tool.diameter²` or fewer than 3 vertices
- For each offset ring (outermost first):
  - Rapid to ring start at safe-Z
  - Plunge to cut depth
  - LinearCut along ring contour
  - Step inward: rapid or linear to next ring start
- Optional finish pass: re-trace first offset ring at full depth
- Retract to safe-Z after last ring
- Unit tests: rectangular pocket produces multiple rings, small pocket (smaller than tool) produces empty toolpath, finish pass adds extra ring traversal

#### Task 3.2: Pocket Entry Strategy

**File:** `crates/thetta-cam/src/pocket.rs`

- Entry point: center of innermost offset ring (or centroid of pocket if no inner ring fits)
- Plunge vertically at center (simple, safe for wood/composite)
- Spiral outward from center to first ring, then follow rings outward
- Alternative: start from outermost ring and work inward (simpler connectivity)
- Decision: work from outermost ring inward is simpler and matches conventional practice. The tool plunges at the start of each ring and cuts along it. Between rings, a retract-rapid-plunge sequence repositions.

### Phase 4: Multi-Operation Orchestration (~15%)

**Goal:** Extend `generate.rs` to orchestrate all three operations with correct ordering and tool changes.

#### Task 4.1: Extend `generate.rs`

**File:** `crates/thetta-cam/src/generate.rs`

- Refactor `SheetGcodeInput` to include `DrillConfig`, `PocketConfig`, per-operation tool references
- New struct `SheetCamInput` (or extend existing) with all three configs and tools
- Implement classification → drill/pocket/profile grouping per placement
- Call `plan_drill_toolpath()` for all drill holes on the sheet
- Call `plan_pocket_toolpath()` for all pockets on the sheet
- Call `plan_profile_toolpath()` for all profiles (existing)
- Pass grouped toolpaths to extended emitter

#### Task 4.2: Multi-Tool G-code Emission

**File:** `crates/thetta-cam/src/gcode.rs`

- New function `emit_multi_op_gcode(drill_paths, pocket_paths, profile_paths, tools, config) -> String`
- Preamble: %, G21, G90, G17, coordinate system
- For each operation group with assigned tool:
  - Tool change: `M5` (spindle off if not first), `T{n} M6`, `S{rpm} M3`
  - Emit operation toolpaths
  - Drill: emit drill cycles + G80
  - Pocket: emit pocket moves
  - Profile: emit profile moves (existing logic)
- Postamble: M5, retract, home, M2, %
- Unit tests: verify tool change ordering, verify G80 after drills, verify empty operation groups are skipped

#### Task 4.3: Update `generate_all_sheets()`

**File:** `crates/thetta-cam/src/generate.rs`

- Update signature to accept all three configs and per-operation tools
- Each sheet produces complete multi-operation G-code
- Empty sheets still produce no output

### Phase 5: App Integration (~10%)

**Goal:** Wire drill and pocket configuration into the UI and state management.

#### Task 5.1: State Extension

**File:** `crates/thetta-app/src/state.rs`

- Add `DrillConfig` and `PocketConfig` to `AppState`
- Extend G-code generation trigger to pass all configs
- Update background thread to call multi-operation `generate_sheet_gcode()`

#### Task 5.2: UI Panels

**File:** `crates/thetta-app/src/panels.rs`

- Add drill configuration section in CAM panel:
  - Enable/disable toggle
  - Tool selection dropdown (from tool table)
  - Peck depth slider/input
  - Circularity threshold slider (advanced, collapsed by default)
- Add pocket configuration section in CAM panel:
  - Enable/disable toggle
  - Tool selection dropdown
  - Step-over percentage slider
  - Finish pass checkbox
- Show operation order summary: "Drill (T1) → Pocket (T2) → Profile (T3)"

## Files Summary

| File | Action | Description |
|------|--------|-------------|
| `crates/thetta-geometry/src/types.rs` | Modify | Add `DrillConfig`, `PocketConfig`, `ToolpathMove::DrillCycle` |
| `crates/thetta-cam/src/lib.rs` | Modify | Register `hole_classify`, `drill`, `pocket` modules |
| `crates/thetta-cam/src/hole_classify.rs` | New | Hole circularity classification (drill vs pocket) |
| `crates/thetta-cam/src/drill.rs` | New | Drill cycle toolpath planning (G81/G83) |
| `crates/thetta-cam/src/pocket.rs` | New | Contour-parallel pocket clearing toolpath |
| `crates/thetta-cam/src/gcode.rs` | Modify | Add `DrillCycle` emission (G81/G83/G80), multi-tool support |
| `crates/thetta-cam/src/generate.rs` | Modify | Multi-operation orchestration (drill → pocket → profile) |
| `crates/thetta-app/src/state.rs` | Modify | Add `DrillConfig`, `PocketConfig` to state, update generation |
| `crates/thetta-app/src/panels.rs` | Modify | Add drill and pocket config UI panels |

## Definition of Done

### P0 (Must Have)

- [ ] `classify_holes()` correctly distinguishes circular holes from non-circular pockets using circularity metric
- [ ] Drill holes produce `DrillCycle` toolpath moves with correct center and depth
- [ ] G-code emitter outputs valid G81 (simple drill) and G83 (peck drill) canned cycles
- [ ] G80 cancels canned cycle mode after drill section
- [ ] Pocket clearing produces contour-parallel toolpaths with configurable step-over
- [ ] Pocket toolpaths respect tool radius compensation (first offset = tool radius)
- [ ] Operation ordering in G-code: drill → pocket → profile (IDEA.md requirement)
- [ ] Tool changes (T/M6) emitted between operation groups when tools differ
- [ ] Multi-sheet export includes all three operations per sheet
- [ ] `DrillConfig` and `PocketConfig` types with validation
- [ ] Unit tests for hole classification, drill planning, pocket planning, and operation ordering
- [ ] `cargo test --workspace` passes

### P1 (Should Have)

- [ ] Nearest-neighbor drill hole ordering to minimize rapid travel
- [ ] Pocket finish pass option (re-trace pocket boundary after clearing)
- [ ] UI panels for drill and pocket configuration with tool assignment dropdowns
- [ ] Enable/disable toggles for drill and pocket operations
- [ ] Pieces with no holes produce profile-only G-code (no spurious tool changes)
- [ ] Circularity threshold configurable in UI

### P2 (Nice to Have)

- [ ] Pocket entry at centroid with connecting path to first ring (vs retract-rapid-plunge per ring)
- [ ] Spiral connect between pocket rings (continuous cut, no retract between rings)
- [ ] Drill cycle modal optimization (omit G83 on second+ hole when parameters unchanged)
- [ ] Operation summary display in UI ("3 holes, 1 pocket, 4 profiles")

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| `geo-buffer` produces degenerate polygons for small or thin pockets | Medium | Medium | Check polygon validity (area, vertex count) after each offset. Skip degenerate rings gracefully. |
| Circularity threshold misclassifies near-circular pockets as drill holes | Low | Medium | Default threshold 0.95 is conservative. User-configurable. Warn if classified drill hole diameter doesn't match available drill bits. |
| Pocket offset rings fail to connect smoothly | Medium | Low | Simple retract-rapid-plunge between rings. Deferred: spiral connect. |
| Multi-tool G-code confuses LinuxCNC tool changer | Low | High | Follow standard LinuxCNC M6 conventions exactly. Test with LinuxCNC simulator. |
| Large pockets with many offset rings produce very long G-code | Low | Low | Step-over limits ring count naturally. A 100mm pocket with 50% step-over on a 6mm tool generates ~17 rings — manageable. |
| Polygon winding assumptions break on unusual STEP imports | Low | Medium | Verify winding with signed area before classification. Normalize to expected winding. |

## Security Considerations

- **File system access:** G-code output writes to user-selected directory only (via native save dialog). No arbitrary path construction.
- **Tool table persistence:** Same JSON file as Sprint 007. No new file system access patterns.
- **No network access:** All computation is local. No external service calls.
- **Input validation:** All user-configurable parameters (peck depth, step-over, circularity threshold) are validated and clamped to safe ranges before use.
- **G-code safety:** Z values clamped to prevent cutting above Z=0 (above material surface). Spindle and feed rate values come from validated tool database entries.

## Dependencies

- **`thetta-geometry`**: Existing. Extended with new config types and `ToolpathMove` variant.
- **`thetta-cam`**: Existing. Extended with three new modules. Depends on `thetta-geometry`.
- **`geo-buffer`**: Existing workspace dependency (used by `toolpath.rs`). Reused for pocket offset generation.
- **`geo-types`**: Existing workspace dependency. Used for polygon conversion in offset operations.
- **No new external dependencies.** All polygon offset work uses the existing `geo-buffer` crate. Classification uses basic geometry (area, perimeter) already available in `polygon_ops`.

## Open Questions

1. **Hole classification edge cases.** What about holes that are nearly circular but have a flat (D-shaped)? Should there be a minimum diameter threshold below which holes are always drilled regardless of circularity? Recommendation: add a `min_drill_diameter` field to `DrillConfig` (default 1.0mm) — anything smaller than the smallest available drill bit is skipped.

2. **Peck drill vs simple drill default.** Should peck drilling (G83) be the default, or should simple drilling (G81) be default with peck as opt-in? Recommendation: default to peck drilling with `peck_depth = 5.0mm` — it's safer for through-holes (chip evacuation) and works on all machines.

3. **Pocket depth vs cut depth.** Should pockets always be cut to the same `cut_depth` as profiles (through-cut), or should pockets support independent depth? For this sprint, use `gcode_config.cut_depth` for all operations. Independent pocket depth is a future enhancement.

4. **Pocket ring connection strategy.** Retract-rapid-plunge between offset rings is simple but slow. Spiral connect (step inward during the last segment of each ring) is faster but more complex. Recommendation: start with retract-rapid-plunge for P0, spiral connect as P2.

5. **Empty pocket handling.** If a pocket is too small for even one offset ring (pocket width < tool diameter), should we skip it silently or warn the user? Recommendation: skip with a log warning. The user can switch to a smaller tool.

6. **Operation ordering scope.** Should ordering be per-sheet (drill ALL holes on sheet, then ALL pockets, then ALL profiles) or per-piece (drill piece 1 holes, pocket piece 1, profile piece 1, then piece 2...)? IDEA.md says "drill holes first, then pockets, then profile cuts" — this reads as per-sheet global ordering. Per-sheet is also more efficient (fewer tool changes). Recommendation: per-sheet.
