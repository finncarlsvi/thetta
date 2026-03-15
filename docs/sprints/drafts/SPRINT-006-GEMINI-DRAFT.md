# Sprint 006: Automatic Tab Placement for CNC Routing

## Overview
This sprint introduces automatic tab placement (also known as holding tabs or bridges) for CNC-routed parts. In CNC routing, especially for wood and plastic, parts must remain attached to the surrounding material during cutting to prevent them from moving, which could damage the part or the cutting tool. Tabs are small uncut sections of material that securely hold the nested parts in place.

The tab placement process will be implemented as an independent post-processing step that runs after the core nesting optimization completes. It analyzes the outer contour of each placed piece and automatically determines the optimal number, size, and location of tabs based on part perimeter, material properties, and edge geometry.

Our approach adapts the algorithm proposed in `RESEARCH-005`, favoring straight edges and convex curves while avoiding corners. This sprint focuses strictly on per-part tab placement. Coordinating shared tabs between adjacent parts is a complex challenge that is explicitly deferred to a future phase.

## Use Cases
1. **Automatic Tab Generation**: After a user nests a set of parts, the system automatically generates tabs for each part based on its perimeter, intelligently skipping parts that are too small.
2. **Material-Aware Sizing**: Tab dimensions (width and height) automatically scale based on user-defined material thickness and type (e.g., soft wood, hard wood, plastic).
3. **Tab Configuration UI**: Users can tweak default tab parameters (e.g., target distance, minimum/maximum counts, exclusion zones) via a new UI panel.
4. **Visual Verification**: Users can see the generated tabs visually represented on the nested parts in the 2D viewport to verify placement quality.
5. **Export Inclusion**: Tab locations and dimensions are included in the output format (as JSON metadata or visual STEP artifacts) for downstream CAM processing.

## Architecture

### Component Changes
- **`thetta-geometry`**: New types `Tab`, `TabConfig`, `MaterialType`, and `TabResult` will be added to `types.rs`. A new module `tabs.rs` will encapsulate the tab placement algorithm (perimeter calculation, edge scoring, and distribution). Utilities for edge iteration and point interpolation will be added to `polygon_ops.rs`.
- **`thetta-nesting`**: The core optimization pipeline remains untouched. A post-processing step will run after nesting to compute tabs for the `NestingResult`.
- **`thetta-app`**: `AppState` will hold the active `TabConfig`. `panels.rs` will expose tab settings. `viewport.rs` will draw tabs as distinct markers along piece contours.
- **`thetta-step`**: Export functions will serialize tab data to JSON and potentially represent them in the exported STEP model as metadata or visual bounding boxes.

### Data Flow
```text
[NestingResult] + [TabConfig]
       |
       v
+-------------------------------+
|   Tab Placement Algorithm     |
| (crates/thetta-geometry/tabs) |
+-------------------------------+
       |
       v
[NestingResult with Tabs]
       |
       +--> Viewport Rendering (crates/thetta-app/viewport)
       |
       +--> STEP/JSON Export (crates/thetta-step/export)
```

## Implementation

### Phase 1: Core Geometry and Types
- **Task 1.1**: Define `Tab`, `TabConfig`, `MaterialType` in `crates/thetta-geometry/src/types.rs`.
- **Task 1.2**: Add edge utilities to `crates/thetta-geometry/src/polygon_ops.rs` (perimeter calculation, edge length, edge classification: straight/convex/concave, and point-on-edge interpolation).
- **Task 1.3**: Implement the tab generation algorithm in `crates/thetta-geometry/src/tabs.rs` following the RESEARCH-005 specification (perimeter count estimation, edge scoring, proportional distribution, overlap validation).
- **Task 1.4**: Write comprehensive unit tests for the algorithm covering basic shapes (rectangles, circles, L-shapes) and edge cases like tiny parts or degenerate polygons.

### Phase 2: App State and UI Integration
- **Task 2.1**: Update `AppState` in `crates/thetta-app/src/state.rs` to include `TabConfig`.
- **Task 2.2**: Add a new configuration section in `crates/thetta-app/src/panels.rs` to control tab parameters (target distance, min/max count, material thickness, corner exclusion).
- **Task 2.3**: Hook up the tab generation step to execute immediately when `NestingUpdate::Finished` is received or when `TabConfig` changes.

### Phase 3: Rendering and Export
- **Task 3.1**: Update `crates/thetta-app/src/viewport.rs` to draw tabs. Tabs should be rendered as distinct visual indicators (e.g., colored rectangles) on the edges of placed pieces.
- **Task 3.2**: Modify JSON export in `crates/thetta-step/src/export.rs` to serialize the computed tab data.
- **Task 3.3**: Modify OCCT STEP export in `crates/thetta-step/src/export.rs` to include tabs as export metadata or visible geometry.

## Files Summary

| File | Proposed Changes |
|------|------------------|
| `crates/thetta-geometry/src/types.rs` | Add `Tab`, `TabConfig`, `MaterialType`, `TabResult` types. |
| `crates/thetta-geometry/src/polygon_ops.rs` | Add `perimeter`, edge classification, point interpolation. |
| `crates/thetta-geometry/src/tabs.rs` | (New) Implement the automatic tab placement algorithm. |
| `crates/thetta-geometry/src/lib.rs` | Export the new `tabs` module. |
| `crates/thetta-app/src/state.rs` | Store `TabConfig` and trigger tab generation post-nesting. |
| `crates/thetta-app/src/panels.rs` | UI controls for tab distance, min/max, size, and material. |
| `crates/thetta-app/src/viewport.rs` | Render tabs on nested parts. |
| `crates/thetta-step/src/export.rs` | Include tab data in JSON and STEP exports. |

## Definition of Done
- [ ] `TabConfig` and related tab types are defined and serializable.
- [ ] Tab placement algorithm accurately classifies straight edges, convex curves, and concave curves.
- [ ] Algorithm enforces minimum tab counts and corner exclusion distances according to RESEARCH-005 spec.
- [ ] Tiny parts (below perimeter threshold) are correctly skipped.
- [ ] Unit tests pass for basic shapes and edge cases.
- [ ] Tab configuration is exposed in the UI and updates recalculate tabs deterministically.
- [ ] Tabs are visually rendered on part contours in the 2D viewport.
- [ ] Tab data is successfully exported in JSON format.
- [ ] No regressions in existing nesting benchmarks or full pipeline tests.

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Edge classification complexity | High | Start with a robust cross-product-based vertex angle check. Add extensive unit tests for complex concave/convex polygons. |
| Performance degradation | Medium | Keep algorithm O(N) relative to polygon edges. Run tab generation on the optimizer background thread if it blocks the UI. |
| Overlapping tabs on short/narrow edges | Medium | Implement the minimum spacing validation (Step 4 of spec) rigorously. Fall back to fewer tabs if spacing constraints cannot be met. |
| Downstream CAM compatibility | Medium | Export as JSON first. For STEP export, consult CAM software standards to decide between metadata or explicit physical modeling of tab volumes. |

## Security Considerations
- No new external dependencies are introduced, minimizing supply chain risks.
- Tab placement relies entirely on pure geometric computations of existing in-memory data, introducing no new filesystem or network I/O attack vectors.

## Dependencies
- Existing `thetta-geometry` mathematical foundation.
- No new external crates required (pure Rust geometry math).

## Open Questions
1. How should tabs be structurally represented in the final STEP export so that standard CAM software (like Fusion 360 or VCarve) can recognize them natively?
2. Should tab generation be re-run continuously during the GLS optimization phase for visual feedback, or strictly once upon receiving `NestingUpdate::Finished`?
3. Should we enforce rectangular tabs only for Phase 1, or lay the structural groundwork for triangular/ramped tabs in the data model immediately?
