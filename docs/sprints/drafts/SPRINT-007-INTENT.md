# Sprint 007 Intent: G-code Profile Cuts with Tool Database

## Seed

Implement G-code generation for profile cuts with tab-aware toolpaths, plus a tool database compatible with LinuxCNC's tool table format. This is the core CAM feature that makes Thetta usable for actual CNC production: after nesting and tab placement, the user exports G-code that a CNC router can cut.

From IDEA.md:
- Profile cuts: cut along outer contour of each nested piece, with holding tabs
- Tool database: tool number, diameter, flute count, feed rate, plunge rate, spindle speed
- G-code export: one file per sheet, LinuxCNC conventions (T/M6), tab-aware profile cuts (ramp down, lift over tabs), configurable safe-Z/feeds/speeds
- Operation ordering: profiles last (so pieces stay fixed) — but this sprint is profile-only, no holes/pockets yet

## Context

- **Current state**: 6 sprints complete. Full pipeline: STEP import → 2D nesting → multi-sheet → tab placement → STEP/JSON export. Tab placement (Sprint 006) produces `Vec<Tab>` per placement with world-space positions, edge indices, and dimensions.
- **Recent direction**: Sprint 006 added per-part tab placement with material-aware sizing. The tab data is now available for G-code generation to use.
- **Key data available**: `NestingResult` with placed pieces, `TabState` with `HashMap<(usize, usize), Vec<Tab>>` keyed by (sheet_index, placement_index). Each `Tab` has position, edge_index, t parameter, width, height.
- **Crate structure**: `thetta-geometry` (types), `jagua-rs` (collision), `thetta-nesting` (optimizer), `thetta-step` (STEP I/O), `thetta-app` (egui UI).

## Recent Sprint Context

| Sprint | Theme |
|--------|-------|
| 001 | Nesting engine + interactive egui GUI |
| 002 | jagua-rs integration + ESICUP benchmarks |
| 003 | STEP import/export via OpenCASCADE |
| 004 | Multi-sheet nesting with fill-and-spill overflow |
| 005 | Architecture documentation |
| 006 | Automatic per-part tab placement |

## Relevant Codebase Areas

### Existing types (`crates/thetta-geometry/src/types.rs`)
- `Polygon`, `Point2D`, `Placement`, `Sheet`, `NestingResult`
- `Tab { position, edge_index, t, width, height }`, `TabConfig`, `MaterialType`
- New types needed: `Tool`, `ToolTable`, `GcodeConfig`, `CamOperation`, `Toolpath`

### Existing tab module (`crates/thetta-geometry/src/tabs.rs`)
- `generate_tabs()` returns `Vec<Tab>` with positions along contour edges
- Tab height = depth left uncut from bottom; G-code must ramp/lift over these

### App state (`crates/thetta-app/src/state.rs`)
- `AppState` owns `TabState`, `NestingResult`, `pieces`
- New: tool table storage, G-code generation trigger, export

### Export (`crates/thetta-step/src/export.rs`)
- Currently exports STEP and JSON. G-code is a fundamentally different format.
- G-code export could go in `thetta-step` (it's the I/O crate) or a new `thetta-cam` crate.

## Constraints

- Must follow existing architecture (non-blocking UI, channel-based updates, types in thetta-geometry)
- G-code must target LinuxCNC specifically (T/M6 tool change, G-code dialect)
- Tool table must be persistable (save/load to disk, likely JSON)
- Profile cut toolpath must be tab-aware: at each tab position, ramp the tool up to (cut_depth - tab_height) instead of cutting through
- One G-code file per sheet
- Safe-Z height for rapid moves between operations
- No new heavy external dependencies (G-code is just text output)

## Success Criteria

1. User can create/edit/delete tools in a tool database UI
2. User can assign a tool to profile cut operations
3. User can configure G-code parameters (safe-Z, cut depth, step-down)
4. Clicking "Generate G-code" produces a valid LinuxCNC G-code file per sheet
5. G-code profile cuts follow piece outer contours in world space
6. G-code respects tab positions: tool lifts to tab height at each tab
7. G-code includes proper preamble (units, coordinate system) and postamble (spindle off, home)
8. Tool table persists across sessions (save/load)

## Verification Strategy

- **Reference**: LinuxCNC G-code reference (https://linuxcnc.org/docs/html/gcode/g-code.html)
- **Testing**: Unit tests for toolpath generation (contour → G-code moves), tab-aware height changes, tool table serialization
- **Visual verification**: Open generated G-code in a G-code viewer/simulator (e.g., CAMotics, ncviewer.com)
- **Edge cases**: empty sheets, pieces with no tabs, single-piece sheets, multi-sheet export

## Uncertainty Assessment

- Correctness uncertainty: **Medium** — G-code syntax is well-documented but tab-aware toolpath generation (ramp transitions, lead-in/lead-out) has nuance
- Scope uncertainty: **Medium** — Profile cuts only (no holes/pockets), but tool DB + G-code + UI is substantial
- Architecture uncertainty: **Medium** — New crate vs existing crate for G-code? Toolpath as intermediate representation vs direct G-code emit?

## Open Questions

1. Should G-code generation live in `thetta-step` (existing I/O crate) or a new `thetta-cam` crate?
2. Should there be an intermediate `Toolpath` representation, or emit G-code directly from contour + tabs?
3. How should lead-in/lead-out be handled? (Tangential arc? Ramp? Direct plunge?)
4. Should step-down (multiple depth passes) be supported in this sprint or deferred?
5. What's the right tool table persistence format? (JSON file in app data dir?)
6. Should the G-code viewer/preview be in-app (viewport overlay) or external only?
7. How to handle cutting direction (climb vs conventional milling)?
