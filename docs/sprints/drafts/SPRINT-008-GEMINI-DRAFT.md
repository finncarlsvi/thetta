# Sprint 008: Hole Drilling and Pocket Clearing

## Overview

Sprint 008 completes the 2D CAM pipeline by introducing hole drilling and pocket clearing operations. Building upon the G-code emission, tool database, and profile cutting foundation established in Sprint 007, this sprint enables full CNC production of parts with internal features. The application will analyze the `holes` (inner contours) of each nested piece, classifying them as either drill locations (for circular contours matching a drill bit) or pockets (for non-circular or larger regions requiring clearing).

The implementation extends the `thetta-cam` crate with two new toolpath planners: a drill planner that produces G81 (simple drill) or G83 (peck drill) canned cycles, and a pocket clearing planner that generates contour-parallel (offset) milling paths. The operation sequence is strictly ordered across each sheet to maximize part stability: all drilling operations occur first, followed by pocket clearing, and finally the profile cuts that free the pieces from the stock. 

Users will be able to assign different tools from the database to each operation type via the UI, with the G-code emitter automatically inserting the appropriate tool changes (`T/M6`). This sprint fulfills the original 2D CAM vision described in IDEA.md, transforming Thetta from a simple 2D nesting layout tool into a complete CAM toolpath generation solution for irregular 2D parts.

## Use Cases

1. **UC1: Automatic Hole Classification**
   The application automatically analyzes the internal contours (`holes`) of each nested piece. Small, nearly circular contours are classified for drilling, while irregular or large internal contours are classified for pocket clearing.
2. **UC2: Configure Drilling Operations**
   The user assigns a specific drill bit or endmill from the Tool Database for drilling operations. They can select either simple drilling (G81) or peck drilling (G83) with a configured peck depth for deep holes.
3. **UC3: Configure Pocket Clearing**
   The user assigns a specific endmill for pocketing and sets the step-over percentage (e.g., 50% of the tool diameter) for the contour-parallel clearing passes.
4. **UC4: Operation Ordering**
   When exporting G-code, the orchestrator orders operations sequentially for the entire sheet: all drill cycles are performed first, then all pocket clearing, and finally all profile cuts. This ensures the material remains fully clamped and rigid while internal features are machined.
5. **UC5: Peck Drilling (G83)**
   For thick materials, the system generates G83 peck drilling cycles that repeatedly retract the tool to clear chips during the plunge, reducing tool wear and improving hole quality.
6. **UC6: Contour-Parallel Pocketing**
   Pockets are cleared by generating inward parallel offsets of the pocket boundary. The tool steps inward by the configured step-over distance and machines concentric passes until the entire region is cleared.

## Architecture

### Component Overview

```text
+------------------------------------------------------------------+
|                          thetta-app                                |
|  +-------------------------------------------------------------+ |
|  |          state.rs / panels.rs                               | |
|  |  DrillConfig, PocketConfig (UI inputs)                      | |
|  |  Assign separate tools to Drill, Pocket, Profile            | |
|  +-----------------------------+-------------------------------+ |
|                                |                                 |
+--------------------------------+---------------------------------+
                                 |
                       +---------v---------+
                       |    thetta-cam     |
                       |                   |
                       | +---------------+ |  1. Classify inner
                       | |  generate.rs  | |     contours (Drill vs Pocket)
                       | +-------+-------+ |  2. Group operations
                       |         |         |
          +----------------------+--------------------+
          |                      |                    |
  +-------v--------+     +-------v--------+   +-------v--------+
  |  toolpath.rs   |     |  toolpath.rs   |   |  toolpath.rs   |
  | (Drill Planner)|     | (Pocket Planner|   | (Profile Plan) |
  +-------+--------+     +-------+--------+   +-------+--------+
          |                      |                    |
          +----------------------+--------------------+
                                 |
                         +-------v--------+
                         |    gcode.rs    |
                         | (G81/G83 logic)|
                         +----------------+
```

### Data Flow

1. **Geometry Classification:** During `generate_sheet_gcode`, iterate over each placed piece's `Polygon.holes`. Calculate the circularity and bounding box of each hole. If the hole is highly circular and matches the assigned drill tool diameter (within a tolerance), it becomes a `DrillHole`. Otherwise, it becomes a `Pocket`.
2. **Toolpath Generation:**
   - **Drill:** Generate `ToolpathMove::Drill` points at the center of each `DrillHole`.
   - **Pocket:** Generate `ToolpathMove::LinearCut` and `ArcCut` sequences using repeated inward parallel offsets of the pocket polygon until the polygon collapses.
   - **Profile:** (Existing logic).
3. **Orchestration & Ordering:** 
   - Gather all Drill toolpaths across the sheet.
   - Gather all Pocket toolpaths.
   - Gather all Profile toolpaths.
4. **G-code Emission:**
   - Emit Tool Change for Drill -> Emit Drill Toolpaths (G81/G83).
   - Emit Tool Change for Pocket -> Emit Pocket Toolpaths.
   - Emit Tool Change for Profile -> Emit Profile Toolpaths.

## Implementation

### Phase 1: Types and Classification

**Files:** `crates/thetta-geometry/src/types.rs`, `crates/thetta-cam/src/toolpath.rs`
- Add `DrillConfig` (cycle type, peck depth, tool number).
- Add `PocketConfig` (step-over percentage, tool number).
- Add `ToolpathMove::Drill { to: Point2D, depth: f64, peck: Option<f64>, feed: f64 }`.
- Implement hole classification logic (`is_circular_hole(hole_polygon, tool_diameter)`).

### Phase 2: Drill Planner and G-code Emission

**Files:** `crates/thetta-cam/src/toolpath.rs`, `crates/thetta-cam/src/gcode.rs`
- Implement `plan_drill_toolpath` to output `ToolpathMove::Drill` for detected centers.
- Update `emit_move` in `gcode.rs` to handle `ToolpathMove::Drill`.
- Manage modal state for canned cycles (emit `G81` or `G83`, clear with `G80`).

### Phase 3: Pocket Clearing Planner

**Files:** `crates/thetta-cam/src/toolpath.rs`
- Implement `plan_pocket_toolpath(pocket: &Polygon, tool: &Tool, step_over: f64)`.
- Use `geo-buffer` to iteratively offset the polygon inward by the step-over amount until it results in an empty geometry.
- Connect concentric passes with transition moves (e.g., straight line connecting inner loops).
- Add an initial plunge move in the center of the innermost remaining area.

### Phase 4: App UI and Orchestration

**Files:** `crates/thetta-cam/src/generate.rs`, `crates/thetta-app/src/panels.rs`, `crates/thetta-app/src/state.rs`
- Modify `generate_sheet_gcode` to orchestrate Drill -> Pocket -> Profile.
- Group operations logically to minimize tool changes (all drills on the sheet, then all pockets on the sheet, then all profiles).
- Add UI panels for Drill and Pocket configuration (tool selection dropdowns, G81/G83 toggle, step-over slider).

## Files Summary

| File | Changes |
|------|---------|
| `crates/thetta-geometry/src/types.rs` | Add `DrillConfig`, `PocketConfig`, `ToolpathMove::Drill`. |
| `crates/thetta-cam/src/toolpath.rs` | Add `plan_drill_toolpath`, `plan_pocket_toolpath`, hole classification logic. |
| `crates/thetta-cam/src/gcode.rs` | Add G81/G83 modal emission and `G80` cancellation. |
| `crates/thetta-cam/src/generate.rs` | Update orchestration for sheet-wide Drill -> Pocket -> Profile ordering. |
| `crates/thetta-app/src/state.rs` | Add drill and pocket configs to `AppState`. |
| `crates/thetta-app/src/panels.rs` | Add Drill and Pocket UI configuration sections. |
| `crates/thetta-cam/tests/toolpath_tests.rs` | Add unit tests for hole classification, drill planning, and pocket offsetting. |

## Definition of Done

### P0 -- Must Ship
- [ ] `DrillConfig` and `PocketConfig` added to `thetta-geometry`.
- [ ] Inner contours are successfully classified into drills and pockets based on circularity.
- [ ] Drill planner outputs G81/G83 canned cycles.
- [ ] `G80` is emitted to cancel drill cycles.
- [ ] Pocket planner uses inward parallel offsetting with configurable step-over.
- [ ] `generate_sheet_gcode` strictly orders operations: Drill -> Pocket -> Profile.
- [ ] G-code correctly handles multiple tool changes (`T<n> M6`).
- [ ] UI panels allow tool assignment for all three operations.

### P1 -- Should Ship
- [ ] Pocket transitions use smooth ramping or arcs instead of direct linear cuts between offset rings.
- [ ] Fallback: if a drill hole is larger than the drill tool diameter, automatically convert it to a pocket operation.
- [ ] Zig-zag (raster) pocketing alternative for large rectangular areas.

### P2 -- Stretch
- [ ] Finish pass configuration for pockets (leave 0.5mm for a final full-depth smoothing pass).
- [ ] User override: manual UI to re-classify a specific hole as a drill or pocket.

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Pocket offsetting fails on complex concave inner holes | Medium | High | Rely on `geo-buffer` robust parallel offset. Discard degenerate sliver geometries during the inward step process. |
| Drill classification false positives (classifying an irregular hole as a drill) | Low | High | Use a strict circularity threshold (e.g., area / bounding box matches circle, vertex distances from centroid are uniform). Default to pocket if uncertain. |
| Plunging into material during pocketing without ramping | High | Medium | For P0, plunge directly into the pocket center. For P1, add helical plunge or ramped entry to reduce endmill stress. |
| Redundant tool changes | Low | Low | Orchestrator groups all operations of the same type together, ensuring at most 3 tool changes per sheet. |

## Security Considerations
- **No new external system dependencies:** Same security footprint as Sprint 007. 
- **Tool safety:** CNC machines can be damaged by improper feeds/speeds. G-code bounds checking and safe-Z usage between all operations remain strictly enforced.

## Dependencies
- Relies on `geo-buffer` (already integrated in Sprint 007) for inward polygon offset.
- `thetta-geometry` and `thetta-cam` crates require synchronous updates to support the new `ToolpathMove` variations.

## Open Questions
1. **Peck vs Simple Drilling:** Should peck depth default to a fixed percentage of the tool diameter, or be strictly user-configured? 
2. **Pocket Plunge Strategy:** Should the pocketing toolpath plunge in the pre-drilled holes (if any exist within the pocket) to save plunge time, or simply plunge at the center of the innermost offset?
3. **Disabling Operations:** Can users entirely disable pocketing or drilling for a run if they only want the profiles cut? (Assumed yes, via UI toggles).
4. **Tool Diameter vs Hole Size:** If a hole perfectly matches the drill bit diameter, it's drilled. What tolerance is acceptable? (e.g., `±0.1mm`).