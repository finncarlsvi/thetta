# Sprint 008: Hole Drilling and Pocket Clearing

## Overview
Sprint 008 completes Thetta's 2D CAM operation set by adding hole drilling and pocket clearing on top of the existing profile-cut pipeline. The current `thetta-cam` flow already handles profile contour generation, tab-aware Z motion, and LinuxCNC G-code emission. This sprint extends that same architecture to support two additional operations sourced from `Polygon.holes`: drilling for circular/near-circular holes and contour-parallel pocket clearing for enclosed non-circular regions.

The CAM planner and emitter will continue using a two-stage design: plan operation-specific `ToolpathMove` sequences in `toolpath.rs`, then format LinuxCNC output in `gcode.rs`. `generate.rs` becomes the operation orchestrator for per-sheet programs, enforcing production-safe order: drill first, then pocket, then profile. This order preserves workholding and part stability by leaving outer contours intact until the end.

Scope includes type-model expansion in `thetta-geometry`, CAM planning and emission changes in `thetta-cam`, and app-layer configuration wiring for per-operation tool assignment and drill/pocket settings. Out of scope: 3D toolpaths, helical boring, trochoidal pocketing, and machine-specific probing workflows.

## Use Cases
1. Drill detected through-holes before contour cutting using a drill bit and LinuxCNC canned cycles (`G81` simple, `G83` peck).
2. Clear non-circular internal pockets using an endmill and contour-parallel inward offsets with configurable stepover.
3. Assign different tools for drill, pocket, and profile operations from the existing tool database.
4. Export one `.ngc` per sheet with all operations in stable order: drill -> pocket -> profile.
5. Skip operations automatically when a sheet has no qualifying holes/pockets while still producing valid output.
6. Generate multi-sheet output where each file is self-contained (`%...M2%`) and operation-complete.
7. Tune drilling behavior (cycle type, retract plane, peck depth, dwell) without affecting profile/pocket settings.

## Architecture
### Components
- `thetta-geometry/src/types.rs`
  - Extend CAM configuration and move vocabulary.
  - Add operation-specific config blocks and tool-number bindings.
- `thetta-cam/src/toolpath.rs`
  - Hole classification (`drill` vs `pocket`) from inner contours.
  - Drill-point planning and pocket offset-ring generation.
- `thetta-cam/src/gcode.rs`
  - Emit canned drill cycles (`G81`/`G83`, `G80`).
  - Emit pocket and profile motion in one modal-safe stream.
- `thetta-cam/src/generate.rs`
  - Build per-sheet operation plans and enforce ordering.
  - Handle tool change boundaries across operation groups.
- `thetta-app/src/state.rs` + `panels.rs`
  - Add drill/pocket config controls and tool selectors.
  - Reuse existing async export path and validation pattern.

### Data Flow
```text
NestingResult + Pieces + TabState + CAM Config + ToolTable
                |
                v
      generate_all_sheets(sheet_i)
                |
                v
      transform piece polygons to world space
                |
                +--> classify inner contours
                |      - drill candidates (circularity + diameter tolerance)
                |      - pocket candidates (remaining closed contours)
                |
                +--> plan drill ops (points + cycle params)
                |
                +--> plan pocket ops (offset rings)
                |
                +--> plan profile ops (existing tab-aware contour)
                |
                v
      ordered op stream: drill -> pocket -> profile
                |
                v
      emit_gcode() with tool changes + canned cycles + motion
                |
                v
          sheet_N.ngc
```

### Operation Model (proposed)
```text
SheetProgram
  - operations: Vec<OperationPlan>

OperationPlan
  - op_type: Drill | Pocket | Profile
  - tool_number: u32
  - toolpath: Toolpath

ToolpathMove (extended)
  - existing: Rapid, LinearCut, ArcCut, TabLift, Retract, Plunge
  - new: DrillCycleStart{...}, DrillHole{...}, DrillCycleEnd, PocketLink{...}
```

### Ordering and Grouping
```text
for each sheet:
  drill group (all pieces, selected drill tool)
  pocket group (all pieces, selected pocket tool)
  profile group (all pieces, selected profile tool)
```
Global-per-sheet grouping minimizes tool changes and matches IDEA ordering requirements.

## Implementation
### Phase 1: Type System and Config Surface
Files:
- `crates/thetta-geometry/src/types.rs`
- `crates/thetta-app/src/state.rs`
- `crates/thetta-app/src/panels.rs`

Tasks:
- Add `DrillCycleType` enum (`G81`, `G83`).
- Add `DrillConfig` (`cycle_type`, `retract_z`, `peck_depth`, `dwell_ms`, `hole_tolerance`, `min_diameter`, `max_diameter`).
- Add `PocketConfig` (`stepover_pct`, `stepdown`, `finish_pass`, `min_area`).
- Expand `GcodeConfig` to include per-operation tool numbers (`drill_tool_number`, `pocket_tool_number`, `profile_tool_number`) while preserving compatibility migration for current `tool_number` field.
- Extend `ToolpathMove` with drill-cycle and pocket-link variants needed by emitter.
- Add validation methods for new config blocks and app precondition checks.
- Wire UI controls for drill/pocket operation toggles and tool selection.

### Phase 2: Toolpath Planning (Drill + Pocket)
Files:
- `crates/thetta-cam/src/toolpath.rs`
- `crates/thetta-cam/src/toolpath.rs` tests section

Tasks:
- Implement inner-contour classification utilities:
  - Circularity score (perimeter/area derived).
  - Diameter consistency check from centroid radii.
  - Threshold routing: drill candidate vs pocket candidate.
- Add drill planning:
  - Compute hole center from contour centroid.
  - Deduplicate near-identical centers.
  - Produce cycle-compatible move sequence.
- Add contour-parallel pocket planning:
  - Convert hole ring to polygon region.
  - Repeated inward offset by `tool_diameter * stepover_pct` until exhausted.
  - Preserve cut direction and add safe linking between rings.
- Keep profile planner unchanged except integration hooks.
- Add unit tests for:
  - classification correctness,
  - pocket ring count/termination,
  - no self-intersection crash cases,
  - mixed piece (drill + pocket + profile) planning.

### Phase 3: G-code Emission and Orchestration
Files:
- `crates/thetta-cam/src/gcode.rs`
- `crates/thetta-cam/src/generate.rs`

Tasks:
- Add canned cycle emission:
  - `G81` and `G83` headers with `R` retract plane and `Z` depth.
  - one `X/Y` line per hole during active cycle.
  - terminate with `G80` before non-drill motion.
- Ensure modal safety:
  - reset/transition motion/feed states when entering/leaving cycle mode.
  - avoid mixed arc/feed commands while cycle active.
- Add operation grouping in `generate_sheet_gcode`:
  - collect drill/pocket/profile plans across placements,
  - enforce group order,
  - request tool changes only when tool number changes.
- Keep output as one self-contained program per sheet with stable header/postamble.
- Add emitter tests for:
  - `G81` sequence,
  - `G83` sequence,
  - `G80` cancellation,
  - ordered blocks and tool changes.

### Phase 4: Integration, Validation, and Regression Coverage
Files:
- `crates/thetta-app/src/state.rs`
- `crates/thetta-cam/src/generate.rs` tests
- `crates/thetta-cam/src/gcode.rs` tests
- `crates/thetta-cam/src/toolpath.rs` tests

Tasks:
- Extend `gcode_export_precondition_error()` for drill/pocket tool validation when operations enabled.
- Validate generated files on multi-sheet results (empty/non-empty combinations).
- Add regression fixtures for:
  - no holes,
  - drill-only holes,
  - pocket-only holes,
  - mixed geometry.
- Verify no regressions to Sprint 007 profile + tabs behavior.

## Files Summary
| File | Change Type | Purpose |
|---|---|---|
| `crates/thetta-geometry/src/types.rs` | Modify | Add drill/pocket configs, operation tool assignment, new move variants |
| `crates/thetta-cam/src/toolpath.rs` | Modify | Hole classification, drill planning, contour-parallel pocket planning |
| `crates/thetta-cam/src/gcode.rs` | Modify | Emit `G81/G83/G80`, pocket/link motion, modal transitions |
| `crates/thetta-cam/src/generate.rs` | Modify | Per-sheet operation orchestration and ordering |
| `crates/thetta-app/src/state.rs` | Modify | New config state, validation, async export preconditions |
| `crates/thetta-app/src/panels.rs` | Modify | Drill/pocket config UI and per-operation tool selectors |
| `crates/thetta-cam/src/lib.rs` | Optional modify | Re-export any new CAM types/helpers if needed |

## Definition of Done
- [ ] `P0` Drill holes are detected from `Polygon.holes` and emitted as valid LinuxCNC canned cycles (`G81` or `G83`).
- [ ] `P0` Pocket regions are cleared with contour-parallel offsets and produce valid toolpaths.
- [ ] `P0` G-code order per sheet is drill -> pocket -> profile.
- [ ] `P0` One `.ngc` file per non-empty sheet includes all enabled operations.
- [ ] `P0` Separate tools can be assigned to drill/pocket/profile, with correct `T/M6` handling.
- [ ] `P0` Existing profile/tab behavior remains functionally unchanged.
- [ ] `P1` Unit tests cover hole classification thresholds and cycle emission edge cases.
- [ ] `P1` Unit tests cover pocket offset termination and minimum-feature filtering.
- [ ] `P1` UI exposes drill/pocket parameters and validates invalid combinations.
- [ ] `P2` Optional finish pass for pockets configurable in UI.
- [ ] `P2` Optional per-material default presets for drill peck depth/stepover.

## Risks & Mitigations
| Risk | Impact | Mitigation |
|---|---|---|
| Hole classification false positives/negatives | Wrong operation (drill vs pocket), scrap risk | Use explicit circularity + diameter variance thresholds, add debug metrics and fixture tests |
| Pocket offset degenerates on narrow features | Invalid or empty toolpaths, possible crashes | Stop on min-area/min-segment threshold, validate finite coordinates, skip invalid rings |
| Canned cycle modal bugs (`G80` missing) | Unintended drilling motion | Centralize cycle state machine in emitter tests and enforce cycle close before mode switches |
| Excessive tool changes | Longer runtime, operator friction | Group by operation per sheet and only change tools when number differs |
| Backward config compatibility | Runtime errors on old saved state | Add serde defaults and migration-compatible fields in `types.rs` |
| Performance on dense pocket geometry | Slow generation | Cache transformed polygons and early-prune tiny pockets; benchmark pathological fixtures |

## Security Considerations
- Continue ASCII sanitization for user-provided strings included in G-code comments (`tool.name`, coordinate system labels).
- Reject non-finite numeric inputs and clamp unsafe ranges before emission.
- Keep generated output deterministic to reduce operator surprise and review burden.
- Treat exported G-code as untrusted machine instructions until reviewed; UI should continue surfacing destination path and operation summary.
- Avoid shelling out or external post-processors in CAM generation path.

## Dependencies
- Internal:
  - `thetta-geometry` CAM type definitions
  - `thetta-cam` planner/emitter/orchestrator modules
  - `thetta-app` state/panel integration
- Existing third-party crates already in use:
  - `geo-buffer` / `geo-types` for contour offsets
  - `serde` / `serde_json` for config persistence compatibility
- Requires Sprint 007 profile CAM baseline to remain stable.

## Open Questions
1. Classification policy: should near-circular holes above a diameter threshold become pockets instead of drills?
2. Drill depth source: reuse global `cut_depth` or add independent `drill_depth` in `DrillConfig`?
3. Pocket depth source: through-pocket at `cut_depth` only, or configurable depth per pocket operation?
4. `G83` defaults: what should be default `peck_depth` and retract behavior for wood/plastic workflows?
5. Operation enablement: should users be able to disable drill or pocket while keeping profile enabled?
6. Grouping policy: strictly per-sheet global ordering (fewer tool changes) vs per-piece ordering (locality).
7. Geometry edge handling: how to treat nested inner contours (hole-inside-hole) if present in imported data?
