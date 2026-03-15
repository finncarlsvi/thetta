# Sprint 006: Automatic Per-Part Tab Placement (Post-Nesting) — Codex Draft

## Overview
Sprint 006 adds automatic holding-tab placement for CNC routing of wood/plastic parts as a deterministic post-processing step that runs after nesting completes. The goal is to convert each placed contour into a set of tab locations and dimensions that improve part retention during cut-through, while avoiding corner-heavy and low-quality placements. This sprint is explicitly scoped to per-part tabs only; cross-part/shared-tab coordination is deferred.

The implementation follows RESEARCH-005 Section 4 and combines three proven heuristics: perimeter-based tab count, edge-length/quality-weighted distribution, and corner exclusion. The algorithm computes tab count with `clamp(round(perimeter / tab_distance), min_tabs, max_tabs)`, builds scored candidate zones on the outer contour, distributes tabs proportionally, enforces spacing constraints, and force-recovers to `min_tabs` when needed.

Architecturally, this fits cleanly into Thetta’s existing pipeline (`import -> nesting -> result`) by attaching a tab-generation stage to nesting output handling, without changing LBF/GLS/compression behavior. The sprint also includes UI controls for tab config, viewport visualization, and serialization support so results remain inspectable and export-ready for downstream toolpath generation.

## Use Cases
1. CNC operator runs nesting for plywood parts and immediately receives tab placements per part, without manual tab authoring.
2. User tunes `tab_distance`, `tab_width`, `corner_exclusion`, and min/max limits to match tool diameter and material behavior.
3. Small parts below a perimeter threshold are automatically skipped to avoid unmachinable or noisy tab outputs.
4. Multi-sheet jobs produce tab sets per sheet with consistent deterministic behavior across repeated runs.
5. Reviewer validates tab algorithm quality visually in viewport and numerically through unit tests for rectangles, circles, concave parts, and degenerate contours.

## Architecture

### Components
- `thetta-geometry::types`
  - Add canonical tab data model: `Tab`, `TabConfig`, `MaterialType`, `TabsForPlacement`, `TabResult`.
  - Keep tab representation independent from UI and optimizer internals.
- `thetta-geometry::polygon_ops`
  - Add geometry primitives needed by Section 4 algorithm: perimeter, edge iteration, edge interpolation by arclength parameter, local convex/concave classification, corner-distance exclusion helpers.
- `thetta-geometry::tab_placement` (new)
  - Implements the full algorithm from RESEARCH-005 Section 4.
  - Public API returns deterministic tabs for one placed polygon and an aggregator for entire `NestingResult`.
- `thetta-nesting`
  - No change to optimization objective/functions.
  - Add post-processing integration point where final (or improved) `NestingResult` is enriched with tab results.
- `thetta-app`
  - Store tab config and latest tab result in `AppState`.
  - Add “Tabs” panel controls and regenerate tabs whenever nesting result/config changes.
  - Render tabs in viewport as overlays anchored to placed part edges.
- `thetta-step`
  - Phase-1 serialization support in JSON export path (metadata only, no OCCT geometry modification).

### Data Flow
1. Nesting pipeline emits `NestingResult`.
2. Post-processing transforms each `Piece.polygon` by its `Placement` to world space.
3. Tab algorithm runs on the placed outer contour only.
4. Results are grouped by sheet + piece placement and stored as `TabResult`.
5. UI consumes `TabResult` for visualization and export metadata.

```text
Input Pieces + SheetConfig
        |
        v
run_optimizer (LBF -> GLS -> compression)
        |
        v
NestingResult { sheets[] }
        |
        |  post-process (new)
        v
compute_tabs_for_nesting_result(..., TabConfig)
        |
        v
TabResult { sheet_tabs[] }
        |
   +----+--------------------+
   |                         |
   v                         v
Viewport overlay        Export metadata (JSON)
```

```text
Per placed part:

Placed outer ring
   |
   +--> perimeter P
   |      if P < size_threshold => []
   |
   +--> count = clamp(round(P/tab_distance), min_tabs, max_tabs)
   |
   +--> candidate edges with score:
   |      straight=1.0, convex=0.8, concave=0.3, too-short=0.0
   |
   +--> allocate by weighted length and distribute on edge
   |
   +--> spacing validation (>= 2*tab_width)
   |
   +--> min_tabs recovery (relax corner exclusion if required)
   |
   v
Vec<Tab>
```

## Implementation

### Phase 1: Data Model + Geometry Primitives
Files:
- `crates/thetta-geometry/src/types.rs`
- `crates/thetta-geometry/src/polygon_ops.rs`
- `crates/thetta-geometry/src/lib.rs`

Tasks:
- Add tab domain types:
  - `MaterialType { WoodSoft, WoodHard, Plastic, Composite }`
  - `Tab { position: Point2D, edge_index: usize, t: f64, width: f64, height: f64 }`
  - `TabConfig { tab_distance, min_tabs, max_tabs, tab_width, tab_height, size_threshold, corner_exclusion, material_thickness, material_type }`
  - `TabsForPlacement { piece_id, sheet_index, tabs: Vec<Tab> }`
  - `TabResult { by_sheet: Vec<Vec<TabsForPlacement>> }`
- Extend polygon utilities:
  - `perimeter(&Polygon) -> f64` (outer ring only for tab placement)
  - Edge iterator helpers with `(start, end, index, length)`
  - Parameterized edge interpolation (`point_at_t`)
  - Corner-distance checks on polyline edges
  - Convex/concave local classification based on winding and cross-product sign
- Export new module from `lib.rs`.

### Phase 2: Core Tab Placement Algorithm
Files:
- `crates/thetta-geometry/src/tab_placement.rs` (new)
- `crates/thetta-geometry/src/lib.rs`
- `crates/thetta-geometry/src/test_shapes.rs` (optional new fixtures)

Tasks:
- Implement Section 4 algorithm functions:
  - `compute_tabs_for_contour(contour, config) -> Vec<Tab>`
  - `tab_height_for_material(material_type, thickness) -> f64`
  - `default_tab_width_from_height(height) -> f64` with practical clamp bounds
- Implement deterministic candidate-zone scoring:
  - straight `1.0`, convex `0.8`, concave `0.3`, invalid `0.0`
- Implement weighted tab allocation and remainder handling with stable ordering.
- Implement spacing conflict resolver (drop lower-score candidate first).
- Implement min-count recovery path with controlled corner-exclusion relaxation.
- Add comprehensive unit tests:
  - rectangle, circle approximation, concave L-shape, short-edge skip, tiny-part skip, determinism, spacing constraints.

### Phase 3: Nesting Integration (Post-Processing)
Files:
- `crates/thetta-nesting/src/runner.rs`
- `crates/thetta-nesting/src/optimizer.rs`
- `crates/thetta-nesting/src/lib.rs`
- `crates/thetta-app/src/state.rs`

Tasks:
- Keep nesting objective unchanged; add tab post-processing after result creation.
- Add helper to map `Placement + Piece.polygon` -> placed polygon and compute tabs.
- Decide storage path for sprint scope:
  - Preferred: keep `NestingResult` unchanged and store `TabResult` in app state to avoid broad serialization churn.
  - Alternate (if accepted): add optional `tabs` field to `Sheet`/`NestingResult`.
- Trigger tab recompute on:
  - new `NestingUpdate::Improved/Finished`
  - tab-config changes
  - thickness/material changes.

### Phase 4: UI Controls + Visualization
Files:
- `crates/thetta-app/src/state.rs`
- `crates/thetta-app/src/panels.rs`
- `crates/thetta-app/src/viewport.rs`

Tasks:
- Add `AppState.tab_config` and `AppState.tab_result`.
- Add “Tabs” panel section with controls:
  - spacing, min/max, width/height (or material-driven auto), corner exclusion, size threshold, enable/disable toggle.
- Draw tab markers in viewport:
  - consistent per-sheet offset handling
  - distinguish selected piece tabs
  - no hit-testing regression for piece selection.
- Ensure non-blocking behavior; tab compute remains fast and deterministic.

### Phase 5: Export + Validation
Files:
- `crates/thetta-step/src/export.rs`
- `crates/thetta-step/src/lib.rs` (if API surface changes)
- `crates/thetta-geometry/src/tab_placement.rs` (tests)
- `crates/thetta-app/src/state.rs` (integration checks)

Tasks:
- Extend JSON export format to include tab metadata per exported solid/placement.
- Leave OCCT STEP geometry unchanged in this sprint (metadata path only).
- Add regression tests for serialization and deterministic output snapshots.
- Manual validation set:
  - one-sheet rectangle parts
  - mixed concave/convex parts
  - multi-sheet overflow dataset.

## Files Summary

| File | Change Type | Purpose |
|------|-------------|---------|
| `crates/thetta-geometry/src/types.rs` | Modify | Add tab domain/config/result types |
| `crates/thetta-geometry/src/polygon_ops.rs` | Modify | Add perimeter, edge helpers, classification utilities |
| `crates/thetta-geometry/src/tab_placement.rs` | New | Section 4 algorithm implementation |
| `crates/thetta-geometry/src/lib.rs` | Modify | Export new module(s) |
| `crates/thetta-nesting/src/runner.rs` | Modify | Hook post-processing after nesting result generation |
| `crates/thetta-app/src/state.rs` | Modify | Store tab config/results and trigger recomputation |
| `crates/thetta-app/src/panels.rs` | Modify | Add tab configuration controls |
| `crates/thetta-app/src/viewport.rs` | Modify | Render tab overlays on placed contours |
| `crates/thetta-step/src/export.rs` | Modify | Include tab metadata in JSON exports |
| `docs/sprints/drafts/SPRINT-006-CODEX-DRAFT.md` | New | Sprint draft artifact |

## Definition of Done
- [ ] Automatic per-part tab placement runs as post-processing after nesting.
- [ ] Tab count uses `clamp(round(perimeter / tab_distance), min_tabs, max_tabs)`.
- [ ] Tiny parts below `size_threshold` produce zero tabs.
- [ ] Candidate scoring and corner exclusion match RESEARCH-005 Section 4.
- [ ] Spacing validation enforces minimum separation and resolves overlaps deterministically.
- [ ] Recovery logic guarantees at least `min_tabs` when geometrically feasible.
- [ ] Works across all sheets in multi-sheet nesting results.
- [ ] UI exposes tab parameters and updates results interactively.
- [ ] Viewport displays tab markers correctly with sheet offsets and piece transforms.
- [ ] Unit tests cover rectangles, circles, concave geometry, tiny/degenerate parts, and determinism.
- [ ] JSON export includes tab metadata; OCCT path remains stable.

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Convex/concave classification errors on noisy polygons | Poor candidate scoring and unstable tab quality | Normalize winding assumptions, add orientation-aware tests, include tolerances for near-collinear edges |
| Too many tabs collapse onto short edges | Invalid or overlapping tabs | Pre-filter by `tab_width + 2*corner_exclusion`, run spacing pass, prune low-score conflicts |
| Determinism drift from floating-point tie-breakers | Hard-to-reproduce behavior | Stable sorting keys with explicit tie order (`edge_index`, `t`) and fixed epsilon comparisons |
| UI re-computation overhead on each frame | Perceived lag in large jobs | Recompute only on config/result changes, cache `TabResult` in state |
| Export schema churn affecting downstream scripts | Integration breakage | Version JSON schema or add backward-compatible optional `tabs` field |
| Concave/tiny degenerate contours | Runtime errors or empty output where tabs expected | Guard clauses for ring size/length, explicit tests for degenerate inputs |

## Security Considerations
Tab placement is geometry-only and does not introduce new network, process, or privilege boundaries. Main security concerns are robustness and safe failure behavior: malformed polygons, NaN coordinates, or extreme config values must not panic or hang worker threads. All new public APIs should validate numeric inputs (finite/non-negative where required) and default to no-tab output rather than unsafe behavior. Export additions should remain strictly data serialization and avoid executing external tooling.

## Dependencies
- Existing canonical geometry model in `thetta-geometry` (`Polygon`, `Placement`, `NestingResult`).
- Existing nesting update lifecycle in `thetta-nesting`/`thetta-app` (`NestingUpdate::{Improved, Finished}`).
- Existing viewport transform/render pipeline for placed polygons.
- Existing JSON export path in `thetta-step` for metadata extension.
- RESEARCH-005 Section 4 as algorithmic source of truth for this sprint.

## Open Questions
1. Should tab results live in `NestingResult` (shared across crates) or remain app-level derived state in Sprint 006?
2. Should default `tab_width` be user-entered directly or derived from material rule (`1.5 * tab_height`) with override?
3. Do we allow tabs on hole contours in later sprints, or permanently restrict to outer contour?
4. What should happen when `min_tabs` is geometrically infeasible even after corner-exclusion relaxation?
5. Should JSON export embed tabs per placement or per piece instance keyed by `piece_id + sheet_index`?
6. Should Phase 1 ship rectangle-only visual markers while reserving future tab-shape variants (triangular/ramped)?
