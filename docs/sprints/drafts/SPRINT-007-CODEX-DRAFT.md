# Sprint 007: G-code Profile Cuts with Tool Database (LinuxCNC)

## Overview

Sprint 007 delivers the first production CAM output path for Thetta: LinuxCNC-compatible G-code for profile cuts, using nested placement geometry and Sprint 006 tab metadata. The sprint focuses on one operation class (profile cuts only) so the implementation can be validated end-to-end before adding drilling and pocketing. Output is one G-code file per sheet, preserving Thetta's multi-sheet model and existing export workflow.

The core technical requirement is tab-aware cutting. Profile toolpaths must follow each placed outer contour in world space, but at tab intervals they must lift from full depth to a tab-preserving depth (`cut_depth - tab_height`) and then return to full depth. This sprint formalizes that behavior in a deterministic, testable planning layer so LinuxCNC text emission is a separate concern from path segmentation logic.

In parallel, Sprint 007 introduces a persistent tool database aligned with LinuxCNC tooling workflows. Users can define tools (number, diameter, flute count, feed, plunge, spindle), assign a tool to the profile operation, and generate G-code with explicit tool change conventions (`Tn M6`) and machining preamble/postamble. The design keeps UI non-blocking and follows current crate boundaries: geometry/types in `thetta-geometry`, file-format/export concerns in `thetta-step`, orchestration in `thetta-app`.

## Use Cases

1. User nests parts, reviews tabs in viewport, selects a profile tool, and exports one LinuxCNC G-code file per sheet.
2. User maintains a reusable tool database across sessions (add/edit/delete tools, persisted locally).
3. User sets CAM parameters (`safe_z`, cut depth, optional step-down, spindle/feed/plunge overrides) and regenerates code without re-nesting.
4. User has placements without tabs; profile code is still valid and runs full-depth contour passes.
5. User has placements with tabs; generated path lifts over each tab location and preserves bridges.
6. User has multi-sheet nesting output; export creates separate files with deterministic naming and per-sheet toolpaths.
7. User reopens the app; tool database and last-selected profile tool remain available.
8. User exports with invalid setup (no tool assigned, invalid depth); UI blocks export with actionable validation errors.

## Architecture

### Component Descriptions

- `thetta-geometry::types`
- Add canonical CAM types: `Tool`, `ToolTable`, `ProfileCutConfig`, `GcodeConfig`, `CutDirection`.
- Keep serialization derives so state persistence and tests remain straightforward.

- `thetta-geometry::toolpath` (new module)
- Pure planner from `Polygon + Placement + Vec<Tab> + ProfileCutConfig` to ordered profile segments with Z-intent (`Cut`, `LiftOverTab`, `Entry`, `Exit`).
- No file I/O, no UI dependencies, deterministic ordering.

- `thetta-step::gcode` (new module)
- LinuxCNC emitter converting planned segments into G-code text.
- Owns dialect-specific preamble/postamble, modal strategy, feed/plunge/spindle/tool change lines.

- `thetta-step::tool_table` (new module)
- Load/save tool table in JSON at app data path.
- Optional LinuxCNC `.tbl` text export helper deferred unless trivial.

- `thetta-app::state`
- Add `CamState` holding tool table, selected profile tool, profile/gcode config, and generation status.
- Trigger background G-code export job similar to existing STEP export pattern.

- `thetta-app::panels`
- Add CAM panel sections for tool management, profile operation config, and `Generate G-code` action.

### Data Flow

```
NestingResult + pieces + TabState.results + CamState
                    |
                    v
          per sheet / per placement
                    |
                    v
   transform piece polygon to world-space contour
                    |
                    v
plan_profile_toolpath(contour, tabs_for_placement, profile_cfg)
                    |
                    v
Vec<ToolpathMove/Segment> (tab-aware Z transitions)
                    |
                    v
emit_linuxcnc_gcode(sheet_plan, tool, gcode_cfg)
                    |
                    v
sheet_001.ngc, sheet_002.ngc, ...
```

### CAM Pipeline Diagram

```
+---------------------------+       +---------------------------+
|      thetta-app           |       |      thetta-geometry      |
| Cam panel + AppState      | ----> | toolpath planner (pure)   |
| validates tool/config     |       | tab-aware profile segments |
+-------------+-------------+       +--------------+------------+
              |                                    |
              v                                    v
+-------------+------------------------------------+------------+
|                      thetta-step                              |
| gcode emitter (LinuxCNC dialect) + tool-table persistence    |
+-------------+------------------------------------+------------+
              |
              v
       .ngc per sheet + tools.json
```

### Design Decisions

1. Intermediate toolpath representation is required. Direct contour-to-text emission is harder to test for tab-lift correctness and future operation ordering.
2. Toolpath planning stays in `thetta-geometry` because it is geometric/domain logic, independent of specific file formats.
3. LinuxCNC string emission stays in `thetta-step` as export boundary responsibility.
4. Tool table persistence uses JSON first for deterministic serialization and easy migration.
5. Export remains background-threaded to preserve responsive egui UI behavior.

## Implementation

### Phase 1: CAM Types and Validation Contracts

**Files**
- `crates/thetta-geometry/src/types.rs`
- `crates/thetta-geometry/src/lib.rs`

**Tasks**
- Add `Tool` with required fields: number, name, diameter_mm, flute_count, feed_mm_min, plunge_mm_min, spindle_rpm.
- Add `ToolTable { tools: Vec<Tool>, version: u32 }` with lookup helpers and uniqueness validation on tool number.
- Add `ProfileCutConfig` (`cut_depth`, `step_down`, `safe_z`, `lead_in_mm`, `lead_out_mm`, `cut_direction`, `use_tabs`).
- Add `GcodeConfig` (`units`, `coordinate_mode`, optional comments, default feed strategy).
- Add validation helpers returning user-facing error enums (no panics on invalid config).
- Export new module/types in `lib.rs`.

### Phase 2: Tab-Aware Profile Toolpath Planner

**Files**
- `crates/thetta-geometry/src/toolpath.rs` (new)
- `crates/thetta-geometry/src/lib.rs`
- `crates/thetta-geometry/tests/toolpath_tests.rs` (new)

**Tasks**
- Implement `plan_profile_cut(...) -> Result<ProfilePlan, ToolpathError>` using placed outer contour only.
- Segment contour arcs/edges into alternating full-depth and tab-lift intervals using `Tab { edge_index, t, width, height }`.
- Implement deterministic ordering: sheet index, placement index, contour order, stable tie-breaks.
- Add direction handling (`climb`/`conventional`) via contour traversal order.
- Support multi-pass depth when `step_down < cut_depth`.
- Unit tests for: no-tabs path, single-tab lift, multiple tabs on same edge, adjacent tabs, tiny edges, deterministic output.

### Phase 3: LinuxCNC G-code Emitter and Sheet Export

**Files**
- `crates/thetta-step/src/gcode.rs` (new)
- `crates/thetta-step/src/export.rs`
- `crates/thetta-step/src/lib.rs`
- `crates/thetta-step/tests/gcode_export_tests.rs` (new)

**Tasks**
- Implement LinuxCNC emitter (`emit_program`, `emit_sheet`) with:
- Preamble (`G17`, `G21`, `G90`, safe start, spindle/tool setup).
- Tool change block (`Tn M6`) and spindle start (`M3 S...`).
- Motion blocks (`G0` rapids at safe-Z, `G1` cut/plunge/lift) from planned profile segments.
- Postamble (`M5`, retract, program end).
- Add `export_all_sheets_gcode(...)` to produce one `.ngc` per sheet.
- Ensure empty sheets still produce valid minimal program or are skipped by explicit policy.
- Add tests asserting required LinuxCNC tokens and tab-lift Z transitions are present.

### Phase 4: Tool Database Persistence

**Files**
- `crates/thetta-step/src/tool_table.rs` (new)
- `crates/thetta-step/src/lib.rs`
- `crates/thetta-step/tests/tool_table_tests.rs` (new)

**Tasks**
- Implement `load_tool_table(path)` and `save_tool_table(path, &ToolTable)` with atomic write pattern.
- Add migration guard via `version` field.
- Validate duplicate tool numbers and non-positive cutting parameters.
- Add roundtrip tests and malformed JSON tests.

### Phase 5: App Integration (State, UI, Background Jobs)

**Files**
- `crates/thetta-app/src/state.rs`
- `crates/thetta-app/src/panels.rs`
- `crates/thetta-app/src/main.rs`
- `crates/thetta-app/tests/cam_state_tests.rs` (new, if test harness exists)

**Tasks**
- Add `CamState` to `AppState` with tool table, selected tool id/number, profile config, gcode config.
- Add panel actions: add/edit/delete tool, select profile tool, generate G-code dialog.
- Add background export flow mirroring STEP export (`start_gcode_export`, `poll_gcode_export`).
- Reuse existing `NestingResult + TabState.results` as inputs; no mutation of nesting output types.
- Surface validation failures directly in panel status.

### Phase 6: End-to-End Validation and Documentation

**Files**
- `docs/sprints/SPRINT-007.md` (future finalized sprint)
- `crates/thetta-step/tests/full_pipeline_test.rs` (extend)
- `crates/thetta-app/examples/` (optional small exporter smoke example)

**Tasks**
- Add integration test for multi-sheet tab-aware G-code generation.
- Verify deterministic output for identical inputs.
- Manual verification in LinuxCNC-compatible viewer (CAMotics/ncviewer).
- Document known constraints (profile-only, no hole/pocket ops in this sprint).

## Files Summary

| File | New/Modified | Purpose |
|------|--------------|---------|
| `crates/thetta-geometry/src/types.rs` | Modified | CAM/tool/gcode config types + validation |
| `crates/thetta-geometry/src/toolpath.rs` | New | Tab-aware profile planning |
| `crates/thetta-geometry/src/lib.rs` | Modified | Module exports |
| `crates/thetta-geometry/tests/toolpath_tests.rs` | New | Planner correctness + determinism tests |
| `crates/thetta-step/src/gcode.rs` | New | LinuxCNC G-code emitter |
| `crates/thetta-step/src/tool_table.rs` | New | Tool table persistence |
| `crates/thetta-step/src/export.rs` | Modified | G-code sheet export entrypoints |
| `crates/thetta-step/src/lib.rs` | Modified | Re-export CAM APIs |
| `crates/thetta-step/tests/gcode_export_tests.rs` | New | Emitter and token checks |
| `crates/thetta-step/tests/tool_table_tests.rs` | New | Persistence + validation tests |
| `crates/thetta-app/src/state.rs` | Modified | `CamState`, async export orchestration |
| `crates/thetta-app/src/panels.rs` | Modified | Tool DB + CAM controls UI |
| `crates/thetta-app/src/main.rs` | Modified | New panel actions wiring |

## Definition of Done

- [ ] User can create, edit, and delete tools in-app.
- [ ] Tool table persists to disk and reloads on startup.
- [ ] Profile operation requires explicit tool selection before export.
- [ ] G-code export generates one `.ngc` file per sheet.
- [ ] G-code includes LinuxCNC-compatible tool change (`Tn M6`) and spindle control.
- [ ] Profile paths follow placed outer contours in world coordinates.
- [ ] Tab-aware lifts are emitted at tab intervals using tab heights.
- [ ] Safe-Z rapids are used for non-cut travel.
- [ ] Optional multi-pass depth (`step_down`) works and is validated.
- [ ] Empty/invalid export scenarios fail gracefully with clear user errors.
- [ ] Planner and emitter unit/integration tests pass.
- [ ] `cargo test --workspace` passes without regressions.

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Incorrect tab interval mapping from `edge_index/t` to contour distance | Medium | High | Keep mapping in pure planner module with focused tests for edge and corner cases. |
| LinuxCNC dialect mismatch (modal assumptions, unsupported tokens) | Medium | High | Keep emitter minimal (`G0/G1`, explicit pre/postamble), validate against LinuxCNC docs and viewer simulations. |
| Tool database corruption on interrupted write | Low | Medium | Use atomic temp-file write + rename and schema versioning. |
| Multi-pass + tab-lift interactions create unexpected Z transitions | Medium | Medium | Separate pass planning from tab segmentation and assert monotonic pass depths in tests. |
| UI complexity for tool editing slows sprint | Medium | Medium | Ship table-based CRUD UI first; defer advanced UX (sorting/filtering/import). |
| Long-running export blocks UI if executed on main thread | Low | High | Reuse existing background-thread export pattern with channel polling. |

## Security Considerations

- No network operations are introduced.
- G-code and tool-table writes are limited to user-selected output/application data paths.
- Input validation rejects invalid/unsafe tool parameters (non-positive feeds, diameters, or spindle values).
- No new `unsafe` Rust is required.
- Deterministic generation reduces hidden state and improves auditability of machine output.

## Dependencies

- No new external crates are required for baseline implementation.
- Existing internal dependencies remain consistent with current architecture:
- `thetta-geometry` for canonical types and planning.
- `thetta-step` for export format emission and persistence.
- `thetta-app` for orchestration and UI.
- LinuxCNC G-code reference semantics are implementation constraints, not build-time dependencies.

## Open Questions

1. Should `step_down` be mandatory in Sprint 007 or shipped as optional with a single-pass fallback default?
2. Should lead-in/lead-out be linear only in this sprint, deferring arc entry (`G2/G3`) to a later sprint?
3. Should tool table persistence path be global app data or project-local for portability?
4. Do we skip empty-sheet G-code files or emit minimal no-op programs for sheet index continuity?
5. Should climb/conventional default be material-dependent or fixed default (`climb`) for now?
6. Is LinuxCNC `.tbl` text export required in-sprint, or is JSON persistence plus in-app tool selection sufficient for v1?
