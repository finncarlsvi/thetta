# Sprint 003: STEP File Import & Export Pipeline

## Overview

Implement the full STEP file pipeline: load STEP files via OpenCASCADE FFI (`opencascade-sys`), detect thickness groups, extract 2D profiles from sheet-metal solids, feed them into the existing nesting engine, and export one STEP file per sheet with solids repositioned to their nested locations. This sprint also adds native file dialogs (`rfd`) and extends the GUI with a thickness selector panel.

**Scope:** OpenCASCADE system setup, STEP import (solid iteration, thickness detection, face extraction, wire discretization), STEP export (solid repositioning, STEP writer), GUI integration (file dialogs, thickness selector), round-trip testing.
**Prerequisites:** Sprint 002 complete. The nesting engine wraps jagua-rs `SPProblem` and accepts `Vec<Piece>` with optional `Vec<RotationRange>` orientations. It produces `NestingResult` with `Vec<Sheet>` containing `strip_length`. The app already supports benchmark file loading and background optimizer with cancel token.
**Deferred:** Advanced face selection heuristics, STEP assembly structure preservation, configurable mesh tolerance UI.

## Use Cases

### UC1: Load STEP File via Native File Dialog
- User clicks "Load STEP" button in the side panel
- Native macOS file dialog opens, filtered to `*.step` and `*.stp` extensions
- App reads the file on a background thread with a loading spinner
- On success, thickness groups are displayed in the side panel
- On error, a label shows the error message (e.g., "No solids found", "OpenCASCADE parse error")

### UC2: Select Thickness and View Profiles
- After loading, the thickness selector shows all detected thicknesses sorted by solid count (descending)
- Each entry shows: thickness value (mm), number of solids, and a radio button
- The most common thickness is auto-selected
- Selecting a thickness re-extracts 2D profiles without re-reading the STEP file
- Profiles appear immediately in the viewport and trigger auto-nesting

### UC3: Nest STEP-Derived Profiles
- After thickness selection, the extracted `Vec<Piece>` feeds into the existing nesting pipeline
- Background optimizer runs via existing `run_optimizer()` -> `run_nesting()` (LBF baseline then GLS with cancel token, then strip compression)
- `NestingEngine::new_with_orientations()` creates the jagua-rs `SPProblem`
- Viewport shows nested result with real-time improvements via `NestingUpdate::Improved` / `NestingUpdate::Finished`
- All existing nesting controls work (start/stop, separation, sheet size)

### UC4: Export Nested Sheets as STEP Files
- User clicks "Export STEP" button
- Native macOS save dialog opens, user picks output directory and base filename
- For each sheet in the nesting result, the app writes `{basename}_sheet_{N}.step`
- Each output STEP file contains the original 3D solids repositioned to their nested 2D locations
- On completion, a status label shows "Exported N sheets"

### UC5: Mixed Input Sources
- User can load JSON polygon files and STEP files in one session
- Pieces keep their `source_file: PathBuf` attribution (used for coloring via `AppState::file_colors`)
- STEP export only includes pieces that originate from STEP; JSON-only pieces are excluded with a clear warning

### UC6: Round-Trip Verification (Developer)
- Developer runs an integration test: import STEP -> extract profiles -> nest -> export STEP -> reimport -> profile counts and areas match within tolerance

## Architecture

### Crate Structure (Current Post-Sprint-002)

```
thetta/
+-- Cargo.toml                    (workspace: resolver = "2")
+-- crates/
|   +-- thetta-geometry/          (2D types: Piece, Polygon, Point2D, Placement, Sheet, NestingResult)
|   +-- thetta-nesting/           (NestingEngine wrapping jagua-rs SPProblem, LBF, GLS, benchmarks)
|   +-- thetta-app/               (egui GUI: panels, viewport, state with optimizer threading)
|   +-- thetta-step/              (STEP import/export -- currently stubs with todo!())
|   +-- jagua-rs/                 (forked collision detection engine, workspace member)
+-- research/repos/sparrow/       (benchmark data -- 34 ESICUP instances)
+-- test_data/
|   +-- step/                     (test STEP files)  <-- NEW
```

### Dependency Graph (Current + Sprint 003 Additions)

```
thetta-app
+-- thetta-nesting
|   +-- thetta-geometry
|   +-- jagua-rs (features = ["spp"])
+-- thetta-step
|   +-- thetta-geometry
|   +-- opencascade-sys            <-- NEW (feature-gated)
+-- thetta-geometry
+-- jagua-rs (features = ["spp"])  (direct dep for RotationRange type)
+-- eframe 0.31 / egui 0.31
+-- serde_json 1
+-- rfd                            <-- NEW
```

Note: `thetta-app` already depends on `thetta-step` (see current Cargo.toml). The `thetta-step` crate currently only depends on `thetta-geometry`. No `rfd` dependency exists yet.

### Data Flow

```
STEP File (.step/.stp)
        |
        v
  +-----------------+
  | STEPControl_    |    opencascade-sys FFI
  | Reader::ReadFile|
  +--------+--------+
           |
           v
  +-----------------+
  | TopExp_Explorer  |   Iterate TopAbs_SOLID
  | (SOLID)          |
  +--------+---------+
           |
           v
  +-----------------+
  | Bnd_Box +       |   Per-solid bounding box
  | BRepBndLib::Add |   -> thickness = min dimension
  +--------+--------+
           |
           v
  +-------------------+
  | Group by thickness |   HashMap<i64, Vec<SolidHandle>>
  | (0.1mm buckets)    |   key = round(thickness * 10)
  +--------+----------+
           |  user selects thickness
           v
  +------------------------+
  | For each solid:         |
  |  TopExp_Explorer(FACE)  |   Find largest planar face
  |  BRepGProp::Surface     |   (BRepGProp::SurfaceProperties)
  |  BRepTools::OuterWire   |   Extract outer wire + holes
  |  GCPnts discretization  |   Edges -> polyline points
  |  Project 3D -> 2D       |   Using face plane's gp_Ax3
  +----------+-------------+
             |
             v
       Vec<Piece>
             |
             v
  +------------------+     +------------------+
  |  NestingEngine   |---->|    jagua-rs      |
  |  (SPProblem,     |<----|    CDEngine      |
  |   LBF + GLS)     |     +------------------+
  +--------+---------+
           |
           v
     NestingResult { sheets: Vec<Sheet { placements, utilization, strip_length }> }
             |
             v
  +------------------------+
  | For each sheet:         |
  |  For each placement:    |
  |    Retrieve original    |
  |    TopoDS_Solid         |
  |    Apply gp_Trsf        |    2D transform on XY plane
  |    (translate + rotate) |
  +----------+-------------+
             |
             v
  +------------------------+
  | STEPControl_Writer     |
  | ::Transfer + ::Write   |    One .step per sheet
  +------------------------+
```

### New Types in thetta-step

```rust
/// Safe wrapper around a loaded STEP model with OCCT state
pub struct StepModel {
    solids: Vec<SolidRecord>,  // OCCT handles + metadata
    // All OCCT state is owned here and freed on Drop
}

/// Per-solid metadata and OCCT handle
struct SolidRecord {
    key: StepSolidKey,
    thickness_mm: f64,
    // handle to TopoDS_Solid (OCCT)
}

/// Opaque key into StepModel's solid table
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StepSolidKey(pub u32);

/// Maps a nesting Piece back to its source OCCT solid
#[derive(Debug, Clone)]
pub struct StepPieceRef {
    pub piece_id: PieceId,
    pub solid_key: StepSolidKey,
    pub source_path: PathBuf,
    pub thickness_mm: f64,
}

/// Detected thickness group (replaces current stub ThicknessInfo)
#[derive(Debug, Clone)]
pub struct ThicknessInfo {
    pub value_mm: f64,
    pub count: usize,
}

/// Result of a STEP import operation (replaces current stub)
pub struct StepImportResult {
    pub model: StepModel,
    pub pieces: Vec<Piece>,
    pub piece_refs: Vec<StepPieceRef>,
    pub thicknesses: Vec<ThicknessInfo>,
    pub selected_thickness_mm: f64,
}

/// Import error types (replaces current stub)
pub enum StepImportError {
    FileNotFound,
    ReadFailed(String),
    NoSolidsFound,
    NoProfilesExtracted { skipped: usize, reason: String },
    OcctError(String),
}

/// Export error types (replaces current stub)
pub enum StepExportError {
    InvalidOutputPath(String),
    MissingSolidForPiece(u64),
    WriteError(String),
    OcctError(String),
}

/// Configurable extraction parameters
pub struct ExtractionConfig {
    pub mesh_tolerance_mm: f64,       // Default: 0.5mm
    pub thickness_bucket_mm: f64,     // Default: 0.1mm
    pub max_points_per_edge: usize,   // Default: 200
}
```

### Design Decisions

1. **Keep OCCT out of `thetta-geometry`:** `Piece` and `Polygon` remain pure Rust types. The mapping from `PieceId` -> OCCT solid lives in `StepPieceRef`, owned by `thetta-step`.

2. **Feature-gated OCCT:** `opencascade-sys` is behind `--features occt` so the workspace builds without OCCT installed. Only `thetta-step` (when feature enabled) and `thetta-app` (when loading STEP) need this feature.

3. **Thread safety strategy:** Extract all Rust data (`Vec<Piece>`, `Vec<ThicknessInfo>`) during import. Keep OCCT solids in `StepModel` behind `Arc<Mutex<>>` only for export. This avoids Send/Sync issues for the UI thread.

4. **Deterministic PieceIds:** Use `PieceId(solid_index as u64)` so IDs are stable across thickness switches.

5. **Integration with existing nesting pipeline:** STEP-derived pieces use continuous rotation (`RotationRange::Continuous`) since sheet-metal parts have no discrete orientation constraints. The existing `AppState::start_optimizer()` already handles `piece_orientations` and passes them through `run_optimizer()` -> `run_nesting()` -> `NestingEngine::new_with_orientations()`.

### Threading Model

Follows the existing pattern established in Sprint 002:

```
Main Thread (egui)                    STEP Import Thread
+-------------------+               +---------------------+
| - Triggers import | -- path -->   | - Calls import_step |
| - Shows spinner   |               |   (all OCCT work)   |
| - Polls via       | <-- mpsc --   | - Sends result      |
|   try_recv()      |               +---------------------+
+-------------------+

Main Thread (egui)                    STEP Export Thread
+-------------------+               +---------------------+
| - Triggers export | -- data -->   | - Calls export_all  |
| - Shows progress  |               |   _sheets           |
| - Polls done      | <-- mpsc --   | - Sends status      |
+-------------------+               +---------------------+
```

This mirrors `AppState::start_optimizer()` which spawns a thread with `mpsc::channel` and `Arc<AtomicBool>` cancel token.

## Implementation

### Phase 0: OpenCASCADE Bring-Up

**Goal:** `thetta-step` compiles and links on macOS ARM64.

#### Task 0.1: Install OpenCASCADE
```sh
brew install opencascade cmake pkg-config
pkg-config --libs opencascade
# Expected: -L/opt/homebrew/lib -lTKernel -lTKMath ...

# If pkg-config doesn't find OCCT:
export PKG_CONFIG_PATH="/opt/homebrew/opt/opencascade/lib/pkgconfig:$PKG_CONFIG_PATH"
```

#### Task 0.2: Evaluate opencascade-sys API coverage

**CRITICAL INVESTIGATION:** Before committing to `opencascade-sys`, verify it exposes:
- `STEPControl_Reader` (read STEP files)
- `STEPControl_Writer` (write STEP files)
- `TopExp_Explorer` (iterate topology)
- `BRepBndLib::Add` / `Bnd_Box` (bounding boxes)
- `BRepGProp::SurfaceProperties` (face areas)
- `BRepTools::OuterWire` (wire extraction)
- `GCPnts_QuasiUniformDeflection` (curve discretization)
- `BRepBuilderAPI_Transform` (solid repositioning)
- `gp_Trsf` (3D transforms)

**If `opencascade-sys` v0.2 is insufficient** (it's labeled "major work in progress"), alternatives:
1. The higher-level `opencascade` crate (wraps opencascade-sys with safer API)
2. Custom `bindgen` bindings for the specific OCCT classes needed
3. Thin C++ shim compiled via `cc` crate that exposes a C API

#### Task 0.3: Add dependency and smoke test
**File:** `crates/thetta-step/Cargo.toml` (MODIFY)
```toml
[features]
default = []
occt = ["opencascade-sys"]

[dependencies]
thetta-geometry = { path = "../thetta-geometry" }
opencascade-sys = { version = "0.2", optional = true }
```

```rust
#[cfg(feature = "occt")]
#[test]
fn occt_smoke_test() {
    // Create a STEPControl_Reader and verify it initializes
}
```

**Deliverable:** `cargo build -p thetta-step --features occt` succeeds on macOS ARM64.

### Phase 1: Safe OCCT FFI Wrapper

**Goal:** Centralize all `unsafe` FFI in a dedicated module with RAII wrappers.

#### Task 1.1: OCCT wrapper module
**File:** `crates/thetta-step/src/occt.rs` (NEW)

Safe Rust wrappers around OCCT types:

```rust
pub struct StepReader { /* raw OCCT handle */ }

impl StepReader {
    pub fn read_file(path: &Path) -> Result<Self, StepImportError>;
    pub fn transfer_roots(&mut self) -> usize;
    pub fn root_shape(&self) -> OcctShape;
}

impl Drop for StepReader { /* release OCCT resources */ }

pub struct StepWriter { /* raw OCCT handle */ }

impl StepWriter {
    pub fn new() -> Self;
    pub fn add_shape(&mut self, shape: &OcctShape);
    pub fn write(&self, path: &Path) -> Result<(), StepExportError>;
}

pub struct SolidExplorer { /* wraps TopExp_Explorer */ }

pub fn bounding_box(solid: &OcctSolid) -> [f64; 6];
pub fn face_area(face: &OcctFace) -> f64;
pub fn face_plane(face: &OcctFace) -> Option<PlaneInfo>;
pub fn discretize_wire(
    wire: &OcctWire,
    plane: &PlaneInfo,
    tolerance: f64,
    max_points_per_edge: usize,
) -> Vec<Point2D>;
pub fn transform_shape(
    shape: &OcctShape,
    x: f64, y: f64, rotation: f64,
) -> OcctShape;
```

All `unsafe` FFI calls confined to this module. Every OCCT handle has a `Drop` impl.

### Phase 2: STEP Import Pipeline

#### Task 2.1: Solid enumeration + thickness detection
**File:** `crates/thetta-step/src/import.rs` (REWRITE -- currently has `todo!()`)

Replace the current stub with:
```rust
pub fn import_step(
    path: &Path,
    thickness: Option<f64>,
    config: &ExtractionConfig,
) -> Result<StepImportResult, StepImportError>
```

OCCT pipeline:
1. `StepReader::read_file(path)` -- load STEP file
2. `reader.transfer_roots()` -- parse into shapes
3. `SolidExplorer::new(root_shape)` -- enumerate all solids
4. For each solid: `bounding_box(solid)` -> `thickness = min(dx, dy, dz)`
5. Group by thickness bucket
6. Default: bucket with most solids

#### Task 2.2: Profile face selection
**File:** `crates/thetta-step/src/profile.rs` (NEW)

Face selection heuristic:
1. Enumerate faces via `TopExp_Explorer(solid, TopAbs_FACE)`
2. Check planarity: `face_plane(face)` returns `Some(PlaneInfo)` if planar
3. Pick largest planar face by area
4. If no planar face: skip solid, collect warning

#### Task 2.3: Wire extraction + curve discretization

For the selected face:
1. Outer wire via `BRepTools::OuterWire(face)`
2. Holes: any wire != outer
3. For each wire, iterate edges, discretize, project to 2D using face `PlaneInfo`
4. Post-process: dedup adjacent points, ensure CCW outer / CW holes, close ring

Output: `Piece { id: PieceId(solid_index), polygon, source_file: path, quantity: 1 }`

### Phase 3: STEP Export Pipeline

#### Task 3.1: Sheet export
**File:** `crates/thetta-step/src/export.rs` (REWRITE -- currently has `todo!()`)

```rust
pub fn export_sheet(
    model: &StepModel,
    piece_refs: &[StepPieceRef],
    sheet: &Sheet,
    output_path: &Path,
) -> Result<(), StepExportError>
```

For each `Placement` in the sheet:
1. Look up `StepSolidKey` via `piece_refs` for the `piece_id`
2. Retrieve original `TopoDS_Solid` from `model`
3. Build `gp_Trsf`: rotate around Z, then translate by `(placement.x, placement.y, 0)`
4. Apply via `BRepBuilderAPI_Transform`
5. Add to `StepWriter`, write file

#### Task 3.2: Multi-sheet export

```rust
pub fn export_all_sheets(
    model: &StepModel,
    piece_refs: &[StepPieceRef],
    result: &NestingResult,
    out_dir: &Path,
    file_stem: &str,
) -> Result<Vec<PathBuf>, StepExportError>
```

Iterates sheets: `{file_stem}_sheet_001.step`, etc. Skips non-STEP pieces with warning.

### Phase 4: GUI Integration

#### Task 4.1: Add rfd dependency
**File:** `crates/thetta-app/Cargo.toml` (MODIFY)
```toml
rfd = "0.14"
thetta-step = { path = "../thetta-step", features = ["occt"] }  # update existing line
```

Note: `thetta-step` is already a dependency but without the `occt` feature.

#### Task 4.2: STEP section in side panel
**File:** `crates/thetta-app/src/panels.rs` (MODIFY)

The current panels.rs (116 lines) has collapsing sections: Files, Sheet, Pieces, Nesting, Benchmark, Stats. Add a new "STEP" collapsing section:
- "Load STEP" button -> `rfd::FileDialog` filtered to `*.step`, `*.stp`
- Loading spinner while import runs
- Error display label
- Thickness selector (radio buttons: `{thickness_mm} mm ({count} solids)`)
- "Export STEP" button -> `rfd::FileDialog` save dialog

Add `PanelAction::LoadStep(PathBuf)`, `PanelAction::SelectThickness(usize)`, `PanelAction::ExportStep`.

#### Task 4.3: STEP session state
**File:** `crates/thetta-app/src/state.rs` (MODIFY)

Current `AppState` fields (from Sprint 002):
```rust
pub struct AppState {
    pub loaded_files: Vec<LoadedFile>,
    pub pieces: Vec<Piece>,
    pub piece_orientations: Vec<RotationRange>,
    pub benchmark: Option<LoadedBenchmark>,
    pub sheet_config: SheetConfig,
    pub optimizer_config: OptimizerConfig,
    pub current_result: Option<NestingResult>,
    pub selected_piece: Option<PieceId>,
    pub optimizer_running: bool,
    pub stats: NestingStats,
    pub update_rx: Option<Receiver<NestingUpdate>>,
    pub cancel_token: Option<Arc<AtomicBool>>,
    pub run_started_at: Option<Instant>,
    pub file_colors: HashMap<PathBuf, Color32>,
}
```

Add new fields:
```rust
pub step_session: Option<Arc<Mutex<StepModel>>>,
pub step_piece_refs: Vec<StepPieceRef>,
pub thickness_groups: Vec<ThicknessInfo>,
pub selected_thickness_idx: usize,
pub step_loading: bool,
pub step_error: Option<String>,
pub step_import_rx: Option<Receiver<Result<StepImportResult, StepImportError>>>,
```

New methods:
- `start_step_import(path)`: spawn thread, store receiver
- `poll_step_import()`: called each frame via `try_recv()` (same pattern as `poll_updates()`)
- `select_thickness(idx)`: re-extract profiles from in-memory model, restart optimizer
- `start_step_export(path)`: spawn thread for export

#### Task 4.4: Wire up in main app
**File:** `crates/thetta-app/src/main.rs` (MODIFY)

Current update loop calls `self.state.poll_updates()` and handles `PanelAction` variants. Add:
- `self.state.poll_step_import()` alongside existing `poll_updates()`
- Handle new `PanelAction::LoadStep`, `SelectThickness`, `ExportStep` variants

### Phase 5: Testing

#### Task 5.1: Generate test STEP files
**Option A (preferred):** Generate at test-time using OCCT primitives:
```rust
#[cfg(feature = "occt")]
fn create_test_box(w: f64, h: f64, t: f64, path: &Path) {
    // Use BRepPrimAPI_MakeBox
}
```

**Option B:** Check in 1-2 tiny `.step` fixtures under `test_data/step/`.

#### Task 5.2: Thickness detection tests
**File:** `crates/thetta-step/tests/thickness_tests.rs` (NEW)

- Single box -> 1 thickness group, correct value
- Two boxes different thickness -> 2 groups, most common auto-selected
- Specific thickness parameter -> selects correct group

#### Task 5.3: Profile extraction tests
**File:** `crates/thetta-step/tests/profile_tests.rs` (NEW)

- Box -> rectangular profile with 4+ vertices, correct approximate area
- Box with cylindrical hole -> 1 hole in polygon
- L-shaped extrusion -> profile with 6+ outer vertices

#### Task 5.4: Round-trip integration test
**File:** `crates/thetta-step/tests/roundtrip_test.rs` (NEW)

- Import -> create manual placements -> export -> reimport -> verify piece count and area within 5%

#### Task 5.5: Build verification
```sh
cargo build --release -p thetta-step --features occt
cargo test -p thetta-step --features occt
cargo build --release -p thetta-app
cargo test --workspace
```

Note: existing tests (engine_integration, benchmark_smoke, integration) must continue passing. The workspace builds without `occt` feature by default.

## Files Summary

### New Files

| File | Purpose | Est. LOC |
|------|---------|----------|
| `crates/thetta-step/src/occt.rs` | Safe OCCT FFI wrappers (RAII, Drop impls) | 350 |
| `crates/thetta-step/src/profile.rs` | Face selection, wire discretization, 2D projection | 250 |
| `crates/thetta-step/build.rs` | Link configuration for OCCT libraries (if needed) | 30 |
| `crates/thetta-step/tests/thickness_tests.rs` | Thickness detection tests | 60 |
| `crates/thetta-step/tests/profile_tests.rs` | Profile extraction tests | 80 |
| `crates/thetta-step/tests/roundtrip_test.rs` | Round-trip integration test | 70 |
| `test_data/step/` | Test STEP files (generated or checked in) | -- |
| **Total new** | | **~840** |

### Modified Files

| File | Changes | Est. Delta |
|------|---------|------------|
| `crates/thetta-step/Cargo.toml` | Add `opencascade-sys` (feature-gated), build deps | +10 |
| `crates/thetta-step/src/lib.rs` | Add `pub mod occt; pub mod profile;`, re-export types | +10 |
| `crates/thetta-step/src/import.rs` | Full rewrite: `todo!()` -> OCCT import pipeline | ~200 (rewrite) |
| `crates/thetta-step/src/export.rs` | Full rewrite: `todo!()` -> OCCT export pipeline | ~100 (rewrite) |
| `crates/thetta-app/Cargo.toml` | Add `rfd`, add `features = ["occt"]` to thetta-step dep | +3 |
| `crates/thetta-app/src/state.rs` | Add STEP session state (~7 fields), import/export/poll methods | +80 |
| `crates/thetta-app/src/panels.rs` | Add STEP collapsing section (load, thickness, export) | +60 |
| `crates/thetta-app/src/main.rs` | Wire up STEP polling and new PanelAction variants | +15 |
| **Total modified** | | **~480 delta** |

### Estimated Total Effort: ~1320 LOC (new + delta)

Current codebase is ~2550 LOC across all crates (post-Sprint-002).

## Definition of Done

### P0 -- Must Ship
- [x] OpenCASCADE installed on macOS ARM64 and `cargo build --release -p thetta-step --features occt` succeeds
- [x] `import_step()` reads a STEP file and returns a list of thickness groups with solid counts
- [x] Thickness grouping correctly identifies distinct thicknesses (0.1mm bucket tolerance)
- [x] Profile extraction produces `Vec<Piece>` with correct 2D polygons from sheet-metal solids
- [x] Extracted pieces feed into the existing nesting engine and produce a valid `NestingResult`
- [x] `export_sheet()` writes a valid STEP file with solids at nested positions
- [x] GUI "Load STEP" button opens a native file dialog and triggers import on a background thread
- [x] Thickness selector panel shows all groups; selecting one re-extracts profiles and re-nests
- [x] GUI "Export STEP" button writes one `.step` file per sheet
- [x] No panics or crashes during import/export of test files
- [x] All `unsafe` FFI code confined to `occt.rs` with RAII wrappers
- [x] All existing tests pass (`cargo test --workspace`)

### P1 -- Should Ship
- [x] Round-trip test passes: import -> nest -> export -> reimport -> areas match within 10%
- [x] Holes in solids are extracted as inner wires in the polygon (CCW outer, CW holes)
- [x] Non-planar faces are skipped gracefully with a warning (not a crash)
- [x] Progress indication during STEP import (spinner in GUI)
- [x] Error messages displayed in GUI for failed imports
- [x] Winding order verified and corrected automatically
- [x] Feature-gated tests: `cargo test -p thetta-step --features occt` passes

### P2 -- Stretch
- [ ] Support loading multiple STEP files (pieces accumulate across files)
- [ ] Face selection heuristic improved: prefer face with normal aligned to thickness axis
- [ ] Configurable mesh tolerance in GUI (advanced settings)
- [ ] STEP export preserves original solid colors/materials
- [ ] Export progress indication
- [ ] "Export selected sheet only" option

### Cut if Behind Schedule
- Multiple STEP file loading (P2)
- Face selection heuristic improvements (P2)
- Color/material preservation (P2)
- Configurable mesh tolerance UI (P2)

## Risks

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| `opencascade-sys` v0.2 lacks required API coverage (labeled "major work in progress" -- may be missing `GCPnts`, `BRepGProp`, `STEPControl_Writer` bindings) | High | **High** | **Phase 0 is a blocking investigation.** Evaluate API coverage before any other work. Fallbacks: higher-level `opencascade` crate, custom `bindgen` bindings, or thin C++ shim via `cc` crate. Budget extra time. |
| OpenCASCADE build/link failures on macOS ARM64 | High | Medium | Homebrew `opencascade` formula supports ARM64. Verify in Phase 0. Use explicit `rustc-link-search` in `build.rs` if `pkg-config` fails. |
| Largest-face heuristic selects wrong face for some geometries | Medium | Medium | For sheet-metal parts, largest planar face is almost always correct. Add normal-alignment heuristic as P2 fallback. |
| Edge discretization produces too many/few vertices | Medium | Low | Use `GCPnts_QuasiUniformDeflection` with configurable tolerance (default 0.5mm). Clamp to [2, 200] points per edge. |
| `StepModel` with OCCT data is not `Send`/`Sync` | High | Medium | Extract all Rust data during import. Keep OCCT solids behind `Arc<Mutex<>>` only for export. All OCCT work on dedicated threads. |
| STEP export produces invalid files other CAD tools cannot read | Medium | Medium | Validate by reimporting. Test with FreeCAD. Use `STEPControl_AsIs` mode. |
| No test STEP files available | Low | High | Generate at test-time using OCCT primitives (`BRepPrimAPI_MakeBox`). Supplement with FreeCAD-generated fixtures if needed. |
| OCCT memory management -- double-free or UAF in unsafe FFI | High | Medium | All `unsafe` in `occt.rs`. RAII wrappers with `Drop` impls. Run under AddressSanitizer during development. |
| Mixed-source export confusion (JSON + STEP pieces) | Low | Medium | STEP export skips non-STEP pieces with explicit warning. `source_file` on `Piece` already tracks origin. |

## Security

- No network access at runtime. Fully local application.
- OpenCASCADE and `opencascade-sys` are build-time dependencies only.
- STEP files are parsed by OpenCASCADE's C++ parser (battle-tested in production CAD systems).
- File I/O via native OS dialog (`rfd`). No arbitrary path traversal.
- All FFI code in `occt.rs` with explicit `unsafe` blocks. No `unsafe` in other modules.
- Export writes only to user-specified paths via save dialog.
- No user-provided code, scripts, or macros.
- Risk: malformed STEP files could trigger bugs in OCCT's C++ parser. Mitigation: OCCT is battle-tested; we propagate errors rather than panicking.

## Dependencies

### New Rust Crates

| Crate | Version | Purpose |
|-------|---------|---------|
| `opencascade-sys` | `0.2` | Low-level FFI bindings to OpenCASCADE (feature-gated). **Needs API coverage verification in Phase 0.** |
| `rfd` | `0.14` | Native macOS file dialogs |
| `tempfile` | `3` | Test dependency for round-trip test |

### Existing Crates (Unchanged from Sprint 002)

| Crate | Purpose |
|-------|---------|
| `thetta-geometry` | Shared types (`Piece`, `Polygon`, `Point2D`, `Placement`, `Sheet`, `NestingResult`) |
| `thetta-nesting` | `NestingEngine` (wraps jagua-rs `SPProblem`), LBF, GLS, benchmarks |
| `jagua-rs` | Collision detection (local fork, workspace member, `spp` feature) |
| `eframe` 0.31 / `egui` 0.31 | GUI framework |
| `serde` / `serde_json` | Serialization |
| `rand` | RNG for GLS optimizer |

### System Dependencies

| Dependency | Install Command | Purpose |
|------------|----------------|---------|
| OpenCASCADE (OCCT) | `brew install opencascade` | C++ geometry kernel for STEP read/write |
| CMake | `brew install cmake` | Required by `opencascade-sys` build |
| pkg-config | `brew install pkg-config` | Locates OCCT libraries |

Post-install verification:
```sh
brew install opencascade cmake pkg-config
pkg-config --libs opencascade
# Expected: -L/opt/homebrew/lib -lTKernel -lTKMath ...
```

## Open Questions

1. **`opencascade-sys` API coverage (BLOCKING):** v0.2 is labeled "major work in progress". Does it expose `STEPControl_Writer`, `GCPnts_QuasiUniformDeflection`, `BRepGProp::SurfaceProperties`, `BRepTools::OuterWire`? If not, evaluate: `opencascade` crate (higher-level wrapper), custom `bindgen`, or C++ shim via `cc`. **Must resolve in Phase 0.**

2. **Thread safety of OCCT objects:** Are `TopoDS_Solid` handles safe to read from multiple threads? Assume not. All OCCT operations on dedicated threads, communicate via channels (same pattern as existing optimizer threading).

3. **STEP unit handling:** Are we assuming mm? OCCT has unit conversion (`STEPConstruct_UnitContext`). Start with mm assumption, document as requirement.

4. **`PieceId` uniqueness across loads:** Current `AppState::load_json_file` appends pieces with their original IDs. STEP import should use `PieceId(solid_index)` for stability. May conflict if both JSON and STEP loaded. Consider centralized ID allocation or offset-based scheme.

5. **Face normal alignment heuristic:** Start with largest-face (simpler). Consider normal-alignment as P2 if real-world STEP files demonstrate issues.

6. **Mesh tolerance default:** 0.5mm absolute is reasonable for typical sheet-metal. Relative tolerance (% of bbox diagonal) is a future improvement.

7. **Export policy for mixed sessions:** STEP export excludes JSON pieces with warning. SVG/DXF export of nested layouts would be a separate feature.
