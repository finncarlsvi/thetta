# Sprint 003: Thetta STEP Import/Export Pipeline — Codex Draft

## Overview

Sprint 003 implements Thetta’s end-to-end STEP pipeline on macOS ARM64: STEP import via OpenCASCADE (preferred: `opencascade-sys`), thickness-group discovery, 2D profile extraction from selected solids, and STEP export of nested results (one STEP file per sheet) with original 3D solids transformed to their nested XY placements.

Context / sequencing:
- Sprint 002 has just started and is focused on benchmark-grade nesting and collision/optimizer quality. Sprint 003 is the STEP I/O + geometry extraction bridge that turns the nesting engine into the product.
- Today: `crates/thetta-step/src/import.rs` and `crates/thetta-step/src/export.rs` are `todo!()` stubs; `thetta-app` uses a manual path text field; neither OpenCASCADE nor any STEP fixtures are set up.

Primary outcomes:
1. Install + link OpenCASCADE on macOS ARM64 (local dev first; CI considered but may be deferred).
2. Implement STEP import in `thetta-step`:
   - Load STEP
   - Enumerate solids
   - Compute thickness buckets + default selection
   - Extract planar 2D profiles (outer + holes) for a selected thickness
   - Provide stable IDs and retain a mapping back to the original OCCT solids for later export
3. Implement STEP export in `thetta-step`:
   - For each sheet, apply XY translation + Z-axis rotation to each original solid
   - Write a STEP per sheet
4. Wire the UI:
   - Replace manual path entry with native file dialogs (`rfd`)
   - Add thickness selection UX for imported STEP models
   - Add export controls (output folder + naming scheme)

Non-goals (unless needed to unblock):
- Perfect “works on every STEP in the world” support (we focus on typical sheet-metal-like solids).
- Advanced face selection UI (we start with a deterministic heuristic and “skip gracefully” behavior).
- Full multi-sheet nesting improvements (export supports multiple sheets if the nesting engine produces them, but Sprint 003 does not require implementing multi-sheet nesting itself).

## Use Cases

1. UC1: Load STEP (single file)
- User clicks “Load STEP…”
- Native file picker opens; user selects a `.step`/`.stp`
- App loads the model (background thread), then displays:
  - Detected thickness buckets (value + count)
  - Default thickness selection (most common bucket)
- App extracts profiles for the selected thickness into `Vec<Piece>` and starts nesting automatically.

2. UC2: Switch thickness
- User picks a different thickness bucket.
- App re-extracts profiles for solids in that bucket (without re-reading the STEP file), refreshes the piece list, and re-nests.

3. UC3: Mixed input sources
- User can load JSON polygon files and STEP files in one session.
- Pieces keep their source attribution (for coloring + management).
- Export STEP only includes pieces that originate from STEP (JSON pieces either:
  - are excluded from STEP export, or
  - cause a clear error/warning; decision is explicit).

4. UC4: Export nested sheets as STEP
- User clicks “Export STEP…”
- User chooses an output directory (or a file stem).
- App writes `sheet_001.step`, `sheet_002.step`, … (one per sheet).
- Exported solids are translated/rotated into their final XY placements.

5. UC5: Round-trip validation workflow (developer)
- Developer runs an integration test:
  - import fixture STEP → extract profiles → nest → export STEP → re-import exported STEP → extracted profile counts/areas roughly match.

## Architecture

### Crate responsibilities (target state)

- `thetta-geometry`:
  - Stays pure 2D and serialization-friendly (`Piece`, `Polygon`, `Placement`, `Sheet`, `NestingResult`).
  - No OCCT types here.

- `thetta-step`:
  - Owns all OpenCASCADE FFI (`unsafe`) and encapsulates OCCT handles safely.
  - Produces `Vec<Piece>` for nesting and retains an opaque mapping to source solids for export.

- `thetta-app`:
  - Owns session state:
    - loaded STEP “models” (opaque handles provided by `thetta-step`)
    - pieces list (what nesting runs on)
    - UI state for thickness selection per model
    - export actions

### Key design decision: keep OCCT solids out of `Piece`

`Piece` currently has `id`, `polygon`, `source_file`, `quantity` only. Export needs “piece_id → original TopoDS_Solid”.
We do **not** add OCCT references into `thetta-geometry` types. Instead:
- `thetta-step` returns a `StepModel` that owns all OCCT state.
- Pieces carry stable `PieceId`s that map into `StepModel` tables.

### Proposed `thetta-step` API (Rust types)

```rust
// crates/thetta-step/src/import.rs

use std::path::{Path, PathBuf};
use thetta_geometry::types::{Piece, PieceId};

#[derive(Debug, Clone)]
pub struct ThicknessInfo {
    pub value_mm: f64,
    pub count: usize,
}

#[derive(Debug)]
pub struct StepModel {
    // Opaque, owns OCCT resources.
    // Internally: reader/document + Vec<StepSolidRecord>.
    _priv: (),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StepSolidKey(pub u32);

#[derive(Debug, Clone)]
pub struct StepPieceRef {
    pub piece_id: PieceId,
    pub solid_key: StepSolidKey,
    pub source_path: PathBuf,
    pub thickness_mm: f64,
}

#[derive(Debug)]
pub struct StepImportResult {
    pub model: StepModel,
    pub pieces: Vec<Piece>,
    pub piece_refs: Vec<StepPieceRef>,     // parallel to `pieces` by `piece_id`
    pub thicknesses: Vec<ThicknessInfo>,
    pub selected_thickness_mm: f64,
}

#[derive(Debug)]
pub enum StepImportError {
    FileNotFound,
    ReadFailed(String),
    ParseError(String),
    NoSolidsFound,
    NoProfilesExtracted(String),
    OcctUnavailable(String),
}


```rust
// crates/thetta-step/src/export.rs

use std::path::Path;
use thetta_geometry::types::{NestingResult, Sheet};

use crate::import::{StepModel, StepPieceRef};

#[derive(Debug)]
pub enum StepExportError {
    InvalidOutputPath(String),
    MissingSolidForPiece(u64),
    WriteError(String),
    OcctError(String),
}

pub fn export_sheet(
    model: &StepModel,
    piece_refs: &[StepPieceRef],
    sheet: &Sheet,
    output_path: &Path,
) -> Result<(), StepExportError> { /* ... */ }

pub fn export_all_sheets(
    model: &StepModel,
    piece_refs: &[StepPieceRef],
    result: &NestingResult,
    out_dir: &Path,
    file_stem: &str,
) -> Result<Vec<std::path::PathBuf>, StepExportError> { /* ... */ }


### App wiring (high level)

- `AppState` stores:
  - `pieces: Vec<Piece>` (as today)
  - plus: `step_sessions: Vec<LoadedStepModel>` where each entry has:
    - `path`
    - `model: StepModel`
    - `piece_refs`
    - `thicknesses`
    - `selected_thickness_mm`
- When a STEP model is loaded or thickness changes:
  - `pieces` is updated (replace those from that model; keep others)
  - optimizer restarts (existing threading model remains)

## Implementation (Phased with Rust Types and OCCT API Calls)

### Phase 0 (P0): OpenCASCADE + `opencascade-sys` bring-up on macOS ARM64

Goals:
- `thetta-step` compiles and links on Apple Silicon.
- A minimal smoke test can call into OCCT and return success.

Tasks:
1. Add dependency:
   - `crates/thetta-step/Cargo.toml`: add `opencascade-sys = "..."`
   - Add any required build helpers (`pkg-config`, `cmake`, etc.) only if `opencascade-sys` requires them.
2. Document local install:
   - Prefer Homebrew on macOS:
     - `brew install opencascade`
   - Decide how `opencascade-sys` finds headers/libs (env vars vs `pkg-config`).
3. Add a tiny `#[test]` behind a feature gate:
   - `--features occt` enables OCCT-dependent code/tests.
   - Without OCCT present, CI/dev can still build other crates.

Deliverable:
- `cargo build -p thetta-step --features occt` succeeds on macOS ARM64.

OCCT calls (smoke-level):
- Create a `Bnd_Box` and ensure it constructs, or initialize a `STEPControl_Reader` and ensure a `ReadFile` call returns a status enum.

### Phase 1 (P0): STEP read + solids enumeration + thickness buckets

Goal:
- Load STEP and discover thickness groups without profile extraction yet.

Rust entrypoint:
```rust
pub fn import_step(path: &Path, thickness: Option<f64>, tolerance: f64) -> Result<StepImportResult, StepImportError>


OCCT pipeline (core calls):
1. `STEPControl_Reader reader;`
2. `IFSelect_ReturnStatus status = reader.ReadFile(path);`
3. `reader.TransferRoots();`
4. `TopoDS_Shape root = reader.OneShape();` (or iterate transferred roots)
5. Enumerate solids:
   - `TopExp_Explorer exp(root, TopAbs_SOLID);`
   - `TopoDS_Solid solid = TopoDS::Solid(exp.Current());`
6. Thickness estimate per solid:
   - `Bnd_Box box;`
   - `BRepBndLib::Add(solid, box);`
   - `box.Get(xmin, ymin, zmin, xmax, ymax, zmax);`
   - `dx = xmax-xmin; dy = ymax-ymin; dz = zmax-zmin; thickness = min(dx,dy,dz);`

Bucketing:
- `bucket = (thickness_mm / tolerance).round() * tolerance`
- Track counts per bucket.
- Default selection: bucket with max count unless caller provided `thickness`.

Data stored in `StepModel`:
- For each solid:
  - `solid_key`
  - `thickness_mm`
  - handle to the `TopoDS_Solid` (owned by `StepModel`)

Deliverable:
- `import_step(..)` returns:
  - `thicknesses: Vec<ThicknessInfo>`
  - `selected_thickness_mm`
  - `model` containing solids
  - `pieces` may be empty at this phase (or include placeholder polygons behind a flag)

### Phase 2 (P0): Profile face selection + planar validation + 2D projection

Goal:
- For solids in the selected thickness bucket, extract a robust planar “profile face” and convert it into a 2D polygon (outer + holes) in a stable coordinate frame.

Face selection heuristic (deterministic, no UI):
1. Enumerate faces:
   - `TopExp_Explorer fexp(solid, TopAbs_FACE)`
2. For each face:
   - Validate planar:
     - `Handle(Geom_Surface) surf = BRep_Tool::Surface(face)`
     - `Handle(Geom_Plane) plane = Handle(Geom_Plane)::DownCast(surf)`
     - if null: skip face (non-planar)
   - Compute area:
     - `GProp_GProps props;`
     - `BRepGProp::SurfaceProperties(face, props);`
     - `area = props.Mass();`
3. Pick largest planar face by area.
4. If none: mark solid “no profile available” (skip gracefully, collect warning).

2D coordinate frame:
- Use the plane’s `gp_Ax3` to define (origin, x_dir, y_dir).
- For any 3D point `P`:
  - `V = P - origin`
  - `u = V · x_dir`
  - `v = V · y_dir`
- This yields consistent 2D coordinates even if the part is rotated in world space.

Deliverable:
- For each selected solid, we can choose a face and project points to its 2D plane.

### Phase 3 (P0): Wire extraction (outer + holes) + curve discretization

Goal:
- Convert the chosen face into `thetta_geometry::types::Polygon` with correct winding.

Wire extraction:
- Outer wire:
  - `TopTools_IndexedMapOfShape map;` (optional)
  - `TopoDS_Wire outer = BRepTools::OuterWire(face);`
- Holes:
  - iterate all wires on face:
    - `TopExp_Explorer wexp(face, TopAbs_WIRE)`
    - any wire != outer is a hole

Discretization (edge → polyline):
- For each wire:
  - Iterate edges in order with `BRepTools_WireExplorer` (preferred for ordered traversal)
  - For each edge:
    - `Standard_Real first, last;`
    - `Handle(Geom_Curve) curve = BRep_Tool::Curve(edge, first, last);`
    - Sample parameters:
      - Use `GCPnts_QuasiUniformDeflection` (deflection-based) OR `GCPnts_UniformAbscissa` (length step-based)
      - Target parameters derived from `tolerance` (e.g., `deflection = tolerance`)
    - Evaluate curve at params to points:
      - `gp_Pnt p = curve->Value(t);`
    - Project `gp_Pnt` to (u,v) using Phase 2 plane frame
- Post-process:
  - Remove adjacent duplicates within epsilon
  - Ensure ring is closed (first == last or explicitly close)
  - Normalize to open/closed representation expected by `Polygon.outer` (Thetta uses open list; rendering assumes polygon is closed by modulo indexing)
  - Ensure winding:
    - compute signed area; enforce outer CCW, holes CW

Rust output:
- Create `Piece { id: PieceId(..), polygon, source_file, quantity: 1 }`
- Create `StepPieceRef { piece_id, solid_key, source_path, thickness_mm }`

Deliverable:
- `import_step(..)` produces non-empty `pieces` for typical planar sheet parts.
- Skips non-planar/unextractable solids without crashing; returns warnings in error text or logs (exact surfacing is a UI decision).

### Phase 4 (P0): App UI integration (file dialogs + thickness selection)

Goal:
- Make import usable from the GUI and match the existing background-thread model.

Tasks:
1. Add `rfd` dependency to `crates/thetta-app/Cargo.toml`.
2. Replace manual path input with:
   - “Load JSON…” (optional keep)
   - “Load STEP…”
3. Load STEP on a background thread (similar to optimizer) to avoid UI stalls:
   - Send back either `StepImportResult` or a structured error
4. Add thickness selector UI:
   - For the selected model: show a dropdown/list of `ThicknessInfo`
   - On change: trigger “re-extract pieces for this model” using the in-memory `StepModel`
5. Restart optimizer after any piece-set change.

Deliverable:
- User can load STEP via GUI and see extracted profiles nested.

### Phase 5 (P0/P1): STEP export (one STEP per sheet)

Goal:
- Export nested results as STEP with solids transformed in XY according to placements.

Core mapping:
- Build `HashMap<PieceId, StepSolidKey>` from `piece_refs`.

OCCT transformation + write:
1. For each `Placement { piece_id, x, y, rotation }`:
   - Fetch `TopoDS_Solid solid` for `piece_id`
2. Build transform:
   - `gp_Trsf rot; rot.SetRotation(gp_Ax1(gp_Pnt(0,0,0), gp_Dir(0,0,1)), rotation);`
   - `gp_Trsf trn; trn.SetTranslation(gp_Vec(x, y, 0));`
   - Combine (order matters; we standardize it and document):
     - `gp_Trsf trsf = trn * rot;` (or `rot` then translate; choose and test)
3. Apply:
   - `BRepBuilderAPI_Transform xf(solid, trsf, true);`
   - `TopoDS_Shape moved = xf.Shape();`
4. Writer:
   - `STEPControl_Writer writer;`
   - `writer.Transfer(moved, STEPControl_AsIs);` (or appropriate mode)
   - `writer.Write(output_path);`

File naming:
- `export_all_sheets`: `"{file_stem}_sheet_{:03}.step"`

Deliverable:
- Export command writes STEP files successfully for parts extracted from STEP.
- Clear error if a placement references a non-STEP piece (or missing mapping).

### Phase 6 (P1): Fixtures + automated tests (round-trip)

Goal:
- Add minimal STEP fixtures and tests to prevent regressions.

Because STEP fixtures aren’t set up yet, pick one of:
- Option A (preferred if feasible): check in 1–2 tiny `.step` fixtures under `test_data/step/` (license-safe, tiny geometry).
- Option B: generate fixtures at test-time using OCCT primitives (box/extrusion) and write them to a temp dir, then re-import.

Test ideas:
1. Thickness bucketing test:
   - Fixture with two solids of different thickness → expect two buckets with correct counts.
2. Profile extraction test:
   - Known planar face → polygon area within tolerance.
3. Export/import round trip:
   - Import → export sheet → import exported → piece count matches (within reason).

Deliverable:
- `cargo test -p thetta-step --features occt` runs on a configured macOS ARM64 dev machine.

## Files Summary

New / updated files (expected):
- `crates/thetta-step/Cargo.toml` (add `opencascade-sys`, feature flags)
- `crates/thetta-step/src/import.rs` (implement import, define `StepModel`, `StepImportResult`, thickness logic, profile extraction)
- `crates/thetta-step/src/export.rs` (implement export functions)
- `crates/thetta-step/src/lib.rs` (re-exports + feature gating)
- `crates/thetta-app/Cargo.toml` (add `rfd`)
- `crates/thetta-app/src/panels.rs` (buttons: Load STEP, thickness selector, Export STEP)
- `crates/thetta-app/src/state.rs` (store loaded STEP models + piece_refs; background import flow)
- `test_data/step/*` (optional fixtures)
- `docs/` updates as needed: installation notes for OCCT on macOS ARM64

## Definition of Done

### P0 (must ship for Sprint 003)
- STEP import works on macOS ARM64 with `opencascade-sys` and a Homebrew-installed OCCT:
  - detects thickness buckets
  - extracts 2D profiles for the selected thickness into `Vec<Piece>`
- UI supports native file dialog for STEP load (no manual path typing)
- Nesting runs on extracted pieces (no app architecture regression)
- STEP export writes one STEP per sheet with solids transformed to placements
- Export errors are explicit (missing mapping, non-STEP pieces, write failure)

### P1 (should ship if time allows)
- Thickness selection UI changes re-extract without re-reading the STEP file
- 1–2 automated OCCT-dependent tests run locally (feature-gated)
- Hole extraction supported (inner wires become `Polygon.holes`)
- Clear user-facing warnings for skipped solids (non-planar face, extraction failure)

### P2 (nice-to-have)
- Progress reporting for STEP import/extraction (counts/phase text)
- Better face selection heuristic (e.g., prefer faces with normals aligned to the inferred thickness axis)
- Output directory chooser + configurable naming scheme in UI
- Optional “export only selected sheet” feature

## Risks

- **FFI + memory safety:** OCCT is C++ with complex ownership. `opencascade-sys` may expose raw pointers/handles that require strict RAII wrappers to prevent leaks/UAF.
- **Linking on macOS ARM64:** locating headers/libs, runtime `.dylib` search paths, and CI reproducibility may be non-trivial.
- **Thickness detection correctness:** global-axis AABB thickness fails for rotated parts or non-plate-like solids. Might mis-bucket.
- **Face selection heuristic:** “largest planar face” can be wrong for some parts (e.g., flanges, complex shapes).
- **Curve discretization tradeoff:** too coarse breaks nesting accuracy; too fine slows optimizer/collision.
- **Mixed-source sessions:** exporting when pieces originate from JSON + STEP requires a clear policy to avoid surprising output.

## Security

- No network behavior introduced.
- File I/O is local and user-initiated via OS dialogs (`rfd`).
- Primary security concern is `unsafe` FFI:
  - Centralize `unsafe` in `thetta-step`
  - Wrap OCCT handles in minimal safe abstractions
  - Validate pointers and handle null/failed-return codes defensively
- Avoid path injection:
  - Use paths from file dialogs
  - Validate output directory exists and is writable; sanitize `file_stem`

## Dependencies

Rust crates (expected additions):
- `opencascade-sys` (preferred binding layer)
- `rfd` (native dialogs)
- Potentially: `thiserror` (clean errors), `anyhow` (internal context), `slotmap`/`indexmap` (stable keys), `once_cell` (lazy init), `approx` (test tolerances)

System dependencies (macOS ARM64):
- OpenCASCADE (OCCT) installed locally (prefer Homebrew)
- Xcode Command Line Tools (C++ toolchain)
- Any environment configuration required by `opencascade-sys` to locate OCCT (exact mechanism is confirmed during Phase 0)

## Open Questions

1. `opencascade-sys` integration details:
   - How does it locate OCCT (env vars, `pkg-config`, hardcoded paths)?
   - Does it expose the specific APIs we need (STEP read/write, topology explorers, curve discretization)?
2. OCCT version pinning:
   - Which OCCT version is supported by `opencascade-sys` on macOS ARM64?
3. Piece ID strategy:
   - Current app assumes `PieceId` uniqueness but does not enforce it across multiple loads.
   - Should Sprint 003 introduce centralized ID allocation in `AppState` to guarantee uniqueness (recommended for export mapping)?
4. Thickness detection robustness:
   - Do we accept AABB-min dimension for Sprint 003, or must we do oriented thickness detection?
5. Coordinate conventions:
   - Are we assuming STEP units are mm? How do we handle unit scaling if not?
6. Export policy with mixed sources:
   - If JSON pieces exist, do we exclude them from STEP export with a warning, or hard-error?
7. Multi-sheet:
   - The current nesting engine returns a single sheet in practice; should Sprint 003 require multi-sheet nesting before export, or simply support export of all sheets that exist?
8. Error surfacing:
   - Should `thetta-step` return structured warnings (skipped solids) alongside success, for UI display?



If you want, I can also draft the minimal `AppState` changes (data structures + event flow) that make “switch thickness without re-reading STEP” work cleanly with the existing optimizer thread model.
