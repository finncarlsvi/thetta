# Sprint 003: STEP File Import & Export Pipeline

## Overview

Implement the full STEP file pipeline: load STEP files via OpenCASCADE FFI (`opencascade-sys`), detect thickness groups, extract 2D profiles from sheet-metal solids, feed them into the existing nesting engine, and export one STEP file per sheet with solids repositioned to their nested locations. This sprint also adds native file dialogs (`rfd`) for STEP load/save operations and extends the GUI with a thickness selector panel.

**Scope:** OpenCASCADE system setup, STEP import (solid iteration, thickness detection, face extraction, wire discretization), STEP export (solid repositioning, STEP writer), GUI integration (file dialogs, thickness selector), round-trip testing.
**Prerequisites:** Sprint 002 (jagua-rs integration + benchmarks) must be complete. The nesting engine accepts `Vec<Piece>` and produces `NestingResult` with `Vec<Placement>` -- this sprint produces pieces from STEP and consumes placements back into STEP.
**Deferred:** Multi-file STEP loading, advanced face selection heuristics beyond largest-face, STEP assembly-level structure preservation.

## Use Cases

### UC1: Load STEP File via Native File Dialog
- User clicks "Load STEP" button in the side panel
- Native macOS file dialog opens, filtered to `*.step` and `*.stp` extensions
- App reads the file on a background thread with a progress spinner
- On success, thickness groups are displayed in the side panel
- On error, a toast/label shows the error message (e.g., "No solids found", "OpenCASCADE parse error")

### UC2: Select Thickness and View Profiles
- After loading, the thickness selector shows all detected thicknesses sorted by solid count (descending)
- Each entry shows: thickness value (mm), number of solids, and a radio button
- The most common thickness is auto-selected
- Selecting a thickness extracts 2D profiles for all solids of that thickness
- Profiles appear immediately in the viewport
- User can switch thicknesses at any time; profiles update accordingly

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
- Progress indicator during export (may be slow for large files)
- On completion, a status label shows "Exported N sheets"

### UC5: Round-Trip Verification
- User can reimport an exported STEP file
- The reimported profiles should match the nested layout (validates export correctness)
- This is primarily a developer/testing use case, not a dedicated UI feature

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
|   +-- opencascade-sys      <-- NEW FFI bindings
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
  |  TopExp_Explorer(FACE)  |   Find largest face by area
  |  BRepGProp::Surface     |   (BRepGProp::SurfaceProperties)
  |  BRepTools::OuterWire   |   Extract outer wire
  |  BRepMesh_Incremental   |   Discretize edges -> polyline
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
/// Opaque handle to an OCCT solid, stored alongside extracted pieces
/// for later use during STEP export.
pub struct SolidHandle {
    /// Index into the internal solid storage vec
    index: usize,
}

/// Result of importing a STEP file -- returned to the app layer.
pub struct StepImportResult {
    pub thicknesses: Vec<ThicknessGroup>,
    pub selected_thickness: f64,
    pub pieces: Vec<Piece>,
    /// Opaque session holding OCCT data for export.
    /// Must be kept alive as long as export is possible.
    pub session: StepSession,
}

/// A group of solids sharing the same thickness.
pub struct ThicknessGroup {
    pub thickness_mm: f64,
    pub solid_count: usize,
}

/// Holds OCCT state between import and export.
/// Wraps the original TopoDS_Shape and per-solid handles.
pub struct StepSession {
    root_shape: OcctShape,                     // opencascade-sys wrapper
    solids: Vec<OcctSolid>,                    // all solids from the file
    thickness_map: HashMap<i64, Vec<usize>>,   // bucket -> solid indices
    solid_to_piece: HashMap<usize, PieceId>,   // solid index -> piece ID
}

/// Configuration for profile extraction.
pub struct ExtractionConfig {
    /// Tolerance for thickness grouping (default: 0.1mm)
    pub thickness_tolerance: f64,
    /// Chord tolerance for edge discretization (default: 0.5mm)
    pub mesh_tolerance: f64,
    /// Maximum number of points per edge (default: 200)
    pub max_points_per_edge: usize,
}
```

### New Types in thetta-geometry

```rust
/// Optional metadata linking a Piece back to its STEP origin.
/// Added to the existing Piece struct as an optional field.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StepOrigin {
    pub solid_index: usize,
    pub thickness_mm: f64,
    pub face_area_mm2: f64,
}
```

### thetta-app State Changes

```rust
/// Added to AppState
pub struct AppState {
    // ... existing fields ...

    /// Active STEP session for import/export round-trip
    pub step_session: Option<StepSession>,
    /// All detected thickness groups from the loaded STEP file
    pub thickness_groups: Vec<ThicknessGroup>,
    /// Currently selected thickness index
    pub selected_thickness_idx: usize,
    /// Whether a STEP import is in progress (background thread)
    pub step_loading: bool,
    /// Error message from last STEP operation
    pub step_error: Option<String>,
}
```

## Implementation

### Phase 0: Environment Setup

#### Task 0.1: Install OpenCASCADE on macOS ARM64
OpenCASCADE must be installed as a system library before `opencascade-sys` can build.

```sh
brew install opencascade
```

This installs OCCT 7.7+ to `/opt/homebrew/Cellar/opencascade/`. The `opencascade-sys` crate uses `pkg-config` or CMake to find it. Verify:

```sh
brew --prefix opencascade
# /opt/homebrew/opt/opencascade
ls /opt/homebrew/opt/opencascade/lib/libTKSTEP*
# Should list libTKSTEP.dylib, libTKSTEPBase.dylib, etc.
```

Environment variables may be needed:

```sh
export OCCT_ROOT=/opt/homebrew/opt/opencascade
export CMAKE_PREFIX_PATH=/opt/homebrew/opt/opencascade
```

#### Task 0.2: Add opencascade-sys dependency
**File:** `crates/thetta-step/Cargo.toml`

```toml
[package]
name = "thetta-step"
version.workspace = true
edition.workspace = true

[dependencies]
thetta-geometry = { path = "../thetta-geometry" }
opencascade-sys = "0.2"

[build-dependencies]
cc = "1"
```

Note: `opencascade-sys` provides raw FFI bindings generated via `bindgen`. If `opencascade-sys` on crates.io does not cover the APIs we need, we may need to vendor OCCT headers and write our own `build.rs` with `bindgen`. This is flagged as Risk R1.

#### Task 0.3: Verify build
```sh
cd crates/thetta-step && cargo check
```

If `opencascade-sys` fails to find OCCT, add a `build.rs` that emits:
```rust
println!("cargo:rustc-link-search=native=/opt/homebrew/opt/opencascade/lib");
println!("cargo:rustc-link-lib=dylib=TKernel");
println!("cargo:rustc-link-lib=dylib=TKMath");
println!("cargo:rustc-link-lib=dylib=TKBRep");
println!("cargo:rustc-link-lib=dylib=TKTopAlgo");
println!("cargo:rustc-link-lib=dylib=TKSTEP");
println!("cargo:rustc-link-lib=dylib=TKSTEPBase");
println!("cargo:rustc-link-lib=dylib=TKSTEPAttr");
println!("cargo:rustc-link-lib=dylib=TKSTEP209");
println!("cargo:rustc-link-lib=dylib=TKXSBase");
println!("cargo:rustc-link-lib=dylib=TKG3d");
println!("cargo:rustc-link-lib=dylib=TKMesh");
println!("cargo:rustc-link-lib=dylib=TKShHealing");
```

#### Task 0.4: Create test STEP files
**Directory:** `test_data/step/`

Create minimal test STEP files programmatically or source them:

| File | Contents | Purpose |
|------|----------|---------|
| `single_box.step` | One 100x50x3mm box | Simplest case: one solid, one thickness |
| `two_boxes_same.step` | Two 100x50x3mm boxes | Multiple solids, same thickness |
| `mixed_thickness.step` | 3mm box + 5mm box | Thickness grouping verification |
| `box_with_hole.step` | Box with a cylindrical through-hole | Inner wire / hole extraction |
| `l_shape.step` | L-shaped extrusion, 3mm thick | Non-rectangular profile |

These can be generated using FreeCAD's Python API or OpenCASCADE directly via a one-off Rust binary:

```rust
// test_data/generate_step.rs (one-off script, not part of workspace)
// Uses opencascade-sys to create and write simple solids
```

Alternatively, source simple STEP files from open CAD model repositories.

### Phase 1: STEP Import -- Solid Iteration and Thickness Detection

#### Task 1.1: OCCT FFI wrapper module
**File:** `crates/thetta-step/src/occt.rs` (NEW)

Thin safe wrappers around `opencascade-sys` raw FFI. All `unsafe` blocks are confined to this module.

```rust
use opencascade_sys::ffi;
use std::ffi::CString;
use std::path::Path;

/// Safe wrapper around STEPControl_Reader.
pub struct StepReader {
    inner: cxx::UniquePtr<ffi::STEPControl_Reader>,
}

impl StepReader {
    pub fn new() -> Self {
        Self {
            inner: unsafe { ffi::STEPControl_Reader_new() },
        }
    }

    /// Read a STEP file. Returns the number of roots transferred.
    pub fn read_file(&mut self, path: &Path) -> Result<i32, StepImportError> {
        let c_path = CString::new(
            path.to_str().ok_or(StepImportError::ParseError("Invalid path".into()))?
        ).map_err(|e| StepImportError::ParseError(e.to_string()))?;

        let status = unsafe {
            ffi::STEPControl_Reader_ReadFile(
                self.inner.pin_mut(),
                c_path.as_ptr(),
            )
        };

        // IFSelect_ReturnStatus: 0 = RetDone (success)
        if status != 0 {
            return Err(StepImportError::ParseError(
                format!("ReadFile returned status {}", status)
            ));
        }

        let roots = unsafe {
            ffi::STEPControl_Reader_TransferRoots(self.inner.pin_mut())
        };
        Ok(roots)
    }

    /// Get the resulting shape after transfer.
    pub fn one_shape(&self) -> OcctShape {
        let shape = unsafe {
            ffi::STEPControl_Reader_OneShape(self.inner.as_ref().unwrap())
        };
        OcctShape { inner: shape }
    }
}

/// Safe wrapper around TopoDS_Shape.
pub struct OcctShape {
    inner: cxx::UniquePtr<ffi::TopoDS_Shape>,
}

/// Safe wrapper around TopoDS_Solid.
pub struct OcctSolid {
    inner: cxx::UniquePtr<ffi::TopoDS_Solid>,
}

/// Iterate all solids in a shape using TopExp_Explorer.
pub fn explore_solids(shape: &OcctShape) -> Vec<OcctSolid> {
    let mut solids = Vec::new();
    unsafe {
        let mut explorer = ffi::TopExp_Explorer_new(
            shape.inner.as_ref().unwrap(),
            ffi::TopAbs_ShapeEnum::TopAbs_SOLID,
        );
        while ffi::TopExp_Explorer_More(&explorer) {
            let current = ffi::TopExp_Explorer_Current(&explorer);
            let solid = ffi::TopoDS_Solid_downcast(&current);
            solids.push(OcctSolid {
                inner: ffi::clone_shape(&solid),
            });
            ffi::TopExp_Explorer_Next(explorer.pin_mut());
        }
    }
    solids
}

/// Compute the bounding box of a solid. Returns (x_size, y_size, z_size).
pub fn bounding_box_dimensions(solid: &OcctSolid) -> (f64, f64, f64) {
    unsafe {
        let mut bbox = ffi::Bnd_Box_new();
        ffi::BRepBndLib_Add(
            solid.inner.as_ref().unwrap(),
            bbox.pin_mut(),
            false, // use triangulation = false, use exact geometry
        );
        let mut x_min = 0.0_f64;
        let mut y_min = 0.0_f64;
        let mut z_min = 0.0_f64;
        let mut x_max = 0.0_f64;
        let mut y_max = 0.0_f64;
        let mut z_max = 0.0_f64;
        ffi::Bnd_Box_Get(
            &bbox,
            &mut x_min, &mut y_min, &mut z_min,
            &mut x_max, &mut y_max, &mut z_max,
        );
        (x_max - x_min, y_max - y_min, z_max - z_min)
    }
}

/// Determine thickness = smallest bounding box dimension.
pub fn solid_thickness(solid: &OcctSolid) -> f64 {
    let (dx, dy, dz) = bounding_box_dimensions(solid);
    dx.min(dy).min(dz)
}
```

#### Task 1.2: Thickness detection and grouping
**File:** `crates/thetta-step/src/import.rs` (REWRITE)

```rust
use crate::occt::{self, OcctSolid, StepReader};
use crate::ExtractionConfig;
use std::collections::HashMap;
use std::path::Path;
use thetta_geometry::types::{Piece, PieceId};

pub fn import_step(
    path: &Path,
    thickness: Option<f64>,
    config: &ExtractionConfig,
) -> Result<StepImportResult, StepImportError> {
    // 1. Read STEP file
    let mut reader = StepReader::new();
    reader.read_file(path)?;
    let root_shape = reader.one_shape();

    // 2. Iterate all solids
    let solids = occt::explore_solids(&root_shape);
    if solids.is_empty() {
        return Err(StepImportError::NoSolidsFound);
    }

    // 3. Compute thickness per solid and group
    let mut thickness_map: HashMap<i64, Vec<usize>> = HashMap::new();
    let mut solid_thicknesses: Vec<f64> = Vec::with_capacity(solids.len());
    for (i, solid) in solids.iter().enumerate() {
        let t = occt::solid_thickness(solid);
        solid_thicknesses.push(t);
        let bucket = (t / config.thickness_tolerance).round() as i64;
        thickness_map.entry(bucket).or_default().push(i);
    }

    // 4. Build thickness groups sorted by count (descending)
    let mut groups: Vec<ThicknessGroup> = thickness_map
        .iter()
        .map(|(&bucket, indices)| ThicknessGroup {
            thickness_mm: bucket as f64 * config.thickness_tolerance,
            solid_count: indices.len(),
        })
        .collect();
    groups.sort_by(|a, b| b.solid_count.cmp(&a.solid_count));

    // 5. Select thickness (user-specified or most common)
    let selected = thickness.unwrap_or(groups[0].thickness_mm);
    let selected_bucket = (selected / config.thickness_tolerance).round() as i64;
    let selected_indices = thickness_map
        .get(&selected_bucket)
        .cloned()
        .unwrap_or_default();

    // 6. Extract 2D profiles for selected thickness
    let pieces = extract_profiles(&solids, &selected_indices, config)?;

    // 7. Build solid-to-piece mapping for export
    let mut solid_to_piece = HashMap::new();
    for (piece_idx, &solid_idx) in selected_indices.iter().enumerate() {
        solid_to_piece.insert(solid_idx, pieces[piece_idx].id);
    }

    Ok(StepImportResult {
        thicknesses: groups,
        selected_thickness: selected,
        pieces,
        session: StepSession {
            root_shape,
            solids,
            thickness_map,
            solid_to_piece,
        },
    })
}
```

#### Task 1.3: Profile extraction (face selection + wire discretization)
**File:** `crates/thetta-step/src/profile.rs` (NEW)

```rust
use crate::occt::{self, OcctSolid};
use crate::ExtractionConfig;
use thetta_geometry::types::{Piece, PieceId, Point2D, Polygon};

/// Extract a 2D profile from a solid by finding its largest planar face,
/// extracting the outer wire, and discretizing it into a polyline.
pub fn extract_profile(
    solid: &OcctSolid,
    piece_id: PieceId,
    config: &ExtractionConfig,
) -> Result<Piece, ProfileError> {
    // 1. Find the largest face
    let face = find_largest_face(solid)?;

    // 2. Check that the face is planar
    if !occt::is_planar_face(&face) {
        return Err(ProfileError::NonPlanarFace);
    }

    // 3. Get the face normal to determine the projection plane
    let normal = occt::face_normal(&face);

    // 4. Extract outer wire
    let outer_wire = occt::outer_wire(&face)?;

    // 5. Discretize outer wire into polyline
    let outer_points = discretize_wire(&outer_wire, config)?;

    // 6. Extract inner wires (holes)
    let inner_wires = occt::inner_wires(&face)?;
    let holes: Vec<Vec<Point2D>> = inner_wires
        .iter()
        .map(|w| discretize_wire(w, config))
        .collect::<Result<Vec<_>, _>>()?;

    // 7. Project 3D points to 2D using the face normal
    let outer_2d = project_to_2d(&outer_points, &normal);
    let holes_2d: Vec<Vec<Point2D>> = holes
        .iter()
        .map(|h| project_to_2d(h, &normal))
        .collect();

    // 8. Ensure correct winding order (CCW for outer, CW for holes)
    let outer_2d = ensure_ccw(outer_2d);
    let holes_2d: Vec<Vec<Point2D>> = holes_2d
        .into_iter()
        .map(ensure_cw)
        .collect();

    Ok(Piece {
        id: piece_id,
        polygon: Polygon {
            outer: outer_2d,
            holes: holes_2d,
        },
        source_file: Default::default(), // set by caller
        quantity: 1,
    })
}

/// Find the face with the largest surface area using BRepGProp.
fn find_largest_face(solid: &OcctSolid) -> Result<OcctFace, ProfileError> {
    let faces = occt::explore_faces(solid);
    if faces.is_empty() {
        return Err(ProfileError::NoFaces);
    }

    let mut best_face = None;
    let mut best_area = 0.0_f64;

    for face in &faces {
        // BRepGProp::SurfaceProperties computes the area
        let area = unsafe {
            let mut props = ffi::GProp_GProps_new();
            ffi::BRepGProp_SurfaceProperties(
                face.inner.as_ref().unwrap(),
                props.pin_mut(),
                1.0e-6, // tolerance
            );
            ffi::GProp_GProps_Mass(&props)
        };
        if area > best_area {
            best_area = area;
            best_face = Some(face.clone());
        }
    }

    best_face.ok_or(ProfileError::NoFaces)
}

/// Discretize a wire into a polyline using BRepMesh_IncrementalMesh.
///
/// Iterates edges in the wire, meshes each edge, and collects
/// the resulting polyline points.
fn discretize_wire(
    wire: &OcctWire,
    config: &ExtractionConfig,
) -> Result<Vec<Point3D>, ProfileError> {
    let mut points = Vec::new();
    let edges = occt::explore_edges(wire);

    for edge in &edges {
        // Use BRepAdaptor_Curve + GCPnts_UniformAbscissa for uniform spacing
        unsafe {
            let adaptor = ffi::BRepAdaptor_Curve_new(edge.inner.as_ref().unwrap());
            let length = ffi::GCPnts_AbscissaPoint_Length(
                adaptor.as_ref().unwrap(),
                1.0e-6,
            );

            // Number of points based on chord tolerance
            let n_points = ((length / config.mesh_tolerance) as usize)
                .clamp(2, config.max_points_per_edge);

            let mut uniform = ffi::GCPnts_UniformAbscissa_new(
                adaptor.as_ref().unwrap(),
                n_points as i32,
            );

            for i in 1..=ffi::GCPnts_UniformAbscissa_NbPoints(&uniform) {
                let param = ffi::GCPnts_UniformAbscissa_Parameter(&uniform, i);
                let pnt = ffi::BRepAdaptor_Curve_Value(
                    adaptor.as_ref().unwrap(),
                    param,
                );
                let x = ffi::gp_Pnt_X(&pnt);
                let y = ffi::gp_Pnt_Y(&pnt);
                let z = ffi::gp_Pnt_Z(&pnt);
                points.push(Point3D { x, y, z });
            }
        }
    }

    // Remove duplicate consecutive points (edge joins)
    points.dedup_by(|a, b| {
        (a.x - b.x).abs() < 1e-9
            && (a.y - b.y).abs() < 1e-9
            && (a.z - b.z).abs() < 1e-9
    });

    Ok(points)
}

/// Project 3D points onto a 2D plane based on the face normal.
/// Uses the two axes perpendicular to the normal as the 2D coordinate system.
fn project_to_2d(points: &[Point3D], normal: &Vec3D) -> Vec<Point2D> {
    // Determine the thickness axis (the one most aligned with the normal)
    let abs_normal = (normal.x.abs(), normal.y.abs(), normal.z.abs());
    let (u_axis, v_axis) = if abs_normal.2 >= abs_normal.0 && abs_normal.2 >= abs_normal.1 {
        // Normal is ~Z: project onto XY
        (Vec3D { x: 1.0, y: 0.0, z: 0.0 }, Vec3D { x: 0.0, y: 1.0, z: 0.0 })
    } else if abs_normal.1 >= abs_normal.0 {
        // Normal is ~Y: project onto XZ
        (Vec3D { x: 1.0, y: 0.0, z: 0.0 }, Vec3D { x: 0.0, y: 0.0, z: 1.0 })
    } else {
        // Normal is ~X: project onto YZ
        (Vec3D { x: 0.0, y: 1.0, z: 0.0 }, Vec3D { x: 0.0, y: 0.0, z: 1.0 })
    };

    points
        .iter()
        .map(|p| Point2D {
            x: p.x * u_axis.x + p.y * u_axis.y + p.z * u_axis.z,
            y: p.x * v_axis.x + p.y * v_axis.y + p.z * v_axis.z,
        })
        .collect()
}

/// Ensure polygon winding is counter-clockwise using the shoelace formula.
fn ensure_ccw(mut points: Vec<Point2D>) -> Vec<Point2D> {
    let area: f64 = points
        .windows(2)
        .map(|w| w[0].x * w[1].y - w[1].x * w[0].y)
        .sum();
    if area < 0.0 {
        points.reverse();
    }
    points
}

fn ensure_cw(mut points: Vec<Point2D>) -> Vec<Point2D> {
    let area: f64 = points
        .windows(2)
        .map(|w| w[0].x * w[1].y - w[1].x * w[0].y)
        .sum();
    if area > 0.0 {
        points.reverse();
    }
    points
}

#[derive(Debug)]
pub enum ProfileError {
    NoFaces,
    NonPlanarFace,
    DiscretizationFailed(String),
}
```

#### Task 1.4: Extract profiles for selected thickness
**File:** `crates/thetta-step/src/import.rs` (continued)

```rust
fn extract_profiles(
    solids: &[OcctSolid],
    indices: &[usize],
    config: &ExtractionConfig,
) -> Result<Vec<Piece>, StepImportError> {
    let mut pieces = Vec::with_capacity(indices.len());
    for (i, &solid_idx) in indices.iter().enumerate() {
        let piece_id = PieceId(solid_idx as u64);
        match crate::profile::extract_profile(&solids[solid_idx], piece_id, config) {
            Ok(piece) => pieces.push(piece),
            Err(ProfileError::NonPlanarFace) => {
                // Skip non-planar solids with a warning
                eprintln!(
                    "Warning: solid {} has non-planar largest face, skipping",
                    solid_idx
                );
            }
            Err(e) => {
                return Err(StepImportError::ParseError(format!(
                    "Failed to extract profile for solid {}: {:?}",
                    solid_idx, e
                )));
            }
        }
    }
    Ok(pieces)
}
```

### Phase 2: STEP Export

#### Task 2.1: Solid repositioning
**File:** `crates/thetta-step/src/export.rs` (REWRITE)

```rust
use crate::occt::{self, OcctSolid, StepSession};
use std::path::Path;
use thetta_geometry::types::{Placement, Sheet};

pub fn export_sheet(
    session: &StepSession,
    sheet: &Sheet,
    output_path: &Path,
) -> Result<(), StepExportError> {
    // 1. Create a STEP writer
    let mut writer = occt::StepWriter::new();

    // 2. For each placement, retrieve and transform the original solid
    for placement in &sheet.placements {
        // Find the solid index for this piece
        let solid_idx = session
            .piece_to_solid(placement.piece_id)
            .ok_or_else(|| {
                StepExportError::WriteError(format!(
                    "No solid found for piece {:?}",
                    placement.piece_id
                ))
            })?;

        let solid = &session.solids[solid_idx];

        // Build a 3D transformation from the 2D placement:
        //   - Rotate around Z axis by placement.rotation
        //   - Translate by (placement.x, placement.y, 0)
        let transformed = occt::transform_solid(
            solid,
            placement.x,
            placement.y,
            placement.rotation,
        )?;

        // Transfer the transformed solid to the STEP writer
        writer.add_solid(&transformed)?;
    }

    // 3. Write the STEP file
    writer.write(output_path)?;

    Ok(())
}

/// Export all sheets, one STEP file per sheet.
pub fn export_all_sheets(
    session: &StepSession,
    sheets: &[Sheet],
    base_path: &Path,
    base_name: &str,
) -> Result<Vec<PathBuf>, StepExportError> {
    let mut output_paths = Vec::new();
    for (i, sheet) in sheets.iter().enumerate() {
        let filename = format!("{}_sheet_{}.step", base_name, i + 1);
        let path = base_path.join(&filename);
        export_sheet(session, sheet, &path)?;
        output_paths.push(path);
    }
    Ok(output_paths)
}
```

#### Task 2.2: OCCT transform and writer wrappers
**File:** `crates/thetta-step/src/occt.rs` (EXTEND)

```rust
/// Apply a 2D transformation (rotation around Z + XY translation) to a solid.
pub fn transform_solid(
    solid: &OcctSolid,
    tx: f64,
    ty: f64,
    rotation_rad: f64,
) -> Result<OcctSolid, StepExportError> {
    unsafe {
        // Create gp_Trsf for rotation around Z axis at origin
        let mut trsf = ffi::gp_Trsf_new();

        // Set rotation: axis = gp::OZ(), angle = rotation_rad
        let oz = ffi::gp_OZ();
        ffi::gp_Trsf_SetRotation(trsf.pin_mut(), &oz, rotation_rad);

        // Compose with translation
        let mut trsf_translate = ffi::gp_Trsf_new();
        let translation_vec = ffi::gp_Vec_new(tx, ty, 0.0);
        ffi::gp_Trsf_SetTranslation(trsf_translate.pin_mut(), &translation_vec);

        // Multiply: result = translate * rotate
        ffi::gp_Trsf_Multiply(trsf.pin_mut(), &trsf_translate);

        // Apply via BRepBuilderAPI_Transform
        let mut transform_builder = ffi::BRepBuilderAPI_Transform_new(
            solid.inner.as_ref().unwrap(),
            &trsf,
            true, // copy = true (don't modify original)
        );

        let result_shape = ffi::BRepBuilderAPI_Transform_Shape(&transform_builder);
        let result_solid = ffi::TopoDS_Solid_downcast(&result_shape);
        Ok(OcctSolid {
            inner: ffi::clone_shape(&result_solid),
        })
    }
}

/// Safe wrapper around STEPControl_Writer.
pub struct StepWriter {
    inner: cxx::UniquePtr<ffi::STEPControl_Writer>,
}

impl StepWriter {
    pub fn new() -> Self {
        Self {
            inner: unsafe { ffi::STEPControl_Writer_new() },
        }
    }

    pub fn add_solid(&mut self, solid: &OcctSolid) -> Result<(), StepExportError> {
        let status = unsafe {
            ffi::STEPControl_Writer_Transfer(
                self.inner.pin_mut(),
                solid.inner.as_ref().unwrap(),
                ffi::STEPControl_StepModelType::STEPControl_AsIs,
                true, // compgraph
            )
        };
        // IFSelect_ReturnStatus::IFSelect_RetDone = 0
        if status != 0 {
            return Err(StepExportError::WriteError(
                format!("Transfer failed with status {}", status)
            ));
        }
        Ok(())
    }

    pub fn write(&mut self, path: &Path) -> Result<(), StepExportError> {
        let c_path = CString::new(
            path.to_str().ok_or(StepExportError::WriteError("Invalid path".into()))?
        ).map_err(|e| StepExportError::WriteError(e.to_string()))?;

        let status = unsafe {
            ffi::STEPControl_Writer_Write(
                self.inner.pin_mut(),
                c_path.as_ptr(),
            )
        };
        if status != 0 {
            return Err(StepExportError::WriteError(
                format!("Write failed with status {}", status)
            ));
        }
        Ok(())
    }
}
```

### Phase 3: GUI Integration

#### Task 3.1: Add rfd dependency
**File:** `crates/thetta-app/Cargo.toml` (MODIFY)

```toml
[dependencies]
# ... existing deps ...
rfd = "0.14"
```

`rfd` is likely already a dependency from Sprint 001 (used for JSON file dialogs). If so, no change needed; just confirm it is present.

#### Task 3.2: Thickness selector panel
**File:** `crates/thetta-app/src/panels.rs` (MODIFY)

Add a "STEP" section to the side panel:

```rust
fn step_panel(ui: &mut egui::Ui, state: &mut AppState) {
    ui.heading("STEP Import");

    // Load button
    if ui.button("Load STEP File...").clicked() {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("STEP Files", &["step", "stp"])
            .pick_file()
        {
            state.start_step_import(path);
        }
    }

    // Loading spinner
    if state.step_loading {
        ui.spinner();
        ui.label("Loading STEP file...");
        return;
    }

    // Error display
    if let Some(ref err) = state.step_error {
        ui.colored_label(egui::Color32::RED, err);
    }

    // Thickness selector
    if !state.thickness_groups.is_empty() {
        ui.separator();
        ui.label("Thickness Groups:");

        for (i, group) in state.thickness_groups.iter().enumerate() {
            let label = format!(
                "{:.1} mm  ({} solid{})",
                group.thickness_mm,
                group.solid_count,
                if group.solid_count == 1 { "" } else { "s" },
            );
            if ui.radio(state.selected_thickness_idx == i, &label).clicked() {
                state.select_thickness(i);
            }
        }
    }

    // Export button (only if nesting result exists and session is active)
    if state.step_session.is_some() && state.current_result.is_some() {
        ui.separator();
        if ui.button("Export STEP...").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("STEP Files", &["step"])
                .set_file_name("nested")
                .save_file()
            {
                state.start_step_export(path);
            }
        }
    }
}
```

#### Task 3.3: Background STEP import threading
**File:** `crates/thetta-app/src/state.rs` (MODIFY)

```rust
impl AppState {
    pub fn start_step_import(&mut self, path: PathBuf) {
        self.step_loading = true;
        self.step_error = None;

        let config = ExtractionConfig::default();
        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let result = thetta_step::import::import_step(
                &path,
                None, // auto-detect thickness
                &config,
            );
            let _ = tx.send(result);
        });

        self.step_import_rx = Some(rx);
    }

    /// Called each frame to poll for STEP import completion.
    pub fn poll_step_import(&mut self) {
        if let Some(ref rx) = self.step_import_rx {
            if let Ok(result) = rx.try_recv() {
                self.step_loading = false;
                match result {
                    Ok(import_result) => {
                        self.thickness_groups = import_result.thicknesses;
                        self.selected_thickness_idx = 0;
                        self.pieces = import_result.pieces;
                        self.step_session = Some(import_result.session);
                        self.step_error = None;
                        // Trigger nesting with extracted pieces
                        self.start_nesting();
                    }
                    Err(e) => {
                        self.step_error = Some(format!("{:?}", e));
                    }
                }
                self.step_import_rx = None;
            }
        }
    }

    pub fn select_thickness(&mut self, idx: usize) {
        if idx == self.selected_thickness_idx {
            return;
        }
        self.selected_thickness_idx = idx;
        let thickness = self.thickness_groups[idx].thickness_mm;

        // Re-extract profiles for the new thickness
        if let Some(ref session) = self.step_session {
            let config = ExtractionConfig::default();
            match session.extract_for_thickness(thickness, &config) {
                Ok(pieces) => {
                    self.pieces = pieces;
                    self.start_nesting();
                }
                Err(e) => {
                    self.step_error = Some(format!("{:?}", e));
                }
            }
        }
    }

    pub fn start_step_export(&mut self, path: PathBuf) {
        let session = self.step_session.as_ref().unwrap().clone();
        let result = self.current_result.as_ref().unwrap().clone();
        let base_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("nested")
            .to_string();
        let base_dir = path.parent().unwrap().to_path_buf();

        std::thread::spawn(move || {
            let _ = thetta_step::export::export_all_sheets(
                &session,
                &result.sheets,
                &base_dir,
                &base_name,
            );
        });
    }
}
```

#### Task 3.4: Wire up STEP panels in main app
**File:** `crates/thetta-app/src/main.rs` (MODIFY)

Add `state.poll_step_import()` to the egui update loop alongside the existing `poll_nesting_updates()`. Add the `step_panel()` call in the side panel layout.

### Phase 4: Testing

#### Task 4.1: Unit tests for thickness detection
**File:** `crates/thetta-step/tests/thickness_tests.rs` (NEW)

```rust
#[test]
fn test_single_box_thickness() {
    let result = import_step(
        Path::new("../../test_data/step/single_box.step"),
        None,
        &ExtractionConfig::default(),
    ).unwrap();
    assert_eq!(result.thicknesses.len(), 1);
    assert!((result.thicknesses[0].thickness_mm - 3.0).abs() < 0.2);
    assert_eq!(result.thicknesses[0].solid_count, 1);
}

#[test]
fn test_mixed_thickness_grouping() {
    let result = import_step(
        Path::new("../../test_data/step/mixed_thickness.step"),
        None,
        &ExtractionConfig::default(),
    ).unwrap();
    assert_eq!(result.thicknesses.len(), 2);
    // Most common thickness should be selected
    assert_eq!(result.selected_thickness, result.thicknesses[0].thickness_mm);
}

#[test]
fn test_select_specific_thickness() {
    let result = import_step(
        Path::new("../../test_data/step/mixed_thickness.step"),
        Some(5.0),
        &ExtractionConfig::default(),
    ).unwrap();
    assert!((result.selected_thickness - 5.0).abs() < 0.2);
}
```

#### Task 4.2: Unit tests for profile extraction
**File:** `crates/thetta-step/tests/profile_tests.rs` (NEW)

```rust
#[test]
fn test_box_profile_is_rectangle() {
    let result = import_step(
        Path::new("../../test_data/step/single_box.step"),
        None,
        &ExtractionConfig::default(),
    ).unwrap();
    assert_eq!(result.pieces.len(), 1);
    let poly = &result.pieces[0].polygon;
    // A box should produce a rectangular profile
    assert!(poly.outer.len() >= 4);
    assert!(poly.holes.is_empty());
    // Approximate area should be ~100*50 = 5000 mm^2
    let area = polygon_area(&poly.outer);
    assert!((area - 5000.0).abs() < 100.0);
}

#[test]
fn test_box_with_hole_has_inner_wire() {
    let result = import_step(
        Path::new("../../test_data/step/box_with_hole.step"),
        None,
        &ExtractionConfig::default(),
    ).unwrap();
    assert_eq!(result.pieces.len(), 1);
    assert_eq!(result.pieces[0].polygon.holes.len(), 1);
}

#[test]
fn test_l_shape_profile() {
    let result = import_step(
        Path::new("../../test_data/step/l_shape.step"),
        None,
        &ExtractionConfig::default(),
    ).unwrap();
    assert_eq!(result.pieces.len(), 1);
    // L-shape should have more than 4 outer vertices
    assert!(result.pieces[0].polygon.outer.len() >= 6);
}
```

#### Task 4.3: Integration test -- full round-trip
**File:** `crates/thetta-step/tests/roundtrip_test.rs` (NEW)

```rust
#[test]
fn test_import_nest_export_reimport() {
    // 1. Import
    let import_result = import_step(
        Path::new("../../test_data/step/two_boxes_same.step"),
        None,
        &ExtractionConfig::default(),
    ).unwrap();
    assert_eq!(import_result.pieces.len(), 2);

    // 2. Create a simple placement (no actual nesting, just translate)
    let sheet = Sheet {
        placements: vec![
            Placement { piece_id: import_result.pieces[0].id, x: 0.0, y: 0.0, rotation: 0.0 },
            Placement { piece_id: import_result.pieces[1].id, x: 120.0, y: 0.0, rotation: 0.0 },
        ],
        utilization: 0.0,
    };

    // 3. Export
    let tmp_dir = tempfile::tempdir().unwrap();
    let output_path = tmp_dir.path().join("roundtrip_sheet_1.step");
    export_sheet(&import_result.session, &sheet, &output_path).unwrap();
    assert!(output_path.exists());

    // 4. Reimport
    let reimport = import_step(&output_path, None, &ExtractionConfig::default()).unwrap();
    assert_eq!(reimport.pieces.len(), 2);

    // 5. Verify profiles approximately match
    for (orig, reimported) in import_result.pieces.iter().zip(reimport.pieces.iter()) {
        let orig_area = polygon_area(&orig.polygon.outer);
        let reimport_area = polygon_area(&reimported.polygon.outer);
        assert!(
            (orig_area - reimport_area).abs() / orig_area < 0.05,
            "Area mismatch: orig={}, reimported={}",
            orig_area,
            reimport_area,
        );
    }
}
```

#### Task 4.4: Build verification
```sh
# Must pass on macOS ARM64
cargo build --release -p thetta-step
cargo test -p thetta-step
cargo build --release -p thetta-app
```

## Files Summary

### New Files

| File | Purpose | Est. LOC |
|------|---------|----------|
| `crates/thetta-step/src/occt.rs` | Safe OCCT FFI wrappers (StepReader, StepWriter, explorers, transforms) | 350 |
| `crates/thetta-step/src/profile.rs` | Face selection, wire discretization, 2D projection | 250 |
| `crates/thetta-step/build.rs` | Link configuration for OCCT libraries (if needed) | 30 |
| `crates/thetta-step/tests/thickness_tests.rs` | Thickness detection unit tests | 60 |
| `crates/thetta-step/tests/profile_tests.rs` | Profile extraction unit tests | 80 |
| `crates/thetta-step/tests/roundtrip_test.rs` | Import-export round-trip integration test | 70 |
| `test_data/step/single_box.step` | Test file: one 100x50x3mm box | -- |
| `test_data/step/two_boxes_same.step` | Test file: two boxes, same thickness | -- |
| `test_data/step/mixed_thickness.step` | Test file: 3mm + 5mm boxes | -- |
| `test_data/step/box_with_hole.step` | Test file: box with cylindrical hole | -- |
| `test_data/step/l_shape.step` | Test file: L-shaped extrusion | -- |
| **Total new** | | **~840** |

### Modified Files

| File | Changes | Est. Delta |
|------|---------|------------|
| `crates/thetta-step/Cargo.toml` | Add `opencascade-sys`, `cc` build dep | +5 |
| `crates/thetta-step/src/lib.rs` | Add `pub mod occt; pub mod profile;`, re-export types | +10 |
| `crates/thetta-step/src/import.rs` | Full rewrite: `todo!()` -> OCCT import pipeline | ~200 (rewrite) |
| `crates/thetta-step/src/export.rs` | Full rewrite: `todo!()` -> OCCT export pipeline | ~100 (rewrite) |
| `crates/thetta-geometry/src/types.rs` | Add optional `StepOrigin` field to `Piece` | +15 |
| `crates/thetta-app/Cargo.toml` | Add `thetta-step` dependency (rfd likely already present) | +2 |
| `crates/thetta-app/src/state.rs` | Add STEP session state, import/export methods, polling | +80 |
| `crates/thetta-app/src/panels.rs` | Add STEP section (load, thickness selector, export) | +60 |
| `crates/thetta-app/src/main.rs` | Wire up STEP polling and panel | +15 |
| `Cargo.toml` | Workspace dep for `opencascade-sys` if needed | +3 |
| **Total modified** | | **~490 delta** |

### Estimated Total Effort: ~1330 LOC (new + delta)

## Definition of Done

### P0 -- Must Ship
- [ ] OpenCASCADE installed on macOS ARM64 and `cargo build --release -p thetta-step` succeeds
- [ ] `import_step()` reads a STEP file and returns a list of thickness groups with solid counts
- [ ] Thickness grouping correctly identifies distinct thicknesses (0.1mm tolerance)
- [ ] Profile extraction produces a `Vec<Piece>` with correct 2D polygons from sheet-metal solids
- [ ] Holes in solids are extracted as inner wires in the polygon
- [ ] Extracted pieces feed into the existing nesting engine (LBF + GLS) and produce a valid result
- [ ] `export_sheet()` writes a valid STEP file with solids at nested positions
- [ ] GUI "Load STEP" button opens a native file dialog and triggers import on a background thread
- [ ] Thickness selector panel shows all groups; selecting one re-extracts profiles
- [ ] GUI "Export STEP" button writes one `.step` file per sheet
- [ ] All test STEP files pass thickness detection and profile extraction tests
- [ ] No panics or crashes during import/export of test files

### P1 -- Should Ship
- [ ] Round-trip test passes: import -> nest -> export -> reimport -> areas match within 5%
- [ ] Non-planar faces are skipped gracefully with a warning (not a crash)
- [ ] Progress indication during STEP import (spinner in GUI)
- [ ] Export progress indication
- [ ] Error messages displayed in GUI for failed imports (file not found, parse error, no solids)
- [ ] Winding order (CCW outer, CW holes) verified and corrected automatically

### P2 -- Stretch
- [ ] Support loading multiple STEP files (pieces accumulate across files)
- [ ] Face selection heuristic improved: prefer face with normal aligned to thickness axis over pure largest-area
- [ ] Configurable mesh tolerance in GUI (advanced settings)
- [ ] STEP export preserves original solid colors/materials
- [ ] Test with real-world sheet-metal STEP files from production use cases

### Cut if Behind Schedule
- Multiple STEP file loading (P2)
- Face selection heuristic improvements (P2)
- Color/material preservation (P2)
- Configurable mesh tolerance UI (P2)

## Risks

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| R1: `opencascade-sys` crate lacks required API coverage (e.g., missing bindings for `GCPnts_UniformAbscissa`, `BRepGProp`, or `STEPControl_Writer`) | High | High | Evaluate `opencascade-sys` API coverage early in Phase 0. If insufficient, consider the higher-level `opencascade` crate (Rust wrapper) or write custom `bindgen` bindings against OCCT headers. Budget 2 days for this investigation. |
| R2: OpenCASCADE build/link failures on macOS ARM64 | High | Medium | Homebrew's `opencascade` formula supports ARM64. Verify early. If `pkg-config` fails, use explicit `cargo:rustc-link-search` and `cargo:rustc-link-lib` in `build.rs`. |
| R3: Largest-face heuristic selects wrong face for some solid geometries | Medium | Medium | For sheet-metal parts, the largest face is almost always correct. If not, fall back to "face with normal most aligned to the thickness axis" heuristic. Both heuristics are implementable; start with largest-face. |
| R4: Edge discretization produces too many or too few vertices | Medium | Low | Use `GCPnts_UniformAbscissa` with configurable chord tolerance (default 0.5mm). Clamp point count per edge to [2, 200]. Test with curved edges (hole boundaries). |
| R5: `StepSession` with OCCT data is not `Send`/`Sync` -- cannot move across threads | High | Medium | OCCT objects are C++ heap-allocated. Wrap in `Arc<Mutex<>>` if needed, or do all OCCT work on a dedicated STEP thread and communicate via channels. |
| R6: STEP export produces invalid files that other CAD tools cannot read | Medium | Medium | Validate exported files by reimporting them. Also test opening in FreeCAD. Use `STEPControl_AsIs` mode to preserve geometry fidelity. |
| R7: No test STEP files available -- must create them | Low | High | Generate test files using FreeCAD Python scripting (`Part.makeBox`, `Part.makeCylinder`, etc.) or a one-off Rust binary using OCCT. Budget half a day for test file creation. |
| R8: OCCT memory management -- double-free or use-after-free in unsafe FFI code | High | Medium | Confine all `unsafe` blocks to `occt.rs`. Use RAII wrappers (`Drop` impls) for all OCCT handles. Run under AddressSanitizer during development. |

## Security

- No network access at runtime. Fully local application.
- OpenCASCADE and `opencascade-sys` are build-time dependencies only (no runtime downloads).
- STEP files are parsed by OpenCASCADE's C++ parser, not by custom parsing code. OCCT is widely used in production CAD systems.
- File I/O via native OS dialog (`rfd`). No arbitrary path traversal.
- All FFI code is in `occt.rs` with explicit `unsafe` blocks. No `unsafe` in other modules.
- Export writes only to user-specified paths via save dialog.
- No user-provided code, scripts, or macros.
- Risk: malformed STEP files could trigger bugs in OCCT's C++ parser. Mitigation: OCCT is battle-tested; we propagate errors rather than panicking.

## Dependencies

### New Rust Crates

| Crate | Version | Purpose |
|-------|---------|---------|
| `opencascade-sys` | `0.2` | Low-level FFI bindings to OpenCASCADE C++ libraries |
| `cc` | `1` | Build dependency for C++ compilation if custom bindings are needed |
| `tempfile` | `3` | Test dependency for round-trip test (temporary export directory) |

### Existing Crates (Unchanged)

| Crate | Purpose |
|-------|---------|
| `thetta-geometry` | Shared types (`Piece`, `Polygon`, `Point2D`, `Placement`) |
| `eframe` / `egui` | GUI framework |
| `rfd` | Native macOS file dialogs (already in thetta-app from Sprint 001) |
| `serde` / `serde_json` | Serialization |
| `jagua-rs` | Collision detection engine (from Sprint 002) |
| `rand` | RNG for optimizer |

### System Dependencies

| Dependency | Install Command | Purpose |
|------------|----------------|---------|
| OpenCASCADE (OCCT) | `brew install opencascade` | C++ geometry kernel for STEP read/write, B-Rep operations |
| CMake | `brew install cmake` | Required by `opencascade-sys` build process |
| pkg-config | `brew install pkg-config` | Used by `opencascade-sys` to locate OCCT libraries |

Post-install verification:

```sh
brew install opencascade cmake pkg-config
pkg-config --libs opencascade
# Expected: -L/opt/homebrew/lib -lTKernel -lTKMath ...
```

If `pkg-config` does not find OCCT, set:

```sh
export PKG_CONFIG_PATH="/opt/homebrew/opt/opencascade/lib/pkgconfig:$PKG_CONFIG_PATH"
```

## Open Questions

1. **`opencascade-sys` vs `opencascade` crate:** The user confirmed preference for `opencascade-sys` (low-level FFI). However, this crate's API coverage is unknown. The higher-level `opencascade` crate wraps more OCCT functionality in safe Rust but may not expose `STEPControl_Writer` or `GCPnts_UniformAbscissa`. **Action:** Evaluate both crates' API coverage in Phase 0 before committing. If neither covers our needs, write custom `bindgen` bindings.

2. **`StepSession` thread safety:** OCCT objects are C++ heap-allocated and likely not `Send`/`Sync`. The app needs STEP data on both the main thread (for thickness switching) and background threads (for import/export). Options: (a) do all OCCT work on a dedicated persistent thread with channel communication, (b) wrap in `Arc<Mutex<>>` and hope OCCT is thread-safe for reads, (c) extract all needed data into Rust-owned types during import and only keep OCCT objects for export. **Recommendation:** Option (c) -- extract `Vec<Piece>` and thickness groups into Rust types; keep OCCT solids in a `Vec` behind `Arc<Mutex<>>` only for export.

3. **Face normal alignment heuristic:** The current plan uses "largest face." For solids where the top and bottom faces have the same area as a side face (e.g., a cube), this may pick the wrong face. Should we prefer faces whose normal is aligned with the smallest bounding-box dimension? **Recommendation:** Start with largest-face (simpler). Add normal-alignment as a P2 improvement if needed.

4. **Mesh tolerance default:** 0.5mm chord tolerance is a reasonable starting point. For large parts (>1m), this may produce thousands of points per curved edge. For small parts (<10mm), it may be too coarse. Should tolerance be relative to part size? **Recommendation:** Start with absolute 0.5mm. Add a relative mode (e.g., 0.5% of bounding box diagonal) as P2.

5. **Test STEP file sourcing:** We need 5 test files. Options: (a) generate via FreeCAD Python scripting, (b) generate via a one-off Rust binary using OCCT, (c) download from open model repositories (e.g., GrabCAD). **Recommendation:** Option (a) is most reliable and reproducible. Write a FreeCAD macro that generates all 5 files.

6. **Piece ID stability across thickness switches:** When the user switches thickness, all pieces are regenerated. Should piece IDs be deterministic (based on solid index) so that UI state (selection, colors) can be preserved across switches? **Recommendation:** Yes -- use `PieceId(solid_index as u64)` for deterministic IDs.

7. **Export coordinate system:** The 2D placement assumes solids are initially at the origin with the profile face on the XY plane. But the original solid may be arbitrarily positioned in 3D space. Should we first translate the solid so the profile face center is at the origin, then apply the nesting transform? **Recommendation:** Yes. During import, record the centroid of the profile face. During export, first translate the solid to place its profile centroid at the origin, then apply the nesting rotation + translation.
