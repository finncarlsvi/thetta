# Sprint 003: STEP File Import & Export Pipeline

## Overview

Implement the full STEP file pipeline: load STEP files via OpenCASCADE FFI (`opencascade-sys`), detect thickness groups, extract 2D profiles from sheet-metal solids, feed them into the existing nesting engine, and export one STEP file per sheet with solids repositioned to their nested locations. This sprint also adds native file dialogs (`rfd`) and extends the GUI with a thickness selector panel.

**Scope:** OpenCASCADE system setup, STEP import (solid iteration, thickness detection, face extraction, wire discretization), STEP export (solid repositioning, STEP writer), GUI integration (file dialogs, thickness selector), round-trip testing.
**Prerequisites:** Sprint 002 (jagua-rs integration + benchmarks) must be complete. The nesting engine accepts `Vec<Piece>` and produces `NestingResult` with `Vec<Placement>` -- this sprint produces pieces from STEP and consumes placements back into STEP.
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
- After thickness selection, the extracted `Vec<Piece>` feeds into the existing nesting pipeline (LBF baseline then GLS optimizer)
- Background optimization runs identically to Sprint 001/002 (mpsc channel, AtomicBool cancel, progressive updates)
- Viewport shows nested result with real-time improvements
- All existing nesting controls work (start/stop, separation, sheet size)

### UC4: Export Nested Sheets as STEP Files
- User clicks "Export STEP" button
- Native macOS save dialog opens, user picks output directory and base filename
- For each sheet in the nesting result, the app writes `{basename}_sheet_{N}.step`
- Each output STEP file contains the original 3D solids repositioned to their nested 2D locations
- On completion, a status label shows "Exported N sheets"

### UC5: Mixed Input Sources
- User can load JSON polygon files and STEP files in one session
- Pieces keep their source attribution (for coloring + management)
- STEP export only includes pieces that originate from STEP; JSON-only pieces are excluded with a clear warning

### UC6: Round-Trip Verification (Developer)
- Developer runs an integration test: import STEP -> extract profiles -> nest -> export STEP -> reimport -> profile counts and areas match within tolerance

## Architecture

### Crate Structure (Updated)

```
thetta/
+-- Cargo.toml                    (workspace)
+-- crates/
|   +-- thetta-geometry/          (2D types, polygon ops, serde)
|   +-- thetta-nesting/           (LBF + GLS optimizer, jagua-rs integration)
|   +-- thetta-app/               (egui GUI, viewport, panels, state)
|   +-- thetta-step/              (STEP import/export via opencascade-sys)  <-- IMPLEMENTED
+-- test_data/
|   +-- step/                     (test STEP files)  <-- NEW
```

### Dependency Graph (Updated)

```
thetta-app
+-- thetta-nesting
|   +-- thetta-geometry
|   +-- jagua-rs
+-- thetta-step              <-- NEW real integration
|   +-- thetta-geometry
|   +-- opencascade-sys      <-- NEW FFI bindings (feature-gated)
+-- thetta-geometry
+-- eframe / egui
+-- rfd (file dialogs)
```

Note: `thetta-step` depends on `thetta-geometry` for `Piece`, `Polygon`, `Point2D` types. It does NOT depend on `thetta-nesting` -- the app layer orchestrates the pipeline.

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
  |  (LBF + GLS)     |<----|    CDEngine      |
  +--------+---------+     +------------------+
           |
           v
     NestingResult { sheets: Vec<Sheet> }
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

/// Detected thickness group
#[derive(Debug, Clone)]
pub struct ThicknessInfo {
    pub value_mm: f64,
    pub count: usize,
}

/// Result of a STEP import operation
pub struct StepImportResult {
    pub model: StepModel,
    pub pieces: Vec<Piece>,
    pub piece_refs: Vec<StepPieceRef>,
    pub thicknesses: Vec<ThicknessInfo>,
    pub selected_thickness_mm: f64,
}

/// Import error types
pub enum StepImportError {
    FileNotFound,
    ReadFailed(String),
    NoSolidsFound,
    NoProfilesExtracted { skipped: usize, reason: String },
    OcctError(String),
}

/// Export error types
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

2. **Feature-gated OCCT:** `opencascade-sys` is behind `--features occt` so the workspace builds without OCCT installed. Only `thetta-step` and `thetta-app` (when loading STEP) need this feature.

3. **Thread safety strategy:** Extract all Rust data (`Vec<Piece>`, `Vec<ThicknessInfo>`) during import. Keep OCCT solids in `StepModel` behind `Arc<Mutex<>>` only for export. This avoids Send/Sync issues for the UI thread.

4. **Deterministic PieceIds:** Use `PieceId(solid_index as u64)` so IDs are stable across thickness switches.

### Threading Model

```
Main Thread (egui)                    STEP Import Thread
+-------------------+               +---------------------+
| - Triggers import | -- path -->   | - Calls import_step |
| - Shows spinner   |               |   (all OCCT work)   |
| - Polls result    | <-- mpsc --   | - Sends result      |
+-------------------+               +---------------------+

Main Thread (egui)                    STEP Export Thread
+-------------------+               +---------------------+
| - Triggers export | -- data -->   | - Calls export_all  |
| - Shows progress  |               |   _sheets           |
| - Polls done      | <-- mpsc --   | - Sends status      |
+-------------------+               +---------------------+
```

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

#### Task 0.2: Add opencascade-sys dependency
**File:** `crates/thetta-step/Cargo.toml` (MODIFY)
```toml
[features]
default = []
occt = ["opencascade-sys"]

[dependencies]
thetta-geometry = { path = "../thetta-geometry" }
opencascade-sys = { version = "0.2", optional = true }
```

#### Task 0.3: Smoke test
```rust
#[cfg(feature = "occt")]
#[test]
fn occt_smoke_test() {
    // Create a STEPControl_Reader and verify it initializes
    // This confirms linking works
}
```

**Deliverable:** `cargo build -p thetta-step --features occt` succeeds on macOS ARM64.

### Phase 1: Safe OCCT FFI Wrapper

**Goal:** Centralize all `unsafe` FFI in a dedicated module with RAII wrappers.

#### Task 1.1: OCCT wrapper module
**File:** `crates/thetta-step/src/occt.rs` (NEW)

Safe Rust wrappers around OCCT types:

```rust
/// Safe wrapper around STEPControl_Reader
pub struct StepReader { /* raw OCCT handle */ }

impl StepReader {
    pub fn read_file(path: &Path) -> Result<Self, StepImportError>;
    pub fn transfer_roots(&mut self) -> usize;
    pub fn root_shape(&self) -> OcctShape;
}

impl Drop for StepReader { /* release OCCT resources */ }

/// Safe wrapper around STEPControl_Writer
pub struct StepWriter { /* raw OCCT handle */ }

impl StepWriter {
    pub fn new() -> Self;
    pub fn add_shape(&mut self, shape: &OcctShape);
    pub fn write(&self, path: &Path) -> Result<(), StepExportError>;
}

/// Safe wrapper around topology exploration
pub struct SolidExplorer { /* wraps TopExp_Explorer */ }

impl SolidExplorer {
    pub fn new(shape: &OcctShape) -> Self;
    pub fn solids(&self) -> Vec<OcctSolid>;
}

/// Safe bounding box computation
pub fn bounding_box(solid: &OcctSolid) -> [f64; 6]; // xmin,ymin,zmin,xmax,ymax,zmax

/// Safe face area computation
pub fn face_area(face: &OcctFace) -> f64;

/// Safe face planarity check + normal
pub fn face_plane(face: &OcctFace) -> Option<PlaneInfo>;

pub struct PlaneInfo {
    pub origin: [f64; 3],
    pub x_dir: [f64; 3],
    pub y_dir: [f64; 3],
    pub normal: [f64; 3],
}

/// Safe wire discretization
pub fn discretize_wire(
    wire: &OcctWire,
    plane: &PlaneInfo,
    tolerance: f64,
    max_points_per_edge: usize,
) -> Vec<Point2D>;

/// Safe transform application
pub fn transform_shape(
    shape: &OcctShape,
    x: f64, y: f64, rotation: f64,
) -> OcctShape;
```

All `unsafe` FFI calls are confined to this module. Every OCCT handle has a `Drop` impl.

### Phase 2: STEP Import Pipeline

#### Task 2.1: Solid enumeration + thickness detection
**File:** `crates/thetta-step/src/import.rs` (REWRITE)

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
4. For each solid, compute bounding box:
   - `bounding_box(solid)` -> `[xmin, ymin, zmin, xmax, ymax, zmax]`
   - `thickness = min(dx, dy, dz)`
5. Group by thickness: `bucket = (thickness / config.thickness_bucket_mm).round() * config.thickness_bucket_mm`
6. Default selection: bucket with most solids (unless `thickness` parameter provided)

#### Task 2.2: Profile face selection
**File:** `crates/thetta-step/src/profile.rs` (NEW)

Face selection heuristic:
1. For each solid in the selected thickness bucket:
   - Enumerate faces via `TopExp_Explorer(solid, TopAbs_FACE)`
   - For each face, check planarity: `face_plane(face)` returns `Some(PlaneInfo)` if planar
   - Compute area: `face_area(face)`
   - Pick largest planar face by area
   - If no planar face found: skip solid, collect warning

#### Task 2.3: Wire extraction + curve discretization

For the selected face:
1. Outer wire: `BRepTools::OuterWire(face)`
2. Holes: iterate all wires; any wire != outer is a hole
3. For each wire, iterate edges with `BRepTools_WireExplorer`:
   - Get curve: `BRep_Tool::Curve(edge, first, last)`
   - Discretize: `GCPnts_QuasiUniformDeflection(adaptor, config.mesh_tolerance_mm)`
   - Evaluate curve at sample parameters -> 3D points
   - Project to 2D using face's `PlaneInfo`:
     - `u = (P - origin) . x_dir`
     - `v = (P - origin) . y_dir`
4. Post-process:
   - Remove adjacent duplicate points (within epsilon)
   - Ensure outer ring is CCW (compute signed area; reverse if negative)
   - Ensure hole rings are CW
   - Close ring (first == last, or close by appending first point)

Output: `Piece { id: PieceId(solid_index), polygon, source_file: path, quantity: 1 }`

#### Task 2.4: StepModel construction

Assemble `StepImportResult`:
- `model`: owns all OCCT solid handles for later export
- `pieces`: extracted 2D profiles
- `piece_refs`: parallel mapping of `PieceId` -> `StepSolidKey`
- `thicknesses`: sorted by count descending
- `selected_thickness_mm`: auto-selected or user-specified

### Phase 3: STEP Export Pipeline

#### Task 3.1: Sheet export
**File:** `crates/thetta-step/src/export.rs` (REWRITE)

```rust
pub fn export_sheet(
    model: &StepModel,
    piece_refs: &[StepPieceRef],
    sheet: &Sheet,
    output_path: &Path,
) -> Result<(), StepExportError>
```

For each placement in the sheet:
1. Look up `StepSolidKey` via `piece_refs` for the `piece_id`
2. Retrieve original `TopoDS_Solid` from `model`
3. Build transform:
   - Rotation around Z-axis: `gp_Trsf::SetRotation(gp_Ax1(origin, Z), rotation)`
   - Translation: `gp_Trsf::SetTranslation(gp_Vec(x, y, 0))`
   - Combine: `translate * rotate` (rotate first, then translate)
4. Apply: `BRepBuilderAPI_Transform(solid, trsf, true)` -> moved shape
5. Add to writer: `writer.add_shape(moved_shape)`
6. Write: `writer.write(output_path)`

#### Task 3.2: Multi-sheet export
**File:** `crates/thetta-step/src/export.rs`

```rust
pub fn export_all_sheets(
    model: &StepModel,
    piece_refs: &[StepPieceRef],
    result: &NestingResult,
    out_dir: &Path,
    file_stem: &str,
) -> Result<Vec<PathBuf>, StepExportError>
```

Iterates sheets: `{file_stem}_sheet_001.step`, `{file_stem}_sheet_002.step`, etc.
Skips pieces without STEP source (JSON-only pieces) with a logged warning.

### Phase 4: GUI Integration

#### Task 4.1: Add rfd dependency
**File:** `crates/thetta-app/Cargo.toml` (MODIFY)
- Add `rfd = "0.14"` if not already present
- Add `thetta-step = { path = "../thetta-step", features = ["occt"] }`

#### Task 4.2: STEP panel in side panel
**File:** `crates/thetta-app/src/panels.rs` (MODIFY)

Add STEP section:
- "Load STEP" button -> `rfd::FileDialog` filtered to `*.step`, `*.stp`
- Loading spinner while import runs
- Error display label
- Thickness selector (radio buttons with `{thickness_mm} mm ({count} solids)`)
- "Export STEP" button -> `rfd::FileDialog` save dialog

#### Task 4.3: Background STEP import
**File:** `crates/thetta-app/src/state.rs` (MODIFY)

New fields in `AppState`:
```rust
pub step_session: Option<Arc<Mutex<StepModel>>>,
pub step_piece_refs: Vec<StepPieceRef>,
pub thickness_groups: Vec<ThicknessInfo>,
pub selected_thickness_idx: usize,
pub step_loading: bool,
pub step_error: Option<String>,
pub step_import_rx: Option<Receiver<Result<StepImportResult, StepImportError>>>,
```

Methods:
- `start_step_import(path)`: spawn thread, store receiver
- `poll_step_import()`: called each frame, `try_recv()` for result
- `select_thickness(idx)`: re-extract profiles from in-memory model, restart nesting
- `start_step_export(path)`: spawn thread for export

#### Task 4.4: Wire up in main app
**File:** `crates/thetta-app/src/main.rs` (MODIFY)
- Add `state.poll_step_import()` to the update loop
- Add `step_panel()` call in the side panel layout

### Phase 5: Testing

#### Task 5.1: Generate test STEP files
**Option A (preferred):** Generate at test-time using OCCT primitives:
```rust
#[cfg(feature = "occt")]
fn create_test_box(w: f64, h: f64, t: f64, path: &Path) {
    // Use BRepPrimAPI_MakeBox to create a box solid
    // Write it to a STEP file using StepWriter
}
```

**Option B:** Check in 1-2 tiny `.step` fixtures under `test_data/step/` for quick validation.

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
```

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
| `crates/thetta-app/Cargo.toml` | Add `thetta-step` dependency, `rfd` | +5 |
| `crates/thetta-app/src/state.rs` | Add STEP session state, import/export methods, polling | +80 |
| `crates/thetta-app/src/panels.rs` | Add STEP section (load, thickness selector, export) | +60 |
| `crates/thetta-app/src/main.rs` | Wire up STEP polling and panel | +15 |
| `Cargo.toml` | Workspace dep for `opencascade-sys` | +3 |
| **Total modified** | | **~480 delta** |

### Estimated Total Effort: ~1320 LOC (new + delta)

## Definition of Done

### P0 -- Must Ship
- [ ] OpenCASCADE installed on macOS ARM64 and `cargo build --release -p thetta-step --features occt` succeeds
- [ ] `import_step()` reads a STEP file and returns a list of thickness groups with solid counts
- [ ] Thickness grouping correctly identifies distinct thicknesses (0.1mm bucket tolerance)
- [ ] Profile extraction produces `Vec<Piece>` with correct 2D polygons from sheet-metal solids
- [ ] Extracted pieces feed into the existing nesting engine (LBF + GLS) and produce a valid result
- [ ] `export_sheet()` writes a valid STEP file with solids at nested positions
- [ ] GUI "Load STEP" button opens a native file dialog and triggers import on a background thread
- [ ] Thickness selector panel shows all groups; selecting one re-extracts profiles and re-nests
- [ ] GUI "Export STEP" button writes one `.step` file per sheet
- [ ] No panics or crashes during import/export of test files
- [ ] All `unsafe` FFI code confined to `occt.rs` with RAII wrappers

### P1 -- Should Ship
- [ ] Round-trip test passes: import -> nest -> export -> reimport -> areas match within 5%
- [ ] Holes in solids are extracted as inner wires in the polygon (CCW outer, CW holes)
- [ ] Non-planar faces are skipped gracefully with a warning (not a crash)
- [ ] Progress indication during STEP import (spinner in GUI)
- [ ] Error messages displayed in GUI for failed imports
- [ ] Winding order verified and corrected automatically
- [ ] Feature-gated tests: `cargo test -p thetta-step --features occt` passes

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
| `opencascade-sys` lacks required API coverage (missing bindings for `GCPnts`, `BRepGProp`, `STEPControl_Writer`) | High | High | Evaluate API coverage in Phase 0. If insufficient, consider the higher-level `opencascade` crate or write custom `bindgen` bindings. Budget extra time for this investigation. |
| OpenCASCADE build/link failures on macOS ARM64 | High | Medium | Homebrew `opencascade` formula supports ARM64. Verify early. Use explicit `rustc-link-search` in `build.rs` if `pkg-config` fails. |
| Largest-face heuristic selects wrong face for some geometries | Medium | Medium | For sheet-metal parts, largest planar face is almost always correct. Add normal-alignment heuristic as P2 fallback. |
| Edge discretization produces too many/few vertices | Medium | Low | Use `GCPnts_QuasiUniformDeflection` with configurable tolerance (default 0.5mm). Clamp to [2, 200] points per edge. |
| `StepModel` with OCCT data is not `Send`/`Sync` | High | Medium | Extract all needed Rust data during import. Keep OCCT solids behind `Arc<Mutex<>>` only for export. Do all OCCT work on dedicated threads. |
| STEP export produces invalid files that other CAD tools cannot read | Medium | Medium | Validate by reimporting. Test with FreeCAD. Use `STEPControl_AsIs` mode. |
| No test STEP files available | Low | High | Generate at test-time using OCCT primitives (`BRepPrimAPI_MakeBox`). Supplement with FreeCAD-generated fixtures if needed. |
| OCCT memory management -- double-free or UAF in unsafe FFI | High | Medium | All `unsafe` in `occt.rs`. RAII wrappers with `Drop` impls. Run under AddressSanitizer during development. |
| Mixed-source export confusion (JSON + STEP pieces) | Low | Medium | STEP export skips non-STEP pieces with explicit warning. Clear UX messaging. |

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
| `opencascade-sys` | `0.2` | Low-level FFI bindings to OpenCASCADE (feature-gated) |
| `rfd` | `0.14` | Native macOS file dialogs |
| `tempfile` | `3` | Test dependency for round-trip test |

### Existing Crates (Unchanged)

| Crate | Purpose |
|-------|---------|
| `thetta-geometry` | Shared types (`Piece`, `Polygon`, `Point2D`, `Placement`) |
| `eframe` / `egui` | GUI framework |
| `serde` / `serde_json` | Serialization |
| `jagua-rs` | Collision detection engine (from Sprint 002) |
| `rand` | RNG for optimizer |

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

If `pkg-config` does not find OCCT:
```sh
export PKG_CONFIG_PATH="/opt/homebrew/opt/opencascade/lib/pkgconfig:$PKG_CONFIG_PATH"
```

## Open Questions

1. **`opencascade-sys` API coverage:** Does it expose `STEPControl_Writer`, `GCPnts_QuasiUniformDeflection`, `BRepGProp::SurfaceProperties`, and `BRepTools::OuterWire`? If not, what's the fallback -- `opencascade` crate, custom `bindgen`, or direct C++ shim? **Action:** Evaluate in Phase 0 before committing.

2. **Thread safety of OCCT objects:** Are `TopoDS_Solid` handles safe to read from multiple threads? If not, must all OCCT work go through a dedicated thread? **Recommendation:** Assume not thread-safe. Do all OCCT operations on dedicated threads; communicate via channels.

3. **Face normal alignment heuristic:** Start with largest-face (simpler). Consider adding normal-alignment as P2 if real-world STEP files demonstrate issues.

4. **Mesh tolerance default:** 0.5mm absolute is reasonable for typical sheet-metal parts. For very large or very small parts, consider relative tolerance (% of bounding box diagonal) as a future improvement.

5. **Test STEP file sourcing:** Generate at test-time using OCCT primitives (no external files needed). Optionally also generate via FreeCAD Python scripting for richer test cases.

6. **STEP unit handling:** Are we assuming STEP units are mm? OCCT has unit conversion facilities (`STEPConstruct_UnitContext`). Should we detect and convert, or assume mm and document the requirement?

7. **Export policy for mixed sessions:** When JSON and STEP pieces coexist, STEP export excludes JSON pieces with a warning. Should we also support "JSON-only export" (e.g., SVG)?

8. **`PieceId` uniqueness:** Current app doesn't enforce uniqueness across multiple loads. Sprint 003 should use deterministic IDs (`PieceId(solid_index)`) for STEP pieces. May need centralized ID allocation if multi-file loading is added later.
