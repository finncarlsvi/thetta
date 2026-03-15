# Sprint 005: Architecture Documentation — Gemini Draft

## Overview

After 4 successful sprints, the Thetta project has reached a level of complexity where an explicit, comprehensive architecture document is required. The system now spans 5 crates (~15K+ LOC) and integrates complex domains including OpenCASCADE FFI for 3D STEP processing, a custom Guided Local Search (GLS) nesting optimizer, the `jagua-rs` collision detection engine, and a non-blocking `egui` GUI.

This sprint is a **documentation-only** sprint. The sole deliverable is a `docs/ARCHITECTURE.md` file that accurately reflects the current state of the codebase, serving as a reference for both onboarding new developers and guiding future development.

## Use Cases

While documentation doesn't have traditional end-user use cases, it has "reader" use cases:

### UC1: Onboarding a New Developer
- A new developer joins the project and reads the document to understand the crate structure and where different responsibilities live.
- They can trace the data flow from a STEP file input to a nested STEP file output.

### UC2: Understanding the Optimization Pipeline
- A contributor wants to improve the nesting algorithm. They use the document to understand the Multi-sheet orchestrator, the LBF heuristic, the GLS meta-heuristic, and the Compression phase, as well as how `jagua-rs` is utilized.

### UC3: UI and State Management Reference
- A developer needing to add a new UI feature references the document to understand the threading model (Main UI thread vs. Background Optimizer/STEP threads) and how state is synchronized via `mpsc` channels.

## Architecture of `ARCHITECTURE.md`

The proposed `docs/ARCHITECTURE.md` will contain the following key sections:

### 1. Introduction and System Context
- High-level overview of Thetta's purpose (2D irregular bin packing from 3D STEP files).
- The "Why": Why a custom GLS optimizer? Why `jagua-rs` instead of NFP? Why OpenCASCADE?

### 2. Crate Dependency Graph
- ASCII diagram illustrating the relationships between the 5 crates (`thetta-app`, `thetta-nesting`, `thetta-step`, `thetta-geometry`, and the forked `jagua-rs`).
- Brief 1-2 sentence description of each crate's responsibility.

### 3. Data Flow Pipeline
- Step-by-step trace of how data transforms through the system:
  `STEP -> opencascade-sys -> Vec<Piece> -> LBF -> GLS -> Compression -> NestingResult -> AppState -> 3D STEP Export`
- Diagram showing the transformation phases.

### 4. Threading Model and State Management
- Detailed explanation of the `eframe` main loop.
- Background worker threads (STEP I/O, Optimizer).
- Synchronization mechanisms (`mpsc::channel`, `Arc<AtomicBool>` for cancellation).
- Ownership rules (e.g., optimizer owns all `jagua-rs` state, isolated from UI).

### 5. Crate Deep Dives

#### 5.1 `thetta-geometry`
- Shared vocabulary: `Polygon`, `Piece`, `Placement`, `SheetConfig`, `NestingResult`.

#### 5.2 `thetta-step`
- OpenCASCADE FFI boundaries.
- Import logic: Bounding box Z-extent heuristic for thickness grouping.
- Discretization of 3D wires to 2D profiles.
- Export logic: 3D transformations for multi-sheet output.

#### 5.3 `thetta-nesting` & `jagua-rs`
- Type bridging via `conversion.rs`.
- The `jagua-rs` collision detection engine (Trigonometric queries, Quadtree).
- The three-phase optimization:
  1. **LBF (Left-Bottom-Fill):** The greedy baseline.
  2. **GLS (Guided Local Search):** Exploration, dynamic penalty weights, sampling strategies.
  3. **Compression:** Strip width minimization via binary search.
- Multi-sheet fill-and-spill orchestration.

#### 5.4 `thetta-app`
- `AppState` architecture.
- `egui` panels and `ViewportState` (zoom/pan matrix math).

### 6. Build and Configuration Notes
- macOS ARM64 specific constraints for `opencascade-sys`.
- Feature gating and workspace setup.

## Implementation

### Phase 1: Skeleton and Diagrams (Day 1)
- Create `docs/ARCHITECTURE.md`.
- Draft the outline.
- Create all ASCII diagrams (Dependency Graph, Threading Model, Data Flow) to ensure they are readable in raw markdown format.

### Phase 2: Core Components and Data Flow (Day 2)
- Write the Introduction.
- Document the crate structure and the end-to-end data pipeline.
- Detail the threading model and state synchronization.

### Phase 3: Algorithm and Crate Deep Dives (Day 3-4)
- Document `thetta-step` and the OCCT integration.
- Document `thetta-nesting`, explicitly defining LBF, GLS, and Compression.
- Explain the `jagua-rs` integration strategy and why it is treated as an isolated engine.

### Phase 4: Review and Verification (Day 5)
- Manually trace the documentation against the codebase to ensure zero drift.
- Validate that all referenced structs, modules, and file paths exist and match the document.

## Files Summary

### Modified Files

| File | Changes |
|------|---------|
| `docs/ARCHITECTURE.md` | New file. Comprehensive system documentation. (~300-500 lines) |

*Note: No code files will be modified in this sprint.*

## Definition of Done

### P0 — Must ship
- [ ] `docs/ARCHITECTURE.md` is created and committed.
- [ ] Document includes a Crate Dependency Graph (ASCII).
- [ ] Document includes a Data Flow diagram (ASCII).
- [ ] Document includes a Threading Model diagram (ASCII).
- [ ] All 5 crates (`app`, `nesting`, `step`, `geometry`, `jagua-rs`) are documented.
- [ ] The optimization algorithm (LBF -> GLS -> Compression) is clearly explained.
- [ ] The document accurately reflects the code currently in `main` (no aspirational features).

### P1 — Should ship
- [ ] Explanation of key architectural decisions (e.g., trigonometric collision vs NFP, custom GLS over existing wrappers).
- [ ] Details on testing strategies and build prerequisites (especially for OCCT on ARM64).

## Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| Document becomes immediately outdated | Medium | Keep descriptions at the architectural/module level rather than documenting individual functions. Focus on data flow and boundaries. |
| Inaccurate reflection of `jagua-rs` fork | Medium | Review the `jagua-rs` source code specifically to ensure the summary of its internal quadtree/trigonometric approaches is accurate. |
| ASCII diagrams are hard to read | Low | Keep diagrams simple and strictly bounded. Use standard box-drawing characters. |

## Security

- No security implications. This is a documentation sprint.

## Dependencies

- No code dependencies.
- Relies on the existing state of the `crates/` directory as the source of truth.

## Open Questions

1. **Diagram Format:** Sticking to plain ASCII art for maximum portability and ease of maintenance within the repo, as opposed to relying on external Mermaid rendering support.
2. **Depth of `jagua-rs` coverage:** We will treat `jagua-rs` mostly as a black box (describing its input/output boundary and primary collision strategy) rather than fully documenting its internals, since it is a forked dependency.
3. **Historical context:** We will omit historical "how we got here" sprint context from `ARCHITECTURE.md` to keep it focused on current state. Historical context will remain in `docs/sprints/`.
