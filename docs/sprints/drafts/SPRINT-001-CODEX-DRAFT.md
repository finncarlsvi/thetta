# Sprint 001: Thetta MVP — Codex Draft

## Overview

Deliver a working macOS desktop MVP of **Thetta** (Rust + egui) that:

1. Imports one or more STEP files via **OpenCASCADE (OCCT) FFI**
2. Detects candidate thicknesses, lets the user select a thickness, and extracts **2D outer profiles** (no part-in-part; holes ignored for nesting)
3. Nests the extracted profiles into fixed-size rectangular sheets using a **Guided Local Search (GLS)** optimizer backed by **jagua-rs** collision detection
4. Renders an interactive 2D viewport (zoom/pan/select) that shows progressive optimization updates
5. Exports one STEP per sheet by placing original solids at the nested transforms

Out of scope for this sprint:
- Part-in-part (placing pieces inside holes)
- Grain direction / discrete rotation constraints (rotation is continuous)
- DXF/SVG import/export
- Advanced nesting controls (advanced tuning, constraints, etc.)

## Use Cases

1. **Import STEP**
   - User loads STEP file(s) from a file picker (and optionally drag/drop).
   - App parses file(s), detects thickness buckets, auto-selects most common thickness.
   - App shows the available thickness values and counts; user can override.

2. **Preview Extracted Profiles**
   - App shows extracted 2D profiles per input file and per thickness.
   - User can delete individual pieces from the set (exclude from nesting).
   - User can reset a file (re-extract pieces from that file).

3. **Run Nesting**
   - After extraction, nesting starts automatically (or via a button).
   - Optimizer runs in background and streams progressive best-so-far layouts to the UI.
   - Multiple sheets are created when not all pieces fit.

4. **Inspect Result**
   - User can switch between sheets.
   - UI displays utilization, piece count, elapsed time, iteration/evaluation counts.
   - User can stop/resume optimization.

5. **Export**
   - User exports one STEP file per sheet.
   - Export places the original 3D solids using the same transforms used for the 2D nest.

## Architecture

### Crate Structure (current + MVP additions)

Existing workspace (from `Cargo.toml` and current stubs):
- `crates/thetta-app` (binary; egui/eframe UI)
- `crates/thetta-geometry` (shared 2D types)
- `crates/thetta-step` (STEP import/export; currently stubbed)
- `crates/thetta-nesting` (GLS optimizer; currently stubbed)

Planned internal structure for MVP (no new workspace crates required, but we introduce modules):
- `thetta-step`
  - `occt_sys` (unsafe FFI boundary + build configuration; minimal surface)
  - `reader` (load STEP, enumerate solids, compute thickness)
  - `extract` (largest-face selection, outer-wire extraction, discretization)
  - `model` (stable IDs and mapping: `PieceId` ↔ original OCCT shape handle)
  - `export` (create new STEP doc per sheet, apply transforms, write file)
- `thetta-nesting`
  - `jagua_bridge` (conversion: `thetta_geometry` → jagua-rs `Item`, `Strip`, `DTransformation`)
  - `gls` (sparrow-inspired separation/exploration/compression loop)
  - `multi_sheet` (sequential sheet filling + overflow strategy)
  - `stats` (iteration, evals/sec, utilization)
- `thetta-app`
  - `state` (app model: loaded docs, extracted pieces, current sheets)
  - `jobs` (thread orchestration + cancel tokens + channels)
  - `ui` (panels: files/thickness, sheet config, optimizer controls, export)
  - `viewport2d` (RectTransform, draw, hit-test, selection)

### Data Model (MVP-level)

Use existing `thetta_geometry::types` as the backbone and extend minimally where export requires a stable link back to OCCT:

- `PieceId` remains stable across extraction/nesting/export.
- Introduce `StepSource` metadata (in `thetta-step`, not in `thetta-geometry`) so we don’t leak OCCT/FFI concerns into shared geometry types.

Proposed core types (conceptual; exact placement in crates is a task):
- `thetta_step::model::StepDocument` (owns OCCT shapes for a loaded file; not `Send`/`Sync`)
- `thetta_step::model::StepPieceRef { piece_id: PieceId, solid_key: StepSolidKey }`
- `thetta_step::StepImportResult { pieces: Vec<Piece>, thicknesses: Vec<ThicknessInfo>, selected_thickness: f64, piece_refs: Vec<StepPieceRef>, document: StepDocumentHandle }`

### jagua-rs Integration (concrete API touchpoints)

The MVP uses jagua-rs **Strip Packing Problem (SPP)** APIs (same approach as `research/repos/sparrow`):

- Build an `SPInstance` from our pieces:
  - Create jagua `SPolygon` from our outer polyline: `SPolygon::new(Vec<Point>)`
  - Wrap as `OriginalShape { shape: SPolygon, pre_transform, modify_mode, modify_config }`
  - Create an `Item` via `Item::new(id, original_shape, RotationRange::Continuous, None, surrogate_config)`
  - Use `(Item, qty)` pairs to form `SPInstance::new(items, base_strip)`
- Manage placements in `SPProblem`:
  - `SPProblem::new(instance)`
  - `SPProblem::place_item(SPPlacement { item_id, d_transf })`
  - `SPProblem::remove_item(pk)` for moves
  - `SPProblem::save()` and `SPProblem::restore(&SPSolution)` for rollback
- Collision detection:
  - `layout.cde().collect_poly_collisions(&pi.shape, collector)` (see sparrow’s `CollisionTracker`)

Important modeling decisions for MVP:
- **No holes for nesting**: extract only the outer wire, or “fill” holes, matching “no part-in-part”.
- **Separation gap**: implement via jagua’s shape modification:
  - Inflate items by `separation / 2` using `ShapeModifyMode::Inflate` and `ShapeModifyConfig { offset: Some(separation/2), ... }`
  - Deflate the container by `separation / 2` (or border clearance) using `Strip.shape_modify_config.offset`

### GLS Optimizer Design (reference: `research/repos/sparrow/`)

Use sparrow as the design reference:
- Separation loop + GLS weight update: `research/repos/sparrow/src/optimizer/separator.rs` and `research/repos/sparrow/src/quantify/tracker.rs`
- Exploration phase (shrink-and-separate with disruption pool): `research/repos/sparrow/src/optimizer/explore.rs`
- Compression phase (iterative shrink with decay strategies): `research/repos/sparrow/src/optimizer/compress.rs`
- Candidate sampling + coordinate descent refinement: `research/repos/sparrow/src/sample/search.rs` and `research/repos/sparrow/src/sample/coord_descent.rs`

Adaptation for rectangular sheets:
- Represent each sheet as a strip with `fixed_height = sheet.height` and `width = sheet.width`, and run **feasibility-first GLS**.
- Optional compression in MVP:
  - If we keep jagua “strip width” variable for compression, treat the “width” dimension as the axis we’re allowed to minimize (used length). Map back to sheet coordinates when presenting/exporting.
  - If complexity is too high, ship MVP with exploration/separation only and treat compression as “stretch goal” within this sprint (still described and scaffolded).

### Threading Model (UI + OCCT + Optimizer)

Constraints:
- egui/eframe expects the UI to run on the main thread.
- OCCT FFI objects should be treated as **thread-affine** (assume not `Send`/`Sync`).
- Nesting can be CPU-heavy and must not block UI.

Model:
- **UI thread**: renders, handles input, submits jobs, receives progress updates.
- **CAD thread** (single dedicated thread): performs *all* OCCT calls (read/extract/export). Receives commands via channel, returns results via channel.
- **Optimizer thread** (one per active optimization): owns jagua-rs `SPProblem` and runs GLS; streams improvements to UI via channel; supports cancellation.
- Optional: internal parallelism in optimizer via a dedicated Rayon pool, mirroring sparrow’s local pool pattern in `Separator`.

Message passing (sketch):
- `thetta-app` owns:
  - `cad_tx/cad_rx`: command/results with CAD thread
  - `opt_tx/opt_rx`: updates from optimizer thread
  - `Arc<AtomicBool>` (or `CancellationToken`) for cancel/stop

## Implementation (Phased with Tasks)

### Phase 0 — Sprint Setup / Build Hygiene (1–2 days)

- Add baseline CI-ish commands (documented in sprint only):
  - `cargo fmt`, `cargo clippy`, `cargo test`, `cargo run -p thetta-app`
- Add structured error handling across crates:
  - Replace `todo!()` with typed errors where APIs are already present (`StepImportError`, `StepExportError`).
- Decide and document OCCT linking strategy:
  - Preferred: `opencascade-sys` + system OCCT install (Homebrew on macOS).
  - Fallback: local vendored OCCT or user-provided `OCCT_DIR` env var.

### Phase 1 — STEP Import: Load + Thickness Buckets + Outer Profiles (4–6 days)

Goal: “Load STEP → list thickness choices → extract 2D outlines → show in viewport (un-nested).”

Tasks (in `crates/thetta-step`):
- Implement an OCCT boundary module:
  - Minimal unsafe layer that exposes:
    - Load STEP file into a document/shape collection
    - Enumerate solids
    - Compute per-solid bounding box
    - Extract candidate face wire(s) and discretize to point lists
- Thickness detection:
  - For each solid: compute thickness as min bbox dimension (in model units).
  - Bucket thicknesses with tolerance (e.g., 0.1mm default, configurable).
  - Return sorted buckets with counts and a recommended thickness (mode).
- Profile extraction (MVP rules):
  - Select the largest face by area for each matching solid.
  - Extract **outer wire only** (ignore inner wires/holes).
  - Discretize edges with a stable sampling rule:
    - Fixed chord tolerance OR fixed “max segment length” in mm.
    - Ensure closed polygon and remove near-duplicate consecutive points.
  - Convert to `thetta_geometry::types::Polygon` (outer only for now; holes empty).
- Stable mapping for export:
  - Create a `StepDocument` model that stores OCCT handles and exposes `StepSolidKey`.
  - Attach `PieceId → StepSolidKey` mapping in import result.

Deliverables:
- `thetta_step::import_step(...) -> StepImportResult` implemented (no `todo!()`).
- A small “golden” sample STEP (or use an existing local file) can be imported and yields pieces + thickness buckets.

### Phase 2 — jagua-rs Bridge + Feasibility Nesting (3–5 days)

Goal: “Convert pieces → jagua instance → produce a collision-free layout in one sheet (or decide overflow).”

Tasks (in `crates/thetta-nesting`):
- Add jagua-rs dependency (workspace-level decision: crates.io version or path dependency).
- Implement conversion layer:
  - `Piece` polygon (f64, mm) → jagua `SPolygon` (f32).
  - Set `RotationRange::Continuous`.
  - Apply separation via `ShapeModifyConfig.offset`.
  - Build `Strip::new(fixed_height, cde_config, shape_modify_config, width)`.
  - Build `SPInstance` and `SPProblem`.
- Implement a basic constructive initializer:
  - Start with a deterministic “scatter” placement or a simple LBF-style placement using jagua feasibility checks.
  - Ensure we can always produce an initial (possibly infeasible) layout for GLS.
- Implement overlap/loss quantification and GLS weights:
  - Start by copying the *structure* of sparrow’s `CollisionTracker` concept:
    - Track per-item collision loss and container collisions.
    - Update weights using a decay/increase rule similar to `CollisionTracker::update_weights()`.
  - For MVP, simplify if needed:
    - Use jagua’s `collect_poly_collisions` to find colliding hazards per moved item.
    - Quantify using a proxy metric (e.g., bbox overlap proxy first), then improve toward sparrow’s overlap proxy.

Deliverables:
- `thetta_nesting::run_optimizer(...)` produces at least one feasible sheet layout for easy polygon sets.
- A `NestingUpdate::Improved(...)` stream works end-to-end through a channel.

### Phase 3 — GLS Optimizer (sparrow-style) + Multi-Sheet (5–8 days)

Goal: “GLS separates collisions, shrinks/compresses when feasible, overflows to multiple sheets sequentially.”

Tasks (in `crates/thetta-nesting`):
- Implement sparrow-inspired separation loop:
  - Multi-worker “try move items in different orders” approach (see sparrow `SeparatorWorker` pattern).
  - Optional Rayon local pool to keep worker threads stable during one run.
- Candidate generation strategy:
  - Uniform samples across the container.
  - Focussed samples around the current placement (Gaussian jitter / local window).
  - Coordinate descent refinement of best candidates (mirror sparrow’s two-stage refinement idea).
- Exploration/solution-pool disruption:
  - Keep a pool of best infeasible solutions, pick one with bias, disrupt by swapping large items (sparrow’s approach).
- Compression (if retained for MVP):
  - Attempt iterative shrink with failure decay (sparrow `compression_phase` concept).
  - Stop when shrink step below threshold or time budget exhausted.
- Multi-sheet sequential filling:
  - Attempt to pack all remaining pieces on current sheet.
  - If not feasible by timeout:
    - Remove the “worst offenders” (highest weighted loss) and retry until feasible.
  - Commit placed subset as sheet 1, repeat for remaining pieces for sheet 2, etc.
- Determinism controls:
  - Seeded RNG in config; record seed in UI for reproducibility.

Deliverables:
- End-to-end nesting of extracted STEP pieces into 1+ sheets with progressive improvements.
- Cancel/stop/resume works reliably (stop: set cancel token; resume: restart optimizer with current best as warm start if supported).

### Phase 4 — egui App + 2D Viewport + Export (5–8 days)

Goal: “User-facing MVP: load STEP, choose thickness, nest with live updates, inspect, delete items, export sheets.”

Tasks (in `crates/thetta-app`):
- UI layout:
  - Left panel: files list, thickness dropdown, piece count, delete/reset.
  - Right/top panel: sheet config (width/height/separation), optimizer controls (start/stop/resume), stats.
  - Center: viewport with sheet outline + pieces.
- Viewport2D:
  - World coordinates in mm; screen mapping with zoom/pan.
  - Draw:
    - Sheet boundary rectangle
    - Piece polygons (filled + outline)
    - Selection highlight and hover
  - Interaction:
    - Scroll zoom (cursor-centered)
    - Drag pan
    - Click select (hit-test via point-in-polygon or bbox first)
    - Delete selected piece
- Job orchestration:
  - Spawn CAD thread at app start.
  - On file load: send CAD command; show progress; update state on completion.
  - On “nest”: spawn optimizer thread; connect update channel.
  - On “export”: send CAD command with placements + selected sheet(s).
- Export pipeline (in `crates/thetta-step`):
  - For each sheet:
    - Create output STEP document
    - For each placement:
      - Find referenced OCCT solid in the source document(s)
      - Apply 2D transform in the sheet plane (translation + rotation)
      - Add to output doc
    - Write file

Deliverables:
- `cargo run -p thetta-app` gives a usable app that performs the full MVP pipeline.

## Files Summary (What Changes Where)

Existing files to implement/expand:
- `crates/thetta-step/src/import.rs` (implement `import_step`)
- `crates/thetta-step/src/export.rs` (implement `export_sheet` / per-sheet export)
- `crates/thetta-nesting/src/optimizer.rs` (implement GLS + jagua integration)
- `crates/thetta-app/src/main.rs` (wire UI + jobs + viewport)
- `crates/thetta-geometry/src/types.rs` (extend types if needed for selection/stats; keep minimal)

Likely new files (MVP):
- `crates/thetta-step/src/model.rs` (OCCT handle ownership + `PieceId` mapping)
- `crates/thetta-step/src/occt.rs` or `crates/thetta-step/src/ffi/*` (unsafe boundary)
- `crates/thetta-nesting/src/jagua_bridge.rs`
- `crates/thetta-nesting/src/gls.rs` (or `src/separator.rs`, `src/tracker.rs`, `src/sampler.rs`)
- `crates/thetta-app/src/state.rs`
- `crates/thetta-app/src/viewport2d.rs`
- `crates/thetta-app/src/jobs.rs`

## Definition of Done

Functional:
- Import STEP file(s) and extract 2D profiles by selected thickness.
- Show thickness buckets and let user switch thickness; re-extract accordingly.
- Run nesting in background with live updates; produces 1+ sheets.
- Viewport supports zoom/pan/select; user can delete pieces; re-nests after deletion.
- Export writes one STEP per sheet using original solids and computed placements.

Quality:
- No `todo!()` in the main user path (import → nest → export).
- App remains responsive while importing/nesting.
- Crash-free handling of invalid STEP files (clear error messages).
- Basic tests exist for:
  - Polygon sanitation (closed polygon, duplicates removed)
  - Deterministic optimizer configuration (seeded run yields stable result for a fixed polygon set)

Performance (MVP target):
- Nesting produces a first feasible result within ~10–30s for small/medium jobs (tunable).
- UI redraw stays smooth during optimization updates (updates throttled if needed).

## Risks

- **OCCT FFI complexity (high):** build/linking on macOS ARM64 and safe lifetime management.
- **Thread affinity / safety (high):** OCCT objects assumed non-`Send`; must isolate to CAD thread.
- **Geometry robustness (medium/high):** discretization of curved edges and numeric tolerances can create self-intersecting or near-degenerate polygons.
- **jagua polygon requirements (medium):** `SPolygon::new` rejects duplicate points and zero-area; extraction must sanitize.
- **GLS tuning (medium):** requires parameter tuning (samples/move, refinement depth, shrink schedule) for acceptable quality/time.
- **Coordinate conventions (medium):** mapping 3D face to 2D plane and consistent units/orientation.

## Security

- Treat STEP inputs as untrusted:
  - Validate paths and handle file-not-found cleanly.
  - Fail closed on parse/FFI errors; never panic in FFI callbacks.
- Minimize unsafe surface area:
  - Concentrate `unsafe` in `thetta-step` OCCT boundary module.
  - Prefer opaque handles and explicit ownership rules.
- Avoid data races:
  - OCCT access confined to one CAD thread.
  - Optimizer thread owns jagua state; UI receives only snapshots/copies.
- Dependency hygiene:
  - Pin jagua-rs to a known-good version or commit; document in `Cargo.toml`.

## Dependencies

Rust:
- `eframe`/`egui` (already present)
- `jagua-rs` (add to `thetta-nesting`)
- `rayon` (optional, for optimizer worker pool)
- `rfd` (file dialog for macOS) or egui file picker alternative
- `crossbeam-channel` or `std::sync::mpsc` for job/update channels

System:
- OpenCASCADE (OCCT) libraries and headers available to the build:
  - Homebrew install on macOS (preferred), or user-provided install + env vars
- Tooling to compile C++ dependencies as required by `opencascade-sys` (if used)

Reference code (already present in repo):
- GLS design reference: `research/repos/sparrow/`
- Collision detection engine: `research/repos/jagua-rs/`

## Open Questions

1. **OCCT binding choice:** use `opencascade-sys` as-is, or generate/curate a smaller bindgen surface tailored to needed functions?
2. **Discretization policy:** chord tolerance vs max segment length; how to pick defaults to balance accuracy/performance?
3. **Face selection heuristic:** “largest face by area” is simple—do we need a more robust “most planar + largest” selection for sheet-metal parts?
4. **Units:** do we assume STEP units are mm? If not, do we infer and normalize?
5. **Compression stage:** do we ship full exploration+compression in MVP, or commit feasibility-only GLS and leave compression as Sprint 2 improvement?
6. **Export transform correctness:** how to define the part’s 2D reference frame so rotation/translation in 2D maps correctly back into 3D placement for STEP export?

