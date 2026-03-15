# Sprint 005: Architecture Documentation — Claude Draft

## Overview

Create `docs/ARCHITECTURE.md` — a single, comprehensive reference document that maps the current Thetta codebase. The document answers three questions: **what** each piece of the system does, **how** data and control flow between them, and **why** the major design decisions were made. It targets both a new Rust developer onboarding to the project and the solo maintainer who needs a stable reference as the codebase evolves.

**Scope:** One new Markdown file. No code changes. The document covers all 5 workspace crates, the full STEP-to-STEP pipeline, the multi-level optimization algorithm, the threading model, key architectural decisions, build/test conventions, and extension points. All diagrams use ASCII art for portability (no external tooling required).
**Prerequisites:** Sprints 001–004 complete. The codebase is the source of truth — the document describes what exists, not what is planned.
**Deferred:** Mermaid diagrams (would require renderer), auto-generated API docs (separate concern), historical sprint narrative (belongs in sprint docs, not architecture).

## Use Cases

### UC1: New Developer Onboarding
- A Rust developer clones the repo and reads `docs/ARCHITECTURE.md` before touching any code.
- After reading, they can: identify which crate owns a given responsibility, trace the path from STEP file input to nested STEP output, understand why the optimizer thread is separate from the UI thread, and locate the right module to modify for a given task.
- **Measure of success:** The developer does not need to grep randomly to understand how pieces fit together.

### UC2: Feature Planning Reference
- Before starting a new sprint (e.g., cross-sheet GLS migration, heterogeneous sheet sizes), the maintainer consults the architecture doc to identify coupling, extension points, and affected modules.
- The doc's crate dependency graph and data flow diagram make it clear where new code would slot in and what existing interfaces it must respect.

### UC3: Bug Triage Aid
- A rendering glitch or optimizer hang is reported. The threading model section immediately shows which thread owns what state, what channels exist, and where shared state (Arc<Mutex>, Arc<AtomicBool>) could cause issues.
- The optimization pipeline section clarifies the LBF → GLS → Compression sequence so the developer can narrow down which phase produced a bad result.

### UC4: Architectural Decision Record
- A contributor asks "why multiple SPP engines instead of BPP?" or "why trigonometric collision detection instead of NFP?" The doc's decision table provides the rationale with enough context to evaluate whether the decision still holds.

## Architecture

This sprint produces a single file. The "architecture" here describes the structure of `ARCHITECTURE.md` itself — what sections it contains, their ordering, and the rationale for that ordering.

### Document Structure

```
docs/ARCHITECTURE.md
├── 1. Project Purpose & Domain
├── 2. Workspace Layout
├── 3. Crate Dependency Graph          (ASCII diagram)
├── 4. Data Flow Pipeline              (ASCII diagram)
├── 5. Threading & Concurrency Model   (ASCII diagram)
├── 6. Crate Deep Dives
│   ├── 6.1 thetta-geometry
│   ├── 6.2 jagua-rs (forked)
│   ├── 6.3 thetta-nesting
│   ├── 6.4 thetta-step
│   └── 6.5 thetta-app
├── 7. Optimization Pipeline
├── 8. Key Architectural Decisions
├── 9. Build, Configuration & Feature Flags
├── 10. Testing Patterns
├── 11. Extension Points
└── 12. Glossary
```

### Ordering Rationale

- **Top-down**: Start with purpose and the 30,000-foot view (crate graph, data flow), then zoom into individual crates, then cross-cutting concerns (optimization, decisions, build).
- **Dependency order for deep dives**: `thetta-geometry` first (no internal deps), then `jagua-rs` (foundational collision engine), then `thetta-nesting` (depends on both), then `thetta-step` (I/O layer), then `thetta-app` (depends on everything). This lets each section reference only previously-explained concepts.
- **Glossary last**: Serves as a lookup reference, not prerequisite reading.

### Content Principles

1. **Accuracy over aspiration.** Every statement must be verifiable against the current codebase. No "should" or "will" — only "is" and "does."
2. **File paths are anchors.** Every module description includes the actual file path (e.g., `crates/thetta-nesting/src/gls.rs`) so readers can jump straight to the source.
3. **Diagrams are ASCII.** Portable, diff-friendly, no rendering tooling needed. Three primary diagrams: crate dependency graph, data flow pipeline, threading model.
4. **Decisions include rationale.** For each key architectural choice, state the decision, the alternatives considered, and why this path was chosen.
5. **Independently updatable sections.** Each section is self-contained enough that a future sprint can update one section without rewriting the whole doc.

## Implementation

The implementation is organized into three phases: research/audit, writing, and verification. Since this is documentation-only, "implementation" means authoring prose, not writing code.

### Phase 1: Audit & Outline (gather all facts)

Before writing any prose, systematically verify the following against the current codebase. This prevents the document from drifting from reality.

**1.1 Crate inventory**

For each of the 5 crates, record:
- All `pub` types and their fields (from `lib.rs` and re-exported modules)
- All `pub fn` signatures on key structs
- Internal module tree (`mod` declarations in `lib.rs`/`main.rs`)
- External dependencies from `Cargo.toml`
- Feature flags (especially `thetta-step`'s `occt` feature)

**1.2 Dependency graph verification**

Read each crate's `Cargo.toml` `[dependencies]` section and draw the actual internal dependency edges. Verify against the intent doc's claimed graph:
```
thetta-app → thetta-step → thetta-geometry
           → thetta-nesting → thetta-geometry
                            → jagua-rs
           → jagua-rs
```
Note any additional edges (e.g., `thetta-app` may also depend directly on `thetta-geometry`).

**1.3 Data flow trace**

Walk the actual code path for the primary workflow:
1. `thetta-step::import::import_step()` → how STEP bytes become `Vec<Piece>`
2. `thetta-nesting::conversion::pieces_to_sp_instance()` → how `Vec<Piece>` becomes a jagua-rs `SPInstance`
3. `thetta-nesting::lbf::run_lbf()` → how LBF produces initial placement
4. `thetta-nesting::gls::run_gls()` → how GLS refines placement
5. `thetta-nesting::compression::compress()` → how strip width is minimized
6. `thetta-nesting::runner::run_nesting()` → how the pipeline is orchestrated and results sent via channel
7. `thetta-app::state::AppState::poll_updates()` → how the UI receives and displays results
8. `thetta-step::export::export_all_sheets()` → how `NestingResult` becomes output STEP files

**1.4 Threading model audit**

Identify every `std::thread::spawn` call, every `mpsc::channel` creation, every `Arc<AtomicBool>` and `Arc<Mutex<T>>` usage. Record which thread owns which data and which channels connect which threads.

**1.5 Collect architectural decisions**

Review sprint docs (SPRINT-001 through SPRINT-004) and commit messages for explicit design decisions:
- SPP-per-sheet instead of BPP (Sprint 004)
- Forked jagua-rs instead of using it as a dependency (Sprint 002)
- Trigonometric collision detection instead of NFP (inherited from jagua-rs)
- Continuous rotation support (Sprint 002)
- Feature-gated OCCT with JSON fallback (Sprint 003)
- Fill-and-spill multi-sheet overflow (Sprint 004)
- Non-blocking UI with background threads (Sprint 001)

### Phase 2: Write `docs/ARCHITECTURE.md`

Each section below specifies the exact content to include. Prose should be concise and technical — assume the reader knows Rust but not this codebase.

---

**Section 1: Project Purpose & Domain**

Content:
- One paragraph: Thetta is a desktop application for 2D irregular polygon nesting — packing arbitrarily-shaped 2D profiles extracted from 3D STEP (CAD) files into rectangular sheets to minimize material waste.
- Problem domain context: This is the Strip Packing Problem (SPP) and its multi-sheet extension. NP-hard. Real-world application: CNC cutting, laser cutting, waterjet cutting of sheet material (metal, wood, composites).
- Pipeline summary in one sentence: STEP import → 2D profile extraction → optimization (LBF + GLS + compression) → multi-sheet layout → STEP export with 3D transforms preserved.

---

**Section 2: Workspace Layout**

Content:
- ASCII tree of the `crates/` directory with one-line descriptions:
  ```
  crates/
  ├── thetta-geometry/   Shared 2D types: Polygon, Piece, Placement, SheetConfig, NestingResult
  ├── thetta-nesting/    Optimization engine: LBF, GLS, compression, multi-sheet orchestration
  ├── thetta-app/        egui desktop GUI: state machine, 2D viewport, control panels
  ├── thetta-step/       STEP file I/O via OpenCASCADE FFI (feature-gated)
  └── jagua-rs/          Forked collision detection: trigonometric queries, quadtree, SPP
  ```
- Table with columns: Crate | LOC (approximate) | Key responsibility | Internal dependencies

---

**Section 3: Crate Dependency Graph**

Content:
- ASCII box-and-arrow diagram showing all 5 crates and their dependency edges.
- Annotate the `thetta-step` → OCCT edge with `[feature = "occt"]`.
- Note that `jagua-rs` has zero internal dependencies (it's the leaf of the tree along with `thetta-geometry`).
- Include key external dependencies on each crate (egui/eframe on `thetta-app`, opencascade-sys on `thetta-step`, rand on `thetta-nesting`).

Diagram (draft):
```
┌──────────────┐
│  thetta-app  │──────────────────────────────┐
│  (egui)      │                              │
└──┬───┬───────┘                              │
   │   │                                      │
   │   ▼                                      ▼
   │  ┌────────────────┐              ┌──────────────┐
   │  │ thetta-nesting │─────────────▶│   jagua-rs   │
   │  │  (rand)        │              │  (forked)    │
   │  └───────┬────────┘              └──────────────┘
   │          │
   ▼          ▼
┌──────────────┐    ┌──────────────────┐
│ thetta-step  │───▶│ thetta-geometry  │
│ [occt]       │    │  (serde)         │
└──────────────┘    └──────────────────┘
```
Note: `thetta-app` also depends directly on `thetta-geometry` and `jagua-rs` (for `RotationRange`). `thetta-nesting` depends on both `thetta-geometry` and `jagua-rs`.

---

**Section 4: Data Flow Pipeline**

Content:
- ASCII diagram showing the full data transformation pipeline with types annotated at each stage.
- Three sub-flows:
  1. **Import flow:** STEP file → `StepModel` + `Vec<SolidRecord>` → thickness grouping → `Vec<Piece>` (with `Polygon` + metadata)
  2. **Optimization flow:** `Vec<Piece>` + `SheetConfig` → `SPInstance` → LBF placement → GLS refinement → compression → `NestingResult` (with `Vec<Sheet>`, each containing `Vec<Placement>`)
  3. **Export flow:** `NestingResult` + `StepModel` → per-sheet 3D solid transforms → output STEP files

For each transformation, include:
- The function that performs it (with file path)
- The input and output types
- What actually happens (one sentence)

---

**Section 5: Threading & Concurrency Model**

Content:
- ASCII diagram showing all threads, their lifetimes, and communication channels.
- Four threads:
  1. **Main thread (egui):** Owns `AppState`, renders UI, polls channels via `try_recv()` each frame. Never blocks.
  2. **Optimizer thread:** Spawned on "Start" click, terminated via `Arc<AtomicBool>` cancel token. Owns `NestingEngine`/`MultiSheetEngine`. Sends `NestingUpdate` via `mpsc::Sender`.
  3. **STEP import thread:** One-shot, spawned on "Load STEP". Returns `(StepModel, Vec<Piece>, Vec<ThicknessInfo>)` via channel.
  4. **STEP export thread:** One-shot, spawned on "Export". Returns `Vec<PathBuf>` or error via channel.
- Synchronization primitives:
  - `mpsc::channel<NestingUpdate>` — optimizer → main thread
  - `Arc<AtomicBool>` — main thread writes `true` to cancel optimizer
  - `Arc<Mutex<StepModel>>` — shared STEP session for export
- State ownership rules: The optimizer thread owns all jagua-rs state in isolation. The UI thread never touches jagua-rs directly. Results cross the thread boundary as owned `NestingResult` values (no sharing).

---

**Section 6: Crate Deep Dives**

For each crate, include:

**6.1 thetta-geometry** (`crates/thetta-geometry/src/`)
- Purpose: Shared type vocabulary — every other crate depends on these types.
- Key types with fields:
  - `Point2D { x: f64, y: f64 }`
  - `Polygon { outer: Vec<Point2D>, holes: Vec<Vec<Point2D>> }`
  - `Piece { id: PieceId, polygon: Polygon, source_file: Option<String>, quantity: usize }`
  - `Placement { piece_id: PieceId, x: f64, y: f64, rotation: f64 }`
  - `SheetConfig { width: f64, height: f64, separation: f64 }`
  - `NestingResult { sheets: Vec<Sheet>, unplaced: Vec<PieceId> }`
  - `Sheet { placements: Vec<Placement>, utilization: f64, strip_width: f64 }`
  - `BoundingBox { min: Point2D, max: Point2D }`
- Utility modules: `polygon_ops` (area, bounding_box, translate, rotate, offset), `test_shapes` (rectangle, circle, L/T-shape, triangle generators)
- Design note: Uses serde throughout for JSON serialization (test data, benchmark format, fallback I/O).

**6.2 jagua-rs** (`crates/jagua-rs/`)
- Purpose: Collision detection engine. Given a strip with placed polygons, answer "does polygon P at transform T collide with anything?" in microseconds.
- Origin: Forked from the jagua-rs academic project. Modified for continuous rotation support and direct Rust integration (no CLI/file-based interface).
- Core algorithm: Trigonometric direct queries — computes overlap analytically using edge-intersection tests, not precomputed No-Fit Polygons (NFPs). This enables continuous rotation without a combinatorial explosion of precomputed NFP pairs.
- Key modules:
  - `collision_detection/` — Surrogate rejection (poles + piers), quadtree spatial index, hazard proximity grid
  - `entities/` — `Item` (polygon metadata + surrogates), `Instance` (container), `PItemKey` (placed-item handle)
  - `geometry/` — `DTransformation` (rotation + translation), `RotationRange` (None/Continuous/Discrete)
  - `probs/spp/` — `SPInstance`, `SPProblem`, `SPPlacement` — the Strip Packing Problem module (the only active problem type)
  - `probs/bpp/`, `probs/mspp/` — Dead code (Bin Packing, Multi-Strip). Not used by Thetta.
  - `io/` — JSON import/export and SVG visualization
- CDE configuration: quadtree_depth=4, surrogate poles for fast bounding-circle rejection before edge-level checks.

**6.3 thetta-nesting** (`crates/thetta-nesting/src/`)
- Purpose: Multi-level optimization pipeline. Converts `Vec<Piece>` into an optimized `NestingResult` across one or more sheets.
- Module inventory (in dependency order):
  - `conversion.rs` — Bridge between thetta-geometry and jagua-rs types. `pieces_to_sp_instance()` converts `Vec<Piece>` + `SheetConfig` → `SPInstance`. Handles separation distance via polygon inflation, rotation range mapping.
  - `engine.rs` — `NestingEngine` — wraps a single jagua-rs `SPProblem`. Provides `place_item()`, `remove_item()`, `collision_cost()`, `current_result()`, `utilization()`, `strip_width()`, `fit_strip()`. Maintains `pk_map` mapping piece indices to jagua-rs `PItemKey` handles.
  - `hazard_counter.rs` — `HazardCollector` impl for counting collisions (excluding self-collisions) during placement queries.
  - `sampler.rs` — `uniform_sample()` (random position + rotation in sheet bounds) and `coordinate_descent_refine()` (iterative hill-climbing on x, y, θ with step-size halving).
  - `lbf.rs` — Left-Bottom-Fill greedy heuristic. Sorts pieces by area descending. For each piece: samples N positions on M×M grid × M rotation angles, picks bottom-left-most collision-free position. Sub-second runtime. Dual mode: `run_lbf()` (single sheet) and `run_lbf_multisheet()` (fill-and-spill across sheets).
  - `gls.rs` — Guided Local Search meta-heuristic. Core loop: select piece with highest `collision_cost × penalty_weight`, sample 1000 random positions, refine top 5 via coordinate descent, apply best. Penalty weights increase dynamically for stuck pieces. Records feasible solutions when total overlap = 0.
  - `compression.rs` — Strip width minimization via binary search. When a feasible solution exists, binary-search for the narrowest strip width where LBF can still produce a feasible layout.
  - `multi_sheet.rs` — `MultiSheetEngine` — orchestrates `Vec<NestingEngine>`, one per sheet. Tracks piece-to-sheet assignment. Fill-and-spill: tries all existing sheets before opening a new one. Aggregates results across sheets.
  - `runner.rs` — Pipeline orchestration. `run_nesting()` (single-sheet): LBF → GLS → Compression, sending `NestingUpdate` variants via channel. `run_nesting_multisheet()`: LBF multisheet → per-sheet GLS → per-sheet compression.
  - `optimizer.rs` — Mode dispatcher. `run_optimizer()` routes to single-sheet or multi-sheet runner based on `NestingMode`.
  - `benchmark.rs` — ESICUP benchmark format loader. Auto-detects benchmark JSON, sets `NestingMode::Strip`, includes known-best strip widths for reference.

**6.4 thetta-step** (`crates/thetta-step/src/`)
- Purpose: STEP file import and export via OpenCASCADE FFI. Feature-gated (`occt`); falls back to JSON when OCCT is unavailable.
- Module inventory:
  - `import.rs` — `import_step()` dispatches to OCCT or JSON. OCCT path: `STEPControl_Reader::ReadFile()` → iterate solids via `TopExp_Explorer(TopAbs_SOLID)` → compute thickness via bounding-box minimum dimension → group by thickness (tolerance-bucketed at 0.25mm) → extract largest face → discretize wire edges to polyline points (tolerance 0.5mm, max 200 points per edge). Returns `StepModel` (source path + `Vec<SolidRecord>`) and `Vec<ThicknessInfo>`.
  - `export.rs` — `export_all_sheets()` writes one STEP file per sheet. For each placed piece: retrieve original 3D solid via `StepPieceRef`, apply 2D placement as 3D transform on XY plane, write to output via `STEPControl_Writer`. JSON fallback writes polygon + thickness data.
  - `profile.rs` — `normalize_polygon()` (center to centroid, enforce CCW winding), `transform_polygon()` (apply 2D placement), edge discretization helpers.
  - `occt.rs` — Feature-gated OpenCASCADE FFI wrappers. `ProfileFrame3D` stores 3D coordinate frame (matrix + origin) for round-trip export.
- Design note: Stores `StepPieceRef` (solid key + metadata) alongside each piece so the original 3D geometry can be retrieved during export. This enables true 3D round-trip: import a STEP file → nest the 2D profiles → export STEP files with the original 3D solids repositioned.

**6.5 thetta-app** (`crates/thetta-app/src/`)
- Purpose: egui-based desktop GUI. Manages application state, renders the 2D viewport, provides control panels, and coordinates background threads.
- Module inventory:
  - `state.rs` — `AppState` — central state machine. Key fields: `pieces: Vec<Piece>`, `piece_orientations: Vec<RotationRange>`, `current_result: Option<NestingResult>`, `sheet_config: SheetConfig`, `optimizer_config: OptimizerConfig`. Threading state: `update_rx: Option<Receiver<NestingUpdate>>`, `cancel_token: Arc<AtomicBool>`. STEP state: `step_session: Arc<Mutex<StepModel>>`, `piece_refs`, `thickness_groups`. Methods: `start_optimizer()`, `stop_optimizer()`, `poll_updates()`, `start_step_import()`, `poll_step_import()`, `start_step_export()`, `poll_step_export()`, `load_json_file()`, `load_benchmark_file()`, `select_thickness()`, `delete_selected_piece()`.
  - `viewport.rs` — 2D rendering of all sheets side-by-side with horizontal spacing. Zoom (scroll wheel, 0.1×–50×), pan (middle-click or Ctrl+drag). Renders sheet boundaries, placed pieces (filled polygons via `earcutr` triangulation), selection highlight, utilization labels, used-width indicators. Click-to-select via point-in-polygon ray casting.
  - `panels.rs` — Side panel UI. Sections: STEP (Load/Export), Sheet (width/height/separation with auto-renest), Pieces (scrollable list with select/delete), Nesting (Start/Stop), Benchmark (if loaded), Stats (utilization, strip length, placed/total, sheets, GLS iteration, overlap). Emits `PanelAction` enum for event handling.
  - `main.rs` — `ThettaApp` struct implementing `eframe::App`. `update()` loop: poll file dialogs (non-blocking via `rfd`), poll nesting/STEP updates, handle Delete key, render side panel + viewport, request repaint each frame.

---

**Section 7: Optimization Pipeline**

Content:
- Detailed walkthrough of the three-phase optimization with ASCII state diagram:
  ```
  ┌─────────┐     ┌─────────┐     ┌─────────────┐
  │   LBF   │────▶│   GLS   │────▶│ Compression │
  │ (greedy │     │ (meta-  │     │  (binary    │
  │baseline)│     │heuristic│     │   search)   │
  └─────────┘     └─────────┘     └─────────────┘
  ~1 second       seconds–minutes   seconds
  ```
- **LBF:** Sort by area descending → grid-sample positions × rotation angles → pick bottom-left-most valid position. O(n × grid² × rotations) per piece. Produces a reasonable baseline fast.
- **GLS:** Iterative improvement. Each iteration: (1) select piece with highest weighted collision cost, (2) sample 1000 random placements, (3) refine top 5 via coordinate descent on (x, y, θ), (4) apply best. Penalty weights escalate for persistently-colliding pieces, forcing the search to explore diverse regions. Feasible solutions (overlap = 0) are recorded; the best is kept.
- **Compression:** Once feasible, binary-search for the minimum strip width where LBF can re-pack all pieces feasibly. Tightens the layout without re-running expensive GLS.
- **Multi-sheet extension:** LBF runs fill-and-spill (overflow to next sheet when current is full). Then GLS and compression run independently per sheet with split time budgets.
- **Channel protocol:** `NestingUpdate::Improved(result)` after each new best, `NestingUpdate::Progress{iteration, overlap, ...}` periodically, `NestingUpdate::Finished(result)` at termination.

---

**Section 8: Key Architectural Decisions**

Content — table format with columns: Decision | Alternatives Considered | Rationale

| Decision | Alternatives | Rationale |
|---|---|---|
| Multiple independent SPP engines for multi-sheet (not BPP) | jagua-rs BPP module | BPP module is dead code (untested, missing deps). Independent SPP engines are simpler, fully tested, and require zero fork modifications. |
| Forked jagua-rs (vendored in workspace) | Use as external crate dependency | Needed continuous rotation support, direct Rust API integration, and the ability to modify internal collision detection parameters. Upstream is academic and may not accept patches. |
| Trigonometric collision detection (no NFP) | No-Fit Polygon precomputation | NFP requires O(n²) polygon pairs × rotation angles. Trigonometric queries work with continuous rotation natively, no precomputation step, and achieve millions of queries/second via surrogates + quadtree. |
| Continuous rotation (any angle ∈ [0, 2π)) | Discrete angles (0°, 90°, 180°, 270°) | Better packing quality for irregular shapes. Enabled by trigonometric collision detection (NFP would make this prohibitively expensive). |
| Non-blocking UI with background threads + mpsc | Single-threaded with periodic yields; async runtime | Optimizer is CPU-bound (not I/O-bound), so async adds complexity without benefit. Background threads with channel polling are simple, correct, and keep the UI responsive. |
| Feature-gated OCCT with JSON fallback | Always require OCCT; use a lighter STEP parser | OCCT provides full 3D fidelity for round-trip export. JSON fallback enables testing and development without the heavy OCCT build dependency. Feature gate keeps the boundary clean. |
| Fill-and-spill multi-sheet overflow | Pack all sheets simultaneously; round-robin assignment | Fill-and-spill produces dense first sheets and minimizes total sheet count. Simpler than simultaneous multi-sheet optimization. Cross-sheet migration deferred to a future sprint. |
| Separation via polygon inflation (not layout constraint) | Post-placement gap enforcement; collision detection margin | Inflating polygons by `separation/2` before converting to jagua-rs items bakes the separation into the collision detection itself — no special handling needed in LBF or GLS. |

---

**Section 9: Build, Configuration & Feature Flags**

Content:
- **Workspace Cargo.toml:** Resolver v2. All 5 crates as members. Shared dependency versions via `[workspace.dependencies]`. Release profile: LTO enabled, codegen-units=1, symbols stripped — produces a single optimized binary.
- **`.cargo/config.toml`:** Sets `CMAKE_POLICY_VERSION_MINIMUM=3.5` for OCCT CMake 4.x compatibility.
- **Feature flags:**
  - `thetta-step/occt` — Enables OpenCASCADE FFI via `opencascade-sys` and `cxx`. When disabled, all STEP operations fall back to JSON. Default: enabled.
- **Running the app:** `cargo run -p thetta-app` (or `cargo run -p thetta-app --no-default-features` to skip OCCT).
- **OCCT system requirement:** OpenCASCADE must be installed (e.g., `brew install opencascade` on macOS). The `opencascade-sys` crate handles CMake-based build of the C++ bridge.

---

**Section 10: Testing Patterns**

Content:
- **Workspace tests:** `cargo test --workspace` runs all tests. OCCT tests must use `--test-threads=1` because OpenCASCADE has global state that is not thread-safe.
- **Test data convention:** STEP test files use `.json` extension to avoid triggering the OCCT reader in non-OCCT test configurations. Test shape generators in `thetta-geometry::test_shapes` provide deterministic polygon data.
- **Nesting integration tests:** Load a JSON polygon set, run the full LBF → GLS → compression pipeline, assert piece count and utilization bounds.
- **STEP round-trip tests:** Import a STEP file → extract pieces → nest → export → verify output files exist and contain valid STEP data.
- **Benchmark tests:** Load ESICUP benchmark instances (e.g., `albano`, `blaz1`), run nesting, compare strip width against known-best values.
- **Convergence tests:** Run the optimizer with a timeout and verify that it produces a feasible solution (overlap = 0) within the time budget.

---

**Section 11: Extension Points**

Content — identify the cleanest places to add new functionality:

| Extension | Where to add | Interface to implement |
|---|---|---|
| New optimization heuristic (e.g., simulated annealing) | `crates/thetta-nesting/src/` as new module | Must produce `NestingResult` and send `NestingUpdate` via channel. Wire into `runner.rs` pipeline. |
| New file format import (e.g., DXF, SVG) | `crates/thetta-step/src/` (or new crate) | Must produce `Vec<Piece>`. Wire into `AppState::load_*()`. |
| New UI panel or widget | `crates/thetta-app/src/panels.rs` | Add section in `draw_side_panel()`, emit new `PanelAction` variant. |
| New nesting mode (e.g., bin packing with fixed sheets) | `crates/thetta-nesting/src/` | Add `NestingMode` variant, implement runner function, wire into `optimizer.rs` dispatch. |
| Different collision detection backend | Replace or wrap jagua-rs usage in `conversion.rs` + `engine.rs` | Must provide equivalent `SPProblem` interface (place/remove/query). |
| Per-piece rotation constraints | Already supported via `Vec<RotationRange>` | Passed through from `AppState` → `pieces_to_sp_instance()` → jagua-rs `Item`. |

---

**Section 12: Glossary**

Content — define domain terms used throughout the doc:

| Term | Definition |
|---|---|
| **SPP (Strip Packing Problem)** | Pack items into a strip of fixed height and variable width, minimizing width. |
| **BPP (Bin Packing Problem)** | Pack items into fixed-size bins, minimizing number of bins used. |
| **LBF (Left-Bottom-Fill)** | Greedy heuristic: place each piece at its bottom-left-most valid position. |
| **GLS (Guided Local Search)** | Meta-heuristic that adds penalty weights to frequently-colliding pieces, guiding the search away from local optima. |
| **NFP (No-Fit Polygon)** | Precomputed polygon describing the locus of positions where two polygons overlap. Not used in Thetta (trigonometric queries instead). |
| **CDE (Collision Detection Engine)** | The jagua-rs subsystem that answers "does this placement collide?" queries. |
| **Surrogate** | Fast bounding approximation (poles + piers) used for early rejection before expensive edge-intersection tests. |
| **Quadtree** | Spatial index that subdivides the sheet into regions for efficient edge-lookup during collision detection. |
| **Piece** | A 2D polygon with metadata (ID, source file, quantity) representing a part to be nested. |
| **Placement** | A positioned piece: piece ID + (x, y, rotation). |
| **Sheet** | A rectangular area (width × height) containing placements. |
| **NestingResult** | The complete output of an optimization run: all sheets with their placements, plus any unplaced pieces. |
| **Fill-and-spill** | Multi-sheet strategy: fill the current sheet as much as possible, then spill remaining pieces to the next sheet. |
| **Compression** | Post-optimization phase that binary-searches for the minimum strip width that still admits a feasible packing. |
| **STEP** | ISO 10303 STEP file format — standard CAD interchange format for 3D geometry. |
| **OCCT (OpenCASCADE)** | Open-source 3D CAD kernel used for STEP file parsing and writing. |

### Phase 3: Verification

After writing the document, verify it systematically:

**3.1 File path audit**
- Extract every file path mentioned in the document.
- Verify each path exists in the current codebase via `ls` or glob.
- Fix or remove any stale references.

**3.2 Type accuracy check**
- For each type documented (struct fields, enum variants, function signatures), verify against the actual source.
- Pay particular attention to `NestingResult`, `NestingUpdate`, `AppState`, and `NestingEngine` — these are the most frequently referenced and most likely to have drifted.

**3.3 Dependency graph check**
- Read each crate's `Cargo.toml` and verify the graph matches.

**3.4 Cross-reference with sprint docs**
- Ensure architectural decisions reference the correct sprint where they were made.

**3.5 Build verification**
- Run `cargo test --workspace` to confirm no code was accidentally modified.
- Run `cargo build -p thetta-app` to confirm the project still compiles.

**3.6 Readability pass**
- Read the full document top to bottom as a new developer would.
- Flag any section that requires knowledge not previously introduced in the document.
- Ensure ASCII diagrams render correctly in a fixed-width font (GitHub Markdown preview).

## Files Summary

| File | Action | Description |
|---|---|---|
| `docs/ARCHITECTURE.md` | **Create** | The sole deliverable. ~800–1200 lines of Markdown covering all 12 sections described above. |

No other files are created, modified, or deleted.

## Definition of Done

### D1: Document exists and is complete
- [ ] `docs/ARCHITECTURE.md` exists at the specified path.
- [ ] All 12 sections are present: Purpose, Workspace Layout, Dependency Graph, Data Flow, Threading, Crate Deep Dives (×5), Optimization Pipeline, Architectural Decisions, Build/Config, Testing, Extension Points, Glossary.
- [ ] Three ASCII diagrams are included: crate dependency graph, data flow pipeline, threading model.

### D2: Accuracy verified against codebase
- [ ] Every file path referenced in the document exists in the current repo.
- [ ] Every type (struct, enum, function signature) documented matches the actual source code.
- [ ] Dependency graph matches actual `Cargo.toml` declarations.
- [ ] Threading model matches actual `thread::spawn` / `mpsc::channel` / `Arc` usage.

### D3: Architectural decisions are documented
- [ ] At least 7 key decisions are listed with alternatives and rationale.
- [ ] Decisions reference the sprint or context where they were made.

### D4: No code changes
- [ ] `cargo test --workspace` passes.
- [ ] `git diff` (excluding `docs/`) shows no changes.

### D5: Readability
- [ ] A Rust developer unfamiliar with the project can read the document top-to-bottom without needing to reference external material (beyond Rust language knowledge).
- [ ] ASCII diagrams render correctly in GitHub Markdown preview (fixed-width font).
- [ ] No section requires understanding a concept introduced later in the document.

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| **Document drifts from code over time** | High | Medium | Structure sections independently so future sprints can update individual sections. Include file paths as anchors for easy verification. |
| **Type signatures documented inaccurately** | Medium | Medium | Phase 3 verification step explicitly checks every type against source. Use grep to extract actual signatures. |
| **Excessive detail makes document hard to maintain** | Medium | Low | Keep descriptions at the module level, not individual function level. Reference source files for details. |
| **ASCII diagrams break in narrow terminals** | Low | Low | Keep diagrams under 80 columns. Test in GitHub preview. |

## Security

No security implications. This sprint creates a documentation file only — no code changes, no new dependencies, no user input handling, no network access.

One consideration: Ensure the architecture document does not inadvertently document security-sensitive details (e.g., file paths containing credentials, internal API keys). Review the final document for any accidental inclusion of sensitive information. Given Thetta is a local desktop application with no network component, this risk is negligible.

## Dependencies

### Internal
- **All prior sprints (001–004):** The document describes the cumulative architecture. All features must be implemented and stable before documentation is accurate.

### External
- **None.** This sprint has no external dependencies. The document is authored in Markdown, which requires no additional tooling.

### Tooling
- A text editor (for writing Markdown).
- `cargo test --workspace` (for verification — already available).
- Git (for diffing to confirm no code changes).

## Open Questions

### OQ1: Depth of jagua-rs internal documentation
**Question:** Should the architecture doc explain jagua-rs collision detection internals (surrogate rejection, quadtree traversal, hazard proximity grid) in detail, or treat jagua-rs as a black box with a documented API surface?

**Recommendation:** Document at the module level — explain what each jagua-rs module does and why it exists, but don't reproduce the algorithm pseudocode. The forked jagua-rs has its own README and academic paper for deep algorithmic details. Reference these instead.

### OQ2: Should the document include quantitative performance data?
**Question:** Include benchmarks (e.g., "LBF runs in ~200ms for 50 pieces", "GLS achieves 85%+ utilization on ESICUP benchmarks")?

**Recommendation:** Include approximate order-of-magnitude timings (sub-second, seconds, minutes) for pipeline stages, but not specific benchmark numbers — those change with hardware and code improvements. Reference the ESICUP benchmark suite for reproducible evaluation.

### OQ3: Should we include a "how we got here" sprint history section?
**Question:** The intent doc's open questions ask whether to include sprint progression history.

**Recommendation:** No. Sprint history belongs in `docs/sprints/`. The architecture doc should describe the current state only, with a brief mention that the system was built incrementally across 4 sprints. Link to the sprint docs for historical context.

### OQ4: Mermaid vs ASCII diagrams
**Question:** The intent doc asks about diagram format.

**Recommendation:** ASCII art. Reasons: (1) diffs cleanly in git, (2) renders without tooling in any terminal or editor, (3) GitHub renders Mermaid but many other Markdown viewers don't, (4) forces diagrams to stay simple and focused. Keep diagrams under 80 columns wide.
