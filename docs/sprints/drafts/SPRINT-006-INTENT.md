# Sprint 006 Intent: Automatic Tab Placement for CNC Routing

## Seed

Implement automatic tab placement for CNC-routed parts in the Thetta nesting tool. The algorithm and research are documented in `research/RESEARCH-005.md` (Section 4: Proposed Algorithm for Thetta). Key requirements: per-part tab placement as a post-processing step after nesting, targeting CNC router for wood/plastic. Shared tabs between adjacent parts are a future phase — this sprint is per-part only.

## Context

- **Current state**: 5 sprints complete. Full pipeline works: STEP import → 2D nesting (jagua-rs LBF+GLS) → multi-sheet overflow → STEP export. Architecture documented in `docs/ARCHITECTURE.md`.
- **Recent direction**: Sprint 005 was documentation. Sprint 004 added multi-sheet nesting. The nesting pipeline is mature; tab placement is the next step toward production-ready CNC output.
- **Key data model**: `NestingResult { sheets: Vec<Sheet> }` where each `Sheet` has `placements: Vec<Placement>`. Pieces have `Polygon { outer: Vec<Point2D>, holes: Vec<Vec<Point2D>> }`. Tab placement will operate on placed pieces (post-nesting).
- **Research foundation**: RESEARCH-005 analyzed 28 resources across academic, open-source, commercial, and community sources. A concrete algorithm is proposed in Section 4.
- **Crate structure**: `thetta-geometry` (types), `jagua-rs` (collision), `thetta-nesting` (optimizer), `thetta-step` (STEP I/O), `thetta-app` (egui UI).

## Recent Sprint Context

| Sprint | Theme |
|--------|-------|
| 001 | Nesting engine + interactive egui GUI |
| 002 | jagua-rs integration + ESICUP benchmarks |
| 003 | STEP import/export via OpenCASCADE |
| 004 | Multi-sheet nesting with fill-and-spill overflow |
| 005 | Architecture documentation (ARCHITECTURE.md) |

## Relevant Codebase Areas

### Types (`crates/thetta-geometry/src/types.rs`)
- `Polygon { outer: Vec<Point2D>, holes: Vec<Vec<Point2D>> }` — tab placement input
- `Placement { piece_id, x, y, rotation }` — needed to get placed contour
- `NestingResult`, `Sheet` — tab placement operates on these
- New types needed: `Tab`, `TabConfig`, `TabResult`

### Geometry utilities (`crates/thetta-geometry/src/polygon_ops.rs`)
- Existing: `area`, `bounding_box`, `translate`, `rotate`, `offset`
- New needed: `perimeter`, edge iteration, edge classification (straight/convex/concave), point-on-edge interpolation

### Nesting pipeline (`crates/thetta-nesting/src/`)
- Tab placement runs AFTER nesting completes (post-processing step)
- Could be a new module in `thetta-nesting` or a new module in `thetta-geometry`
- The `NestingResult` would gain a `tabs` field or there'd be a separate `TabResult`

### App/UI (`crates/thetta-app/src/`)
- `viewport.rs` — needs tab rendering (small rectangles on piece edges)
- `panels.rs` — needs tab configuration UI controls
- `state.rs` — needs tab config storage and tab generation trigger

### Export (`crates/thetta-step/src/export.rs`)
- Tab data needs to be included in export (future: G-code generation)
- For now, tabs could be exported as metadata or visual indicators in STEP/JSON

## Constraints

- Must follow existing architecture patterns (non-blocking UI, channel-based updates, types in thetta-geometry)
- Tab placement is a **post-processing step** — it does not modify the nesting algorithm
- Must work with multi-sheet results (tabs per sheet, per piece)
- Algorithm must handle: straight edges, convex curves, concave curves, short edges, corner avoidance
- Must gracefully handle edge cases: tiny parts (below size threshold), single-vertex edges, degenerate polygons
- No new external dependencies if possible (pure geometry computation)

## Success Criteria

1. Given a `NestingResult` with placed pieces, the tab placement algorithm produces a set of tabs for each piece
2. Tabs are placed on edges proportional to edge length, with corner avoidance
3. Minimum 3 tabs per part (preventing rotation)
4. Tab count scales with perimeter via `clamp(round(perimeter / tab_distance), min_tabs, max_tabs)`
5. Tab sizing scales with material thickness (configurable)
6. Tabs are rendered in the viewport as visual indicators
7. UI panel exposes tab configuration (distance, min/max count, width, height, corner exclusion)
8. Tab placement is deterministic (same input → same output)
9. Comprehensive unit tests covering: rectangles, circles, L-shapes, tiny parts, large parts, edge cases

## Verification Strategy

- **No reference implementation** — this is novel for the project
- **Spec**: Research RESEARCH-005 Section 4 defines the algorithm
- **Edge cases**: tiny parts (skip), very large perimeters (max_tabs cap), narrow edges (skip), circular shapes (uniform distribution), parts with holes (outer contour only for tabs)
- **Testing**: Unit tests for the algorithm module, visual inspection via app viewport
- **Determinism**: Same `Polygon` + `TabConfig` must always produce same `Vec<Tab>`

## Uncertainty Assessment

- Correctness uncertainty: **Medium** — The algorithm is well-specified in RESEARCH-005, but edge classification (convex vs concave) for arbitrary polygons needs careful implementation
- Scope uncertainty: **Low** — Per-part tab placement is well-bounded; shared tabs are explicitly deferred
- Architecture uncertainty: **Low** — Post-processing step fits cleanly into existing pipeline; no changes to nesting core

## Open Questions

1. Should tabs be stored on `Sheet` (alongside placements) or in a separate `TabResult` structure?
2. Should the tab module live in `thetta-geometry` (pure geometry) or `thetta-nesting` (nesting-adjacent)?
3. How should tabs be represented in export? (JSON metadata? Visual geometry in STEP?)
4. Should tab shapes be rectangular only for Phase 1, or include triangular/ramped?
5. Should the viewport show tabs only when they're computed, or always show "potential tab zones"?
6. What's the right default `tab_distance` for typical 2440x1220mm wood sheets?
