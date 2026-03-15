# Sprint 005 Intent: Architecture Documentation (ARCHITECTURE.md)

## Seed

Create a comprehensive `docs/ARCHITECTURE.md` document that covers the architecture of the Thetta project — a desktop GUI application for 2D irregular nesting of shapes extracted from STEP files.

## Context

- **Current state**: 4 sprints complete. Full pipeline works: STEP import → 2D nesting (jagua-rs LBF+GLS) → multi-sheet overflow → STEP export. ~15K+ LOC across 5 crates.
- **Recent direction**: Sprint 004 added multi-sheet nesting with fill-and-spill overflow. The codebase has grown complex enough that an architecture document is needed for onboarding and maintainability.
- **Key architectural patterns**: Type-driven design, non-blocking UI with background threads and mpsc channels, multi-level optimization (LBF → GLS → Compression), forked jagua-rs for collision detection.
- **Constraints**: The document must accurately reflect the current codebase. No code changes — this is a documentation-only sprint.

## Recent Sprint Context

| Sprint | Theme |
|--------|-------|
| 001 | Nesting engine + interactive egui GUI |
| 002 | jagua-rs integration + ESICUP benchmarks |
| 003 | STEP import/export via OpenCASCADE |
| 004 | Multi-sheet nesting with fill-and-spill |

Recent commits focus on multi-sheet compression fixes, frame-aware 3D STEP export, and collision detection improvements.

## Relevant Codebase Areas

### Workspace Structure
```
crates/
├── thetta-geometry/   (2D types: Polygon, Piece, Placement, SheetConfig, NestingResult)
├── thetta-nesting/    (Optimizer: LBF, GLS, MultiSheet, NestingEngine, conversion)
├── thetta-app/        (egui GUI: state, viewport, panels, main loop)
├── thetta-step/       (STEP I/O via OpenCASCADE: import, export, profile, occt FFI)
└── jagua-rs/          (Forked collision detection: trigonometric queries, quadtree, SPP)
```

### Key Modules Per Crate

**thetta-geometry**: Polygon, Piece, Placement, SheetConfig, NestingResult, Sheet — the shared type vocabulary.

**thetta-nesting**: NestingEngine (single SPP wrapper), MultiSheetEngine (orchestrator for Vec<NestingEngine>), LBF (greedy heuristic), GLS (guided local search meta-heuristic), compression (strip width minimization), conversion (thetta-geometry ↔ jagua-rs bridge), sampler (random + coordinate descent), runner (pipeline orchestration).

**thetta-app**: AppState (central state machine), ViewportState (zoom/pan 2D rendering of all sheets), panels (side panel UI), main (eframe event loop with channel polling).

**thetta-step**: import (STEP → pieces via opencascade-sys FFI), export (NestingResult → STEP files with 3D transforms), profile (edge discretization), occt (FFI bindings, feature-gated).

**jagua-rs**: collision_detection (trigonometric direct queries, quadtree), entities (Item, Instance, PItemKey), geometry (transforms, rotation ranges), probs::spp (SPInstance, SPProblem, SPPlacement).

### Data Flow
```
STEP File → [thetta-step import] → Vec<Piece>
Vec<Piece> → [thetta-nesting LBF] → Initial Placement
Initial Placement → [thetta-nesting GLS] → Optimized Placement
Optimized Placement → [thetta-nesting Compression] → Final NestingResult
NestingResult → [thetta-app viewport] → 2D Rendering
NestingResult → [thetta-step export] → STEP Files (one per sheet)
```

### Threading Model
- Main thread: egui event loop, polls channels
- STEP import thread: background parsing via opencascade-sys
- Optimizer thread: LBF + GLS, sends progress via mpsc
- STEP export thread: background file writing

### Crate Dependency Graph
```
thetta-app → thetta-step → thetta-geometry
           → thetta-nesting → thetta-geometry
                            → jagua-rs
           → jagua-rs
```

## Constraints

1. **Documentation only** — No code changes in this sprint.
2. **Accuracy** — Must reflect the current codebase exactly, not aspirational architecture.
3. **Audience** — Should be useful for a new developer onboarding to the project, as well as for the existing developer (solo) as a reference.
4. **Maintainability** — Structure the document so sections can be updated independently as the codebase evolves.
5. **Sprint conventions** — Follow the existing sprint planning/documentation patterns in `docs/sprints/`.

## Success Criteria

1. `docs/ARCHITECTURE.md` exists and covers: project purpose, crate structure, dependency graph, data flow, threading model, optimization pipeline, UI architecture, testing patterns, and build/configuration notes.
2. A new developer could read this document and understand where to find things and how the pieces fit together.
3. All crates and their responsibilities are documented.
4. Key architectural decisions are explained (why SPP not BPP, why forked jagua-rs, why trigonometric collision detection, etc.).
5. Diagrams (ASCII art) illustrate the crate dependency graph, data flow, and threading model.

## Verification Strategy

- **Manual review**: Read the document alongside the code to verify accuracy.
- **Completeness check**: Every crate and key module should be mentioned.
- **No broken references**: All file paths mentioned in the doc should exist in the current codebase.
- **Build verification**: `cargo test --workspace` still passes (no code changes, but verify nothing was accidentally modified).

## Uncertainty Assessment

| Factor | Level | Reasoning |
|--------|-------|-----------|
| **Correctness** | Low | We're documenting existing code, not building new things. The codebase is the source of truth. |
| **Scope** | Low | Well-defined deliverable: one Markdown file. The challenge is depth vs. breadth. |
| **Architecture** | Low | No new architecture decisions — we're describing what exists. |

## Open Questions

1. **Depth of jagua-rs coverage**: Should we document the internals of the forked jagua-rs in detail, or treat it as a black box with documented API surface?
2. **Diagram format**: Plain ASCII art (portable, no tooling needed) vs. Mermaid (richer, rendered by GitHub)?
3. **Historical context**: Should we include a "how we got here" section covering sprint progression, or keep it purely current-state?
4. **Optimization algorithm detail**: How deep should the LBF/GLS/Compression algorithm descriptions go? Pseudocode level, or conceptual overview?
