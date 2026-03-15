# Thetta Architecture
## 1. Project Purpose & Domain
Thetta is a desktop Rust application for 2D irregular polygon nesting: it imports
profiles extracted from 3D STEP solids, packs those 2D parts into rectangular
sheets to reduce material waste, and exports transformed solids back to STEP for
CNC, laser, or waterjet workflows.
The optimization domain is strip packing and fixed-sheet packing for irregular
geometry.
In complexity terms, these packing problems are NP-hard.
Thetta uses practical heuristics instead of exact search.
Current production pipeline in this repository is:
STEP import -> 2D profile extraction -> LBF baseline -> GLS improvement ->
compression -> multi-sheet layout aggregation -> STEP export.
SPP context in this codebase means one strip with fixed height and adjustable
length.
Multi-sheet mode is implemented by orchestrating multiple independent SPP
engines.
BPP modules exist in the forked engine source tree but are intentionally not in
use by the Thetta crates.
The entire pipeline is implemented in Rust workspace crates under `crates/`.
## 2. Workspace Layout
### 2.1 Crates Tree
```text
crates/
|-- thetta-geometry/   # Core 2D types + polygon utilities shared by all crates
|-- jagua-rs/          # Vendored collision engine fork (SPP API consumed)
|-- thetta-nesting/    # Nesting engine orchestration (LBF, GLS, compression)
|-- thetta-step/       # STEP import/export + OCCT integration + JSON fallback
`-- thetta-app/        # eframe/egui desktop UI orchestration
```
### 2.2 Crate Responsibilities
| Crate | Key responsibility | Internal dependencies |
|---|---|---|
| `thetta-geometry` | Canonical 2D model (`Piece`, `Polygon`, `NestingResult`) and geometry helpers | none |
| `jagua-rs` | Collision detection and SPP domain structures consumed by nesting layer | none (workspace internal) |
| `thetta-nesting` | Converts Thetta types to SPP, runs LBF/GLS/compression, supports multi-sheet | `thetta-geometry`, `jagua-rs` |
| `thetta-step` | Imports STEP/JSON into `Piece` sets and exports placed sheets | `thetta-geometry` |
| `thetta-app` | UI state, rendering, panel controls, threading, and app-level orchestration | `thetta-geometry`, `thetta-nesting`, `thetta-step`, `jagua-rs` |
### 2.3 Top-Level Build Metadata Anchors
Workspace members and release profile are in:
- `Cargo.toml`
CMake compatibility override for OCCT builds is in:
- `.cargo/config.toml`
## 3. Crate Dependency Graph
```text
                   +----------------------+
                   |   External Systems   |
                   |----------------------|
                   | OpenCASCADE runtime  |
                   +----------+-----------+
                              ^
                              | [feature = "occt"]
                              |
+----------------+    +-------+--------+    +-------------------+
| thetta-geometry|    |  thetta-step   |    |     jagua-rs      |
| (leaf crate)   |    |  (STEP I/O)    |    | (vendored fork)   |
+--------+-------+    +-------+--------+    +---------+---------+
         ^                    ^                       ^
         |                    |                       |
         |                    |                       |
         +---------+----------+-----------+-----------+
                   |                      |
                   |                      |
           +-------+--------+             |
           | thetta-nesting |-------------+
           | (optimizer)    |
           +-------+--------+
                   ^
                   |
           +-------+--------+
           |   thetta-app   |
           |  (eframe UI)   |
           +----------------+
```
### 3.1 Verified Dependency Edges
From `Cargo.toml` files:
- `crates/thetta-nesting/Cargo.toml` depends on `thetta-geometry` and `jagua-rs`
- `crates/thetta-step/Cargo.toml` depends on `thetta-geometry`
- `crates/thetta-app/Cargo.toml` depends on all other workspace crates
- `crates/thetta-app/Cargo.toml` enables `thetta-step` feature `occt`
### 3.2 External Dependency Notes by Crate
- `thetta-app`: `eframe`, `egui`, `rfd`, `earcutr`
- `thetta-step`: optional `opencascade-sys` and `cxx` behind `occt`
- `thetta-nesting`: `rand` for GLS sampling
- `jagua-rs`: `anyhow`, `ordered-float`, `serde`
- `thetta-geometry`: `serde`
## 4. Data Flow Pipeline
### 4.1 End-to-End Pipeline Diagram
```text
Import flow:
STEP/JSON file
   |
   v
import_step(...)
(crates/thetta-step/src/import.rs)
   |
   v
StepImportResult { model, pieces, piece_refs, thicknesses }
Optimization flow:
Vec<Piece> + SheetConfig + orientations
   |
   v
run_optimizer(...)
(crates/thetta-nesting/src/optimizer.rs)
   |
   +--> run_nesting(...) or run_nesting_multisheet(...)
        (crates/thetta-nesting/src/runner.rs)
           |
           +--> LBF
           +--> GLS
           +--> compression
           v
NestingResult + NestingUpdate stream
Export flow:
StepModel + StepPieceRef + NestingResult
   |
   v
export_all_sheets(...)
(crates/thetta-step/src/export.rs)
   |
   v
STEP files (or JSON fallback files)
```
### 4.2 Transformation Table (Import Sub-Flow)
| Function + file path | Input types | Output types | What it does |
|---|---|---|---|
| `import_step` in `crates/thetta-step/src/import.rs` | `&Path`, `Option<f64>`, `&ExtractionConfig` | `Result<StepImportResult, StepImportError>` | Dispatches to OCCT importer for `.step`/`.stp` when feature is active, otherwise JSON fallback importer |
| `import_step_occt` in `crates/thetta-step/src/import.rs` | same as above | same as above | Reads solids through OCCT wrapper, extracts planar 2D profiles, computes thickness groups |
| `import_step_json` in `crates/thetta-step/src/import.rs` | same as above | same as above | Parses JSON test payload into `StepModel`, thickness groups, and extracted pieces |
| `extract_pieces_for_thickness` in `crates/thetta-step/src/import.rs` | `&StepModel`, `f64`, `&ExtractionConfig` | `Result<(Vec<Piece>, Vec<StepPieceRef>), StepImportError>` | Filters solids by thickness bucket and generates piece IDs + mapping references |
### 4.3 Transformation Table (Optimization Sub-Flow)
| Function + file path | Input types | Output types | What it does |
|---|---|---|---|
| `run_optimizer` in `crates/thetta-nesting/src/optimizer.rs` | `Vec<Piece>`, `SheetConfig`, `Option<Vec<RotationRange>>`, `OptimizerConfig`, `NestingMode`, `Sender<NestingUpdate>`, `Arc<AtomicBool>` | `()` | Routes to strip or multi-sheet pipeline |
| `run_nesting` in `crates/thetta-nesting/src/runner.rs` | same core optimization inputs | `()` via channel updates | Runs LBF baseline, GLS refinement, then strip compression |
| `run_nesting_multisheet` in `crates/thetta-nesting/src/runner.rs` | same core optimization inputs | `()` via channel updates | Runs fill-and-spill LBF, per-sheet GLS, per-sheet compression |
| `run_lbf` in `crates/thetta-nesting/src/lbf.rs` | `&mut NestingEngine`, `LbfConfig`, optional cancel | `NestingResult` | Greedy area-sorted placement baseline |
| `GLSOptimizer::run` in `crates/thetta-nesting/src/gls.rs` | callbacks + duration + cancel | `NestingResult` | Guided local search with penalty weights and coordinate-descent refinement |
| `compress_strip` in `crates/thetta-nesting/src/compression.rs` | pieces/sheet/step/orientations/cancel/max_width | `NestingResult` | Binary-search-like tightening using repeated LBF under width changes |
### 4.4 Transformation Table (Export Sub-Flow)
| Function + file path | Input types | Output types | What it does |
|---|---|---|---|
| `export_all_sheets` in `crates/thetta-step/src/export.rs` | `&StepModel`, `&[StepPieceRef]`, `&NestingResult`, `&SheetConfig`, `&Path`, `&str` | `Result<Vec<PathBuf>, StepExportError>` | Iterates sheets and writes one output file per sheet |
| `export_sheet` in `crates/thetta-step/src/export.rs` | model + refs + single `&Sheet` + config + output path | `Result<(), StepExportError>` | Chooses OCCT or JSON export based on feature + source extension |
| `export_sheet_occt` in `crates/thetta-step/src/export.rs` | same | `Result<(), StepExportError>` | Re-reads source STEP, transforms original solids with placement frame data, writes STEP |
| `export_sheet_json` in `crates/thetta-step/src/export.rs` | same | `Result<(), StepExportError>` | Serializes transformed 2D polygons and thickness metadata as JSON |
### 4.5 Bridge Conversion Functions
Core conversion boundary from Thetta polygons to jagua-rs SPP instance:
- `pieces_to_sp_instance` in `crates/thetta-nesting/src/conversion.rs`
Transformation conversion helpers:
- `to_internal_transformation` in `crates/thetta-nesting/src/conversion.rs`
- `to_external_transformation` in `crates/thetta-nesting/src/conversion.rs`
Rotation policy bridge:
- `rotation_range_to_samples` in `crates/thetta-nesting/src/conversion.rs`
## 5. Threading & Concurrency Model
### 5.1 Threading Diagram (App Runtime)
```text
+----------------------- Main UI Thread (egui) -----------------------+
| Owns AppState, renders panels/viewport, polls channels every frame  |
|                                                                      |
|  poll_updates()      poll_step_import()      poll_step_export()      |
+-----+--------------------+------------------------+------------------+
      |                    |                        |
      | mpsc::Receiver     | mpsc::Receiver         | mpsc::Receiver
      v                    v                        v
+-----+------+      +------+-------+         +------+-------+
| Optimizer  |      | STEP Import  |         | STEP Export  |
| Thread     |      | Thread       |         | Thread       |
| run_optimizer      import_step              export_all_sheets
| cancel via Arc<AtomicBool>                 StepModel via Arc<Mutex>
+------------+      +--------------+         +--------------+
```
### 5.2 Primary Thread Roles
Main/UI thread:
- Runs `eframe::App::update` in `crates/thetta-app/src/main.rs`
- Owns mutable `AppState` in `crates/thetta-app/src/state.rs`
- Polls all receivers with non-blocking `try_recv`
- Never blocks on joins or blocking channel `recv`
- Calls `ctx.request_repaint()` every frame
Optimizer thread:
- Spawned in `AppState::start_optimizer` in `crates/thetta-app/src/state.rs`
- Calls `run_optimizer` from `thetta-nesting`
- Reports `NestingUpdate::{Improved, Progress, Finished}` via `mpsc::Sender`
- Reads cancellation from `Arc<AtomicBool>`
STEP import thread:
- Spawned in `AppState::start_step_import` in `crates/thetta-app/src/state.rs`
- Calls `import_step`
- Sends one result (`StepImportResult` or `StepImportError`)
STEP export thread:
- Spawned in `AppState::start_step_export` in `crates/thetta-app/src/state.rs`
- Clones `StepModel` from `Arc<Mutex<StepModel>>`
- Calls `export_all_sheets`
- Sends one result (`Vec<PathBuf>` or `StepExportError`)
### 5.3 Synchronization Primitives in Active Use
- `std::sync::mpsc::channel` for optimizer/import/export update streams
- `Arc<AtomicBool>` for cooperative optimizer cancellation
- `Arc<Mutex<StepModel>>` for shared STEP model session state across UI/export
### 5.4 State Ownership Rules
- `AppState` is owned by main thread only
- Worker threads operate on cloned inputs or guarded shared `StepModel`
- No worker mutates UI state directly
- Worker to UI communication is message passing only
- Optimizer internal jagua-rs state is thread-confined to optimizer thread
### 5.5 Auxiliary Background Threads
`crates/thetta-app/src/native_dialog.rs` also spawns short-lived threads for
non-blocking native open/save dialogs.
Those threads communicate using one-shot channels but are not part of the core
optimization/import/export pipeline.
## 6. Crate Deep Dives
### 6.1 `thetta-geometry` (leaf)
#### Purpose
Defines the canonical 2D data model shared across app, optimizer, and STEP I/O.
This crate has no internal workspace dependencies.
#### Module Inventory
- `crates/thetta-geometry/src/lib.rs`
- `crates/thetta-geometry/src/types.rs`
- `crates/thetta-geometry/src/polygon_ops.rs`
- `crates/thetta-geometry/src/test_shapes.rs`
#### Key Public Types
From `crates/thetta-geometry/src/types.rs`:
- `Point2D`
- `Polygon`
- `PieceId`
- `Piece`
- `Placement`
- `BoundingBox`
- `SheetConfig`
- `NestingResult`
- `Sheet`
#### Key Public Functions
From `crates/thetta-geometry/src/polygon_ops.rs`:
- `area`
- `bounding_box`
- `translate`
- `rotate`
- `offset`
#### Design Notes
- `PieceId` is a stable wrapper struct over `u64`
- `NestingResult` explicitly supports multi-sheet via `Vec<Sheet>`
- `Sheet.strip_length` is used both in strip mode and as per-sheet used width
- `offset` is a lightweight centroid-radial approximation, not full Minkowski
#### Test Surface
- Unit tests in `polygon_ops.rs`
- Fixtures in `test_shapes.rs`
### 6.2 `jagua-rs` (forked, API surface only)
#### Purpose (Black Box Contract)
`jagua-rs` provides the collision-detection and SPP placement API that Thetta
calls from `thetta-nesting`.
For Thetta architecture purposes, this crate is treated as a black box with a
stable consumed surface.
#### Consumed API Surface in This Workspace
Core modules and items referenced by `thetta-nesting`:
- `jagua_rs::collision_detection::CDEConfig`
- `jagua_rs::entities::{Instance, PItemKey}`
- `jagua_rs::geometry::DTransformation`
- `jagua_rs::geometry::geo_enums::RotationRange`
- `jagua_rs::geometry::geo_traits::Transformable`
- `jagua_rs::geometry::fail_fast::SPSurrogateConfig`
- `jagua_rs::io::import::{Importer, ext_to_int_transformation}`
- `jagua_rs::io::export::int_to_ext_transformation`
- `jagua_rs::io::ext_repr::{ExtItem, ExtShape, ExtSPolygon}`
- `jagua_rs::probs::spp::entities::{SPInstance, SPPlacement, SPProblem}`
- `jagua_rs::probs::spp::io::ext_repr::{ExtSPInstance, ExtItem as ExtSPItem}`
- `jagua_rs::probs::spp::io::import_instance`
#### Source Anchors for Public Surface
- `crates/jagua-rs/src/lib.rs`
- `crates/jagua-rs/src/entities/mod.rs`
- `crates/jagua-rs/src/geometry/mod.rs`
- `crates/jagua-rs/src/io/mod.rs`
- `crates/jagua-rs/src/probs/mod.rs`
- `crates/jagua-rs/src/probs/spp/entities/mod.rs`
#### Feature-Gating Observed in Fork
- `crates/jagua-rs/Cargo.toml` defines features: `spp`, `bpp`, `mspp`
- `crates/jagua-rs/src/probs/mod.rs` exports only `spp` behind `#[cfg(feature="spp")]`
#### Unused Subtrees in Current Thetta Build
These folders exist in the fork but are not used by Thetta runtime path:
- `crates/jagua-rs/src/probs/bpp/`
- `crates/jagua-rs/src/probs/mspp/`
#### Algorithmic Internals Reference
Algorithm internals are intentionally out of scope here.
Reference for deeper internals is the jagua-rs paper linked from:
- `research/repos/jagua-rs/README.md`
The README cites DOI `10.1287/ijoc.2024.1025` and an arXiv manuscript.
### 6.3 `thetta-nesting`
#### Purpose
Implements all nesting orchestration:
- conversion from Thetta geometry into SPP instance
- single-strip and multi-sheet pipelines
- LBF heuristic
- GLS meta-heuristic
- compression tightening
- benchmark ingestion and runner CLI
#### Module Inventory
From `crates/thetta-nesting/src/lib.rs`:
- `benchmark` -> `crates/thetta-nesting/src/benchmark.rs`
- `compression` -> `crates/thetta-nesting/src/compression.rs`
- `conversion` -> `crates/thetta-nesting/src/conversion.rs`
- `engine` -> `crates/thetta-nesting/src/engine.rs`
- `gls` -> `crates/thetta-nesting/src/gls.rs`
- `hazard_counter` -> `crates/thetta-nesting/src/hazard_counter.rs`
- `lbf` -> `crates/thetta-nesting/src/lbf.rs`
- `multi_sheet` -> `crates/thetta-nesting/src/multi_sheet.rs`
- `optimizer` -> `crates/thetta-nesting/src/optimizer.rs`
- `runner` -> `crates/thetta-nesting/src/runner.rs`
- `sampler` -> `crates/thetta-nesting/src/sampler.rs`
Binary entrypoint:
- `crates/thetta-nesting/src/bin/benchmark.rs`
Tests:
- `crates/thetta-nesting/tests/integration.rs`
- `crates/thetta-nesting/tests/engine_integration.rs`
- `crates/thetta-nesting/tests/benchmark_smoke.rs`
- `crates/thetta-nesting/tests/multi_sheet.rs`
#### Key Public Types
From crate public modules:
- `NestingEngine` (`engine.rs`)
- `MultiSheetEngine` (`multi_sheet.rs`)
- `LbfConfig` (`lbf.rs`)
- `OptimizerConfig` (`gls.rs`)
- `GLSOptimizer` (`gls.rs`)
- `NestingMode` (`runner.rs`)
- `NestingUpdate` (`runner.rs`)
- `BenchmarkInstance` (`benchmark.rs`)
- `BenchmarkMeta` (`benchmark.rs`)
#### Core Runtime Objects
`NestingEngine` (`engine.rs`) owns:
- immutable list of `Piece` metadata
- `SheetConfig`
- jagua-rs `SPInstance`
- jagua-rs `SPProblem`
- placement key map `Vec<Option<PItemKey>>`
- per-piece `RotationRange`
`MultiSheetEngine` (`multi_sheet.rs`) owns:
- vector of `NestingEngine` (one per sheet)
- global piece list
- assignment map piece index -> sheet index
- `PieceId` to index map
#### Design Notes
- Collision cost is measured by counting colliding hazards via
  `CountingCollector` in `hazard_counter.rs`
- `run_lbf_multisheet` uses fill-and-spill strategy
- Per-sheet GLS uses sub-problem extraction and write-back in `runner.rs`
- Compression uses repeated LBF under width changes, not GLS
- Multi-sheet compression caps strip width to sheet width using
  `max_strip_width` parameter
#### Conversion Boundary Notes
- Uses `Importer` with configured separation from `SheetConfig.separation`
- Applies jagua-rs transform conversion helpers for coordinate correctness
- Rotation ranges are translated into sampled radians for LBF and GLS
### 6.4 `thetta-step`
#### Purpose
Provides import and export boundaries between CAD files and 2D nesting model.
Supports two modes:
- OCCT-backed STEP read/write when feature `occt` is enabled and input is STEP
- JSON fallback mode for tests and feature-disabled environments
#### Module Inventory
From `crates/thetta-step/src/lib.rs`:
- `export` -> `crates/thetta-step/src/export.rs`
- `import` -> `crates/thetta-step/src/import.rs`
- `occt` -> `crates/thetta-step/src/occt.rs`
- `profile` -> `crates/thetta-step/src/profile.rs`
Tests:
- `crates/thetta-step/tests/profile_tests.rs`
- `crates/thetta-step/tests/thickness_tests.rs`
- `crates/thetta-step/tests/roundtrip_test.rs`
- `crates/thetta-step/tests/full_pipeline_test.rs`
- `crates/thetta-step/tests/occt_integration_test.rs`
- `crates/thetta-step/tests/gen_test_step.rs`
#### Key Public Types
From `import.rs` and `lib.rs` re-exports:
- `ExtractionConfig`
- `StepImportError`
- `StepImportResult`
- `StepModel`
- `StepPieceRef`
- `StepSolidKey`
- `ThicknessInfo`
From `export.rs` and `lib.rs` re-exports:
- `StepExportError`
#### Key Public Functions
Re-exported in `lib.rs`:
- `import_step`
- `extract_pieces_for_thickness`
- `export_sheet`
- `export_all_sheets`
#### OCCT Boundary Notes
- Unsafe/raw OCCT calls are confined to `occt.rs`
- `import.rs` and `export.rs` call safe wrappers
- `SolidFrame` stores 3D extraction frame + 2D center for accurate re-transform
#### JSON Fallback Notes
- JSON import/export uses same `StepModel` and `StepPieceRef` contract
- Tests intentionally use `.json` filenames to force JSON path and avoid OCCT
### 6.5 `thetta-app`
#### Purpose
Provides desktop UI and orchestrates all asynchronous workflows.
This crate is the integration point where geometry, nesting, and STEP I/O meet.
#### Module Inventory
- `crates/thetta-app/src/main.rs`
- `crates/thetta-app/src/state.rs`
- `crates/thetta-app/src/panels.rs`
- `crates/thetta-app/src/viewport.rs`
- `crates/thetta-app/src/native_dialog.rs`
Examples:
- `crates/thetta-app/examples/test_dialog.rs`
- `crates/thetta-app/examples/test_nesting.rs`
- `crates/thetta-app/examples/dump_profiles.rs`
- `crates/thetta-app/examples/time_lbf.rs`
- `crates/thetta-app/examples/rocket_convergence.rs`
#### Key Public/Internal Types Driving Architecture
- `AppState` (`state.rs`)
- `NestingStats` (`state.rs`)
- `LoadedFile` (`state.rs`)
- `LoadedBenchmark` (`state.rs`)
- `ViewportState` (`viewport.rs`)
- `PanelAction` (`panels.rs`)
#### App Orchestration Notes
`main.rs` update loop each frame:
1. poll optimizer/import/export channels
2. poll non-blocking dialog channels
3. process UI actions from panel
4. render viewport
5. request repaint
`state.rs` centralizes:
- optimizer lifecycle start/stop/poll
- STEP import lifecycle start/poll/apply
- STEP export lifecycle start/poll/status
- piece deletion and auto-renest
- thickness group switching
`viewport.rs` implements:
- side-by-side sheet rendering
- zoom/pan
- triangulated polygon fill using `earcutr`
- piece hit testing and selection
`panels.rs` implements:
- STEP controls
- sheet controls
- piece list controls
- nesting controls
- benchmark/status display
## 7. Optimization Pipeline
### 7.1 High-Level Sequence
The optimization pipeline in active use is:
1. LBF baseline
2. GLS refinement
3. Compression pass
In strip mode this pipeline runs on one engine.
In sheet mode it runs as multi-sheet orchestration with per-sheet refinement.
### 7.2 Phase 1: LBF (Greedy Baseline)
Implementation entry points:
- `run_lbf` in `crates/thetta-nesting/src/lbf.rs`
- `run_lbf_multisheet` in `crates/thetta-nesting/src/lbf.rs`
Behavior:
- Sort pieces by area descending
- Sample candidate grid positions
- Sample candidate rotations from per-piece range
- Pick lowest-left feasible candidate (y priority, then x)
- Place immediately when feasible
- Fit strip after placement phase
Operational role:
- Produces quick first feasible/near-feasible layout
- Sends immediate `NestingUpdate::Improved` to UI
- Establishes seed placements for later phases
Order-of-magnitude runtime target in this codebase:
- around one second for typical interactive runs
### 7.3 Phase 2: GLS (Meta-Heuristic)
Implementation entry points:
- `GLSOptimizer` in `crates/thetta-nesting/src/gls.rs`
- called from `run_nesting` and `run_nesting_multisheet` in `runner.rs`
Behavior:
- Initializes penalty weights per piece
- Re-randomizes problematic or unplaced pieces
- Iteratively chooses highest weighted collision offender
- Samples candidate positions and rotations randomly
- Applies coordinate-descent local refinement for top candidates
- Updates penalties for still-colliding pieces
- Tracks best feasible layout by strip length
Progress reporting:
- Emits `NestingUpdate::Progress` each iteration
- Emits `NestingUpdate::Improved` when better feasible layout appears
Operational runtime profile:
- seconds to minutes depending on time budget (`OptimizerConfig.time_limit_secs`)
### 7.4 Phase 3: Compression
Implementation entry point:
- `compress_strip` in `crates/thetta-nesting/src/compression.rs`
Behavior:
- Rebuilds engine from pieces
- Runs LBF to get a baseline width
- Iteratively tests narrower strip widths (bounded iterations)
- Keeps narrower feasible candidate when all pieces remain placed
Mode behavior:
- strip mode allows expansion beyond initial sheet width
- multi-sheet mode uses `max_strip_width = Some(sheet.width)`
Operational runtime profile:
- typically seconds
### 7.5 Multi-Sheet Extension Strategy
Implementation entry points:
- `run_nesting_multisheet` in `crates/thetta-nesting/src/runner.rs`
- `MultiSheetEngine` in `crates/thetta-nesting/src/multi_sheet.rs`
Strategy:
- Fill-and-spill LBF across existing sheets before opening new sheet
- Split total GLS budget across sheets
- Run per-sheet GLS independently
- Write back feasible per-sheet placements
- Run per-sheet compression with width cap
Current non-goals (deferred architecture):
- cross-sheet migration during GLS
- global sheet-count minimization during compression
### 7.6 Channel Protocol (`NestingUpdate`)
Defined in `crates/thetta-nesting/src/runner.rs`:
- `Improved(NestingResult)`
- `Finished(NestingResult)`
- `Progress { iteration, total_overlap, best_overlap, feasible_count }`
Consumption in UI:
- handled in `AppState::poll_updates` in `crates/thetta-app/src/state.rs`
## 8. Key Architectural Decisions
### 8.1 Decision Table
| Decision | Alternatives considered | Rationale |
|---|---|---|
| 1. Multi-sheet uses multiple SPP engines | Enable and rely on jagua-rs BPP path | `probs/mod.rs` exports SPP only in current fork build path; multi-SPP reused working SPP logic with lower integration risk |
| 2. Vendor jagua-rs in workspace | Remote git dependency only | Local fork keeps API stable for current code, enables direct patching and deterministic builds |
| 3. Trigonometric collision checks via jagua-rs CDE | NFP-based custom engine | Existing jagua-rs quadtree/CDE path already integrated and tested in Thetta |
| 4. Continuous rotation support by default | Discrete fixed angles only | Sheet-metal parts are often orientation-flexible; continuous mode improves feasible search space |
| 5. Non-blocking UI with worker threads + mpsc | Single-threaded blocking compute in UI loop | Keeps egui responsive and supports progressive results |
| 6. OCCT behind feature gate with JSON fallback | Hard dependency on OCCT for all builds | Allows development/testing where OCCT is unavailable and keeps CI/local workflows flexible |
| 7. Fill-and-spill overflow for multi-sheet LBF | Always open new sheet on first failure | Trying existing sheets first reduces wasted gaps and sheet count in practice |
| 8. Separation enforced at geometry conversion boundary | Post-placement collision padding | Conversion-level separation makes collision checks consistent across heuristics |
| 9. Message-driven ownership boundaries | Shared mutable optimizer state in UI thread | Simplifies reasoning: UI owns UI state, workers own compute state |
### 8.2 Rationale Anchors in Sprint History
Decision context is documented in:
- `docs/sprints/SPRINT-001.md`
- `docs/sprints/SPRINT-002.md`
- `docs/sprints/SPRINT-003.md`
- `docs/sprints/SPRINT-004.md`
Specific rationale source for multi-SPP over BPP:
- `docs/sprints/SPRINT-004.md` section "Key Design Decision: Multiple SPP Instances"
### 8.3 Tradeoffs Captured by Current Architecture
- Multi-SPP increases memory with piece duplication per sheet engine
- Per-sheet independent GLS/compression is simpler but not globally optimal
- JSON fallback broadens testability but is not a production CAD substitute
- Continuous rotation increases search space and can increase runtime variance
## 9. Build, Configuration & Feature Flags
### 9.1 Workspace Build Configuration
From root `Cargo.toml`:
- resolver: `"2"`
- workspace members: 5 crates
- release profile:
  - `lto = true`
  - `codegen-units = 1`
  - `strip = true`
- release build override:
  - `opt-level = 3`
### 9.2 CMake/OCCT Compatibility Config
From `.cargo/config.toml`:
- `CMAKE_POLICY_VERSION_MINIMUM = "3.5"`
Purpose:
- enables OCCT source build compatibility with newer CMake toolchains
### 9.3 Feature Flags
`thetta-step` features from `crates/thetta-step/Cargo.toml`:
- `default = []`
- `occt = ["dep:opencascade-sys", "dep:cxx"]`
Important observed behavior in this workspace:
- `thetta-step` does not enable `occt` by default
- `thetta-app` enables it explicitly via dependency declaration in
  `crates/thetta-app/Cargo.toml`
Fallback behavior:
- when OCCT path is not selected, import/export falls back to JSON path in
  `crates/thetta-step/src/import.rs` and `crates/thetta-step/src/export.rs`
### 9.4 Platform and Runtime Expectations
Primary development target (from sprint context and current setup):
- macOS ARM64
OCCT system dependency for OCCT-backed mode:
- `brew install opencascade`
### 9.5 Common Commands
Run app:
```bash
cargo run -p thetta-app
```
Run optimizer benchmark CLI:
```bash
cargo run -p thetta-nesting --bin benchmark -- --all --time 10
```
Run workspace tests:
```bash
cargo test --workspace
```
When running OCCT-heavy tests, single-threaded execution avoids shared-state
issues:
```bash
cargo test -p thetta-step --features occt -- --test-threads=1
```
## 10. Testing Patterns
### 10.1 Test Entry Pattern
Primary suite command:
- `cargo test --workspace`
This runs unit and integration tests across all workspace crates.
### 10.2 Test Categories in Repository
Geometry unit tests:
- `crates/thetta-geometry/src/polygon_ops.rs`
Nesting integration tests:
- `crates/thetta-nesting/tests/integration.rs`
- `crates/thetta-nesting/tests/engine_integration.rs`
- `crates/thetta-nesting/tests/benchmark_smoke.rs`
- `crates/thetta-nesting/tests/multi_sheet.rs`
STEP fallback and roundtrip tests:
- `crates/thetta-step/tests/profile_tests.rs`
- `crates/thetta-step/tests/thickness_tests.rs`
- `crates/thetta-step/tests/roundtrip_test.rs`
STEP + OCCT integration tests:
- `crates/thetta-step/tests/full_pipeline_test.rs`
- `crates/thetta-step/tests/occt_integration_test.rs`
Benchmark/utility examples and binaries:
- `crates/thetta-nesting/src/bin/benchmark.rs`
- `crates/thetta-app/examples/`
### 10.3 OCCT Test Concurrency Pattern
OCCT integration tests use global mutex guards (for example `OCCT_LOCK` in
`crates/thetta-step/tests/occt_integration_test.rs`) due to non-thread-safe
shared OCCT state.
Running those tests with one test thread remains the conservative mode.
### 10.4 Test Data Conventions
JSON-extension convention to force fallback importer path appears in tests such
as:
- `crates/thetta-step/tests/roundtrip_test.rs`
- `crates/thetta-step/tests/thickness_tests.rs`
- `crates/thetta-step/tests/profile_tests.rs`
Rationale:
- `import_step` checks file extension and routes `.step`/`.stp` to OCCT path
  when feature is active
- `.json` avoids accidental OCCT dependency in baseline test runs
### 10.5 Behavioral Patterns Validated by Tests
- basic placement and collision rejection in `NestingEngine`
- LBF placement completeness on small instances
- benchmark format ingestion and strip-mode stability
- multi-sheet behavior and regression checks
- profile normalization and hole handling
- thickness grouping and explicit thickness selection
- JSON roundtrip area drift bounds
- OCCT import/export/reimport smoke with real solids
## 11. Extension Points
### 11.1 Extension Table
| Extension | Where to add | Interface to implement |
|---|---|---|
| New optimization heuristic | `crates/thetta-nesting/src/` (new module + `runner.rs` integration) | Consume `Vec<Piece>`, `SheetConfig`, orientation policy; emit `NestingResult` or `NestingUpdate` stream |
| New file format import | `crates/thetta-step/src/import.rs` (new branch/adapter) | Produce `StepImportResult` with `StepModel`, `Vec<Piece>`, `Vec<StepPieceRef>` |
| New file format export | `crates/thetta-step/src/export.rs` (new writer branch) | Consume `StepModel`, `StepPieceRef`, `Sheet`, `SheetConfig`; return `Result<(), StepExportError>` |
| New UI panel | `crates/thetta-app/src/panels.rs` + `state.rs` | Add `PanelAction` variant and state transition in `main.rs` action dispatcher |
| New nesting mode | `crates/thetta-nesting/src/runner.rs` + `optimizer.rs` + app selector | Extend `NestingMode` and implement dispatch path with matching update semantics |
| Per-piece rotation constraints UI | `crates/thetta-app/src/state.rs` + panel controls | Maintain `piece_orientations: Vec<RotationRange>` aligned to pieces and pass to optimizer |
| New progress telemetry | `crates/thetta-nesting/src/runner.rs` + `thetta-app/src/state.rs` | Add `NestingUpdate` variant fields and handle in `poll_updates` |
| Alternative collision metric | `crates/thetta-nesting/src/hazard_counter.rs` | Implement collector/scoring adapter used by `NestingEngine::collision_cost` |
### 11.2 Practical Integration Rules
- keep UI thread non-blocking
- use channels for worker-to-UI updates
- preserve `PieceId` stability across conversions
- keep `StepPieceRef` mapping valid for export roundtrips
- gate heavy platform dependencies behind features when feasible
### 11.3 High-Impact Files for Most Extensions
- `crates/thetta-app/src/state.rs`
- `crates/thetta-nesting/src/runner.rs`
- `crates/thetta-nesting/src/engine.rs`
- `crates/thetta-step/src/import.rs`
- `crates/thetta-step/src/export.rs`
## 12. Glossary
### 12.1 Domain Terms
`SPP`
- Strip Packing Problem
- fixed strip height, optimize used strip length
`BPP`
- Bin Packing Problem
- fixed bin dimensions, minimize bin count
`LBF`
- Lowest Bottom-First greedy heuristic used as baseline placement phase
`GLS`
- Guided Local Search meta-heuristic used for iterative refinement
`NFP`
- No-Fit Polygon method used in some irregular packing systems
- not the primary strategy in this repository
`CDE`
- Collision Detection Engine from jagua-rs
`surrogate`
- simplified geometry representation used internally by jagua-rs collision path
`quadtree`
- hierarchical spatial partition used by jagua-rs collision structures
`piece`
- one nestable 2D polygon part represented by `thetta_geometry::types::Piece`
`placement`
- pose for one piece (`x`, `y`, `rotation`) represented by
  `thetta_geometry::types::Placement`
`sheet`
- one output layout surface represented by `thetta_geometry::types::Sheet`
`NestingResult`
- aggregate optimizer output containing `Vec<Sheet>` and `unplaced` piece IDs
`fill-and-spill`
- multi-sheet LBF policy: try all existing sheets before opening a new one
`compression`
- strip-length tightening phase via repeated feasible-placement checks under
  reduced width
`STEP`
- ISO 10303 CAD exchange format (`.step`, `.stp`)
`OCCT`
- OpenCASCADE Technology, used for STEP solid read/write and geometric queries
### 12.2 Project-Specific Operational Terms
`StepModel`
- imported source model abstraction in `thetta-step`
`StepPieceRef`
- mapping from nested `PieceId` back to source solid key for export
`RotationRange`
- jagua-rs rotation policy enum (`None`, `Continuous`, `Discrete`)
`NestingUpdate`
- optimizer-to-UI message enum used for progressive UI updates
`cancel token`
- `Arc<AtomicBool>` shared flag for cooperative termination
`strip_length`
- used width of a strip/sheet result after fitting or compression
`separation`
- user-configurable spacing distance propagated into conversion/collision model
## Appendix A. Source Index Used for This Document
This architecture description was verified against current source files including:
- `Cargo.toml`
- `.cargo/config.toml`
- `crates/thetta-geometry/src/lib.rs`
- `crates/thetta-geometry/src/types.rs`
- `crates/thetta-geometry/src/polygon_ops.rs`
- `crates/jagua-rs/Cargo.toml`
- `crates/jagua-rs/src/lib.rs`
- `crates/jagua-rs/src/entities/mod.rs`
- `crates/jagua-rs/src/io/mod.rs`
- `crates/jagua-rs/src/probs/mod.rs`
- `crates/jagua-rs/src/probs/spp/mod.rs`
- `crates/jagua-rs/src/probs/spp/entities/mod.rs`
- `crates/thetta-nesting/Cargo.toml`
- `crates/thetta-nesting/src/lib.rs`
- `crates/thetta-nesting/src/conversion.rs`
- `crates/thetta-nesting/src/engine.rs`
- `crates/thetta-nesting/src/lbf.rs`
- `crates/thetta-nesting/src/gls.rs`
- `crates/thetta-nesting/src/compression.rs`
- `crates/thetta-nesting/src/multi_sheet.rs`
- `crates/thetta-nesting/src/optimizer.rs`
- `crates/thetta-nesting/src/runner.rs`
- `crates/thetta-nesting/src/sampler.rs`
- `crates/thetta-nesting/src/hazard_counter.rs`
- `crates/thetta-nesting/src/benchmark.rs`
- `crates/thetta-nesting/src/bin/benchmark.rs`
- `crates/thetta-nesting/tests/integration.rs`
- `crates/thetta-nesting/tests/benchmark_smoke.rs`
- `crates/thetta-step/Cargo.toml`
- `crates/thetta-step/src/lib.rs`
- `crates/thetta-step/src/import.rs`
- `crates/thetta-step/src/export.rs`
- `crates/thetta-step/src/profile.rs`
- `crates/thetta-step/src/occt.rs`
- `crates/thetta-step/tests/profile_tests.rs`
- `crates/thetta-step/tests/thickness_tests.rs`
- `crates/thetta-step/tests/roundtrip_test.rs`
- `crates/thetta-step/tests/full_pipeline_test.rs`
- `crates/thetta-step/tests/occt_integration_test.rs`
- `crates/thetta-app/Cargo.toml`
- `crates/thetta-app/src/main.rs`
- `crates/thetta-app/src/state.rs`
- `crates/thetta-app/src/panels.rs`
- `crates/thetta-app/src/viewport.rs`
- `crates/thetta-app/src/native_dialog.rs`
- `docs/sprints/SPRINT-001.md`
- `docs/sprints/SPRINT-002.md`
- `docs/sprints/SPRINT-003.md`
- `docs/sprints/SPRINT-004.md`
- `research/repos/jagua-rs/README.md`
