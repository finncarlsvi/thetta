# Sprint 008 Intent: Hole Drilling and Pocket Clearing

## Seed

Complete the 2D CAM pipeline by adding hole drilling and pocket clearing operations. From IDEA.md:
- **Hole drilling** — Drill through-holes detected in the piece geometry. Use canned drill cycles (G81 for simple, G83 for peck drilling).
- **Pocket clearing** — Mill out enclosed pockets detected in the piece geometry. Use contour-parallel (offset) pocketing strategy.
- **Operation ordering** — Drill holes first, then pockets, then profile cuts (profiles last so pieces stay fixed).

After this sprint, Thetta has all three 2D CAM operations specified in IDEA.md and can generate complete G-code programs for CNC production.

## Context

- **Current state**: 7 sprints complete. Full pipeline: STEP import → nesting → tab placement → G-code profile cuts. The `thetta-cam` crate has toolpath planner, G-code emitter, tool DB, and profile cut generation.
- **Recent direction**: Sprint 007 added `thetta-cam` with profile cuts. The G-code emitter and tool DB infrastructure are in place — this sprint extends them with two new operation types.
- **Key existing infrastructure**:
  - `Polygon { outer: Vec<Point2D>, holes: Vec<Vec<Point2D>> }` — holes are already parsed from STEP import but unused in CAM
  - `thetta-cam/src/toolpath.rs` — profile toolpath planner (offset, tab projection, lead-in/out, contour walk)
  - `thetta-cam/src/gcode.rs` — LinuxCNC G-code emitter with modal output
  - `thetta-cam/src/generate.rs` — per-sheet orchestration (currently profile-only)
  - `Tool`, `ToolTable` — tool DB supports multiple tools, can assign different tools to different operations
  - `ToolpathMove` enum — may need new variants for drill cycles and pocket moves

## Recent Sprint Context

| Sprint | Theme |
|--------|-------|
| 004 | Multi-sheet nesting |
| 005 | Architecture documentation |
| 006 | Per-part tab placement |
| 007 | G-code profile cuts + tool database |

## Relevant Codebase Areas

### Hole geometry (`crates/thetta-geometry/src/types.rs`)
- `Polygon.holes: Vec<Vec<Point2D>>` — inner contours already exist
- Holes are CW-wound (opposite to outer CCW)
- Need to detect which holes are "drill holes" (circular, small) vs "pockets" (non-circular or large)

### CAM crate (`crates/thetta-cam/src/`)
- `toolpath.rs` — add drill cycle planning and pocket toolpath planning
- `gcode.rs` — add G81/G83 drill cycle emission, pocket move emission
- `generate.rs` — orchestrate all three operations in correct order (drill → pocket → profile)
- `tool_db.rs` — may need tool type classification (drill bit vs endmill)

### App integration (`crates/thetta-app/src/`)
- `state.rs` — add drill and pocket operation configs, tool assignment per operation
- `panels.rs` — add drill and pocket config panels, operation ordering display

## Constraints

- Must follow existing `thetta-cam` patterns (toolpath intermediate representation → G-code emission)
- Must work with multi-sheet results (per-sheet G-code includes all three operation types)
- Hole detection: a "drill hole" is a circular or near-circular inner contour. A "pocket" is a non-circular enclosed region.
- Pocketing strategy: contour-parallel (offset-based) is simplest and sufficient for 2D pockets
- Tool assignment: user can assign different tools to drill, pocket, and profile operations
- Operation order in G-code: drill first, then pocket, then profile (IDEA.md requirement)
- Each operation type gets its own tool change (T/M6) if using a different tool

## Success Criteria

1. Holes in piece geometry are detected and classified (drill vs pocket)
2. Drill holes produce G81 or G83 canned cycles in G-code
3. Pockets produce contour-parallel clearing toolpaths
4. G-code ordering: drill → pocket → profile within each piece (or globally per sheet)
5. User can assign separate tools to each operation type
6. Multi-sheet export includes all three operation types per sheet
7. UI shows operation configuration for drill and pocket

## Verification Strategy

- **Testing**: Unit tests for hole classification, drill cycle generation, pocket toolpath, operation ordering
- **Visual**: G-code viewer (CAMotics) to verify drill positions and pocket clearing patterns
- **Edge cases**: pieces with no holes, pieces with only drill holes, pieces with only pockets, mixed, nested pockets (hole within a hole — unlikely but possible)

## Uncertainty Assessment

- Correctness uncertainty: **Medium** — Drill cycles are straightforward (G81/G83). Pocket clearing (contour-parallel offset) is moderately complex.
- Scope uncertainty: **Low** — Three well-defined operations, infrastructure already exists.
- Architecture uncertainty: **Low** — Extends existing `thetta-cam` patterns. No new crates.

## Open Questions

1. How to classify holes: threshold on circularity? Or all holes are drilled, only non-circular ones also get pocketed?
2. Peck drilling (G83) vs simple drilling (G81): always peck, or configurable?
3. Pocket clearing strategy: contour-parallel (offset inward) or zigzag (raster)?
4. Should pocket clearing include a finish pass at full pocket depth along the pocket boundary?
5. Step-over for pocket clearing: percentage of tool diameter (e.g., 50%)?
6. How to handle operation ordering: per-piece or per-sheet? (IDEA.md says drill first globally)
7. Should the user be able to disable specific operations (e.g., skip drilling)?
