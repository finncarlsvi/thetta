# Sprint 001: Thetta MVP — Claude Draft

## Overview

Build the first working version of Thetta: load STEP files, extract 2D profiles by thickness, nest them into 2440x1220mm sheets using a GLS optimizer backed by jagua-rs, and visualize the result in an interactive egui viewport.

This sprint delivers the complete pipeline from file input to visual output. STEP export is deferred to Sprint 2 to reduce scope.

## Use Cases

### UC1: Load STEP File
- User clicks "Load" or drags a STEP file onto the window
- App parses the file, groups solids by thickness
- Auto-selects the most common thickness
- Displays extracted 2D profiles in the viewport

### UC2: Configure Sheet Parameters
- User adjusts sheet width/height (default 2440x1220mm)
- User adjusts separation distance (default 2mm)
- Changes trigger re-nesting

### UC3: Run Nesting
- Nesting starts automatically after loading
- Optimizer runs in background, progressively improves
- Viewport updates in real-time
- Stats panel shows: utilization %, pieces placed, time elapsed, sheets used

### UC4: Control Optimization
- User can stop optimization at any time
- User can resume/restart optimization
- Default time limit: 60s, then pauses (user can continue)

### UC5: Manage Pieces
- User can select and delete individual pieces from the nest
- User can reload all pieces from a file
- Deleting/reloading triggers re-nesting

### UC6: View Results
- 2D top-down viewport with zoom (scroll wheel) and pan (middle-click drag)
- Each sheet shown as a labeled rectangle
- Pieces colored by source file
- Selected piece highlighted

## Architecture

### Crate Dependency Graph

```
thetta-app
├── thetta-nesting
│   ├── thetta-geometry
│   └── jagua-rs (git dependency)
├── thetta-step
│   └── thetta-geometry
└── thetta-geometry
```

### Data Flow

```
STEP File → [thetta-step] → Vec<Piece> → [thetta-nesting] → NestingResult → [thetta-app] → Viewport
                                               ↕
                                          [jagua-rs CDE]
```

### Threading Model

```
Main Thread (egui)                    Background Thread
┌──────────────┐                     ┌──────────────────┐
│ UI rendering │ ←── mpsc channel ── │ GLS Optimizer     │
│ Viewport     │                     │ (owns jagua-rs    │
│ Controls     │ ── cancel token ──→ │  CDE + Layout)    │
└──────────────┘                     └──────────────────┘
```

- Optimizer sends `NestingUpdate` messages (improved placements) via `mpsc::channel`
- UI polls `rx.try_recv()` each frame
- `Arc<AtomicBool>` cancel token for stop/resume
- jagua-rs is single-threaded per CDE; parallelism happens at the optimizer level (multiple candidate evaluations)

## Implementation

### Phase 1: Geometry + Test Data (no STEP yet)

**Goal:** Get the nesting pipeline working with hardcoded test polygons before tackling OCCT FFI.

#### Task 1.1: Enhance geometry types
- Add `Polygon::area()`, `Polygon::bounding_box()`, `Polygon::translate()`, `Polygon::rotate()`
- Add `serde` derives for JSON serialization (test data)
- Add test polygon generators (rectangles, L-shapes, circles approximated as polygons)

#### Task 1.2: jagua-rs integration in thetta-nesting
- Add jagua-rs as a git dependency from the local clone (or GitHub)
- Write a conversion layer: `thetta_geometry::Polygon` → `jagua_rs::geometry::SimplePolygon`
- Create a `NestingEngine` struct that wraps jagua-rs's `BPProblem` (bin packing problem)
- Implement basic left-bottom-fill (LBF) as a first placement strategy using jagua-rs's `lbf` reference

#### Task 1.3: Basic egui viewport
- Implement `NestingViewport` widget with zoom/pan via `RectTransform`
- Render polygons as filled shapes with outlines
- Render sheet boundaries as dashed rectangles
- Color-coded pieces

#### Task 1.4: Wire up the pipeline with test data
- On app start, generate test polygons (mix of shapes)
- Run LBF placement
- Display result in viewport
- Verify: no overlaps, pieces within sheet bounds

### Phase 2: GLS Optimizer

#### Task 2.1: Implement GLS core loop
Based on sparrow's `optimizer/explore.rs`:
- Random initialization (place all pieces, overlaps allowed)
- Piece selection weighted by collision cost × penalty weight
- Candidate position sampling (uniform random within sheet)
- Coordinate descent refinement on best candidates
- Dynamic penalty weight augmentation
- Feasibility detection (total overlap == 0)

#### Task 2.2: Implement compression phase
Based on sparrow's `optimizer/compress.rs`:
- When feasible solution found, record best strip length
- Reduce container width/height, re-run exploration
- Binary search for minimum feasible length

#### Task 2.3: Background threading
- Spawn optimizer on `std::thread`
- Send `NestingUpdate::Improved(result)` on each improvement
- Send `NestingUpdate::Finished(result)` when done/cancelled
- UI updates from channel each frame

#### Task 2.4: Sequential multi-sheet
- After optimizer converges on sheet 1, identify unplaced pieces
- Start new optimizer instance for sheet 2 with remaining pieces
- Continue until all pieces placed

### Phase 3: STEP Import

#### Task 3.1: Set up opencascade-sys
- Add `opencascade-sys` or `occt-sys` as dependency
- Write build.rs to find/link OCCT (brew install opencascade)
- Verify it compiles on Apple Silicon

#### Task 3.2: STEP reader FFI
- Call `STEPControl_Reader::ReadFile()` and `TransferRoots()`
- Iterate solids via `TopExp_Explorer(TopAbs_SOLID)`
- Compute bounding box per solid via `BRepBndLib::Add()`
- Group by thickness (smallest BB dimension)

#### Task 3.3: Profile extraction FFI
- For each solid matching target thickness:
  - Find the largest face by area (`BRepGProp::SurfaceProperties()`)
  - Extract outer wire (`BRepTools::OuterWire()`)
  - Discretize edges to polyline (`GCPnts_UniformAbscissa`)
- Return `Vec<Piece>`

#### Task 3.4: Thickness auto-detection UI
- Show dropdown of detected thicknesses with solid counts
- Auto-select most common
- On change, re-extract profiles and re-nest

### Phase 4: GUI Polish

#### Task 4.1: File loading
- File dialog via `rfd` crate (native macOS file picker)
- Drag-and-drop support via egui's `DroppedFile` API
- Support loading multiple files

#### Task 4.2: Control panels
- Side panel: file list, thickness selector, sheet dimensions, separation
- Bottom panel: stats (utilization, pieces, time, sheets)
- Toolbar: Start/Stop nesting, Reset, Export (disabled until Sprint 2)

#### Task 4.3: Piece interaction
- Click to select a piece in viewport
- Delete key removes selected piece
- Right-click context menu: delete, info
- "Reload" button per file resets its pieces

#### Task 4.4: Multi-sheet navigation
- Tab bar or carousel for switching between sheets
- Each sheet has its own viewport state (zoom/pan)

## Files Summary

### New Files

| File | Purpose |
|------|---------|
| `crates/thetta-geometry/src/polygon_ops.rs` | Area, bbox, transform operations |
| `crates/thetta-geometry/src/test_shapes.rs` | Test polygon generators |
| `crates/thetta-nesting/src/engine.rs` | NestingEngine wrapping jagua-rs |
| `crates/thetta-nesting/src/gls.rs` | GLS optimizer core loop |
| `crates/thetta-nesting/src/compression.rs` | Phase B compression |
| `crates/thetta-nesting/src/conversion.rs` | Geometry type conversion to/from jagua-rs |
| `crates/thetta-app/src/viewport.rs` | 2D viewport widget with zoom/pan |
| `crates/thetta-app/src/panels.rs` | Side panel, stats, controls |
| `crates/thetta-app/src/state.rs` | Application state management |
| `crates/thetta-step/src/ffi.rs` | Raw OpenCASCADE FFI wrappers |
| `crates/thetta-step/src/build.rs` | Build script for OCCT linking |

### Modified Files

| File | Changes |
|------|---------|
| `Cargo.toml` | Add workspace dependencies |
| `crates/thetta-nesting/Cargo.toml` | Add jagua-rs dependency |
| `crates/thetta-step/Cargo.toml` | Add opencascade-sys dependency |
| `crates/thetta-app/Cargo.toml` | Add rfd, serde dependencies |
| `crates/thetta-app/src/main.rs` | Full app implementation |
| `crates/thetta-geometry/src/types.rs` | Add serde derives, methods |

## Definition of Done

### Must Have (Sprint 1 ship criteria)
- [ ] App launches on macOS Apple Silicon
- [ ] Can load at least one STEP file and extract 2D profiles
- [ ] Thickness auto-detection works and can be changed
- [ ] GLS optimizer runs in background and progressively improves
- [ ] Viewport shows nested pieces with zoom/pan
- [ ] No overlaps in final nesting (verified by jagua-rs)
- [ ] All pieces within sheet bounds
- [ ] Separation distance respected between pieces
- [ ] Multiple sheets created when needed
- [ ] User can stop/resume optimization
- [ ] User can delete pieces

### Nice to Have (stretch goals)
- [ ] Drag-and-drop file loading
- [ ] Multiple file loading
- [ ] STEP export per sheet
- [ ] Piece color-coding by source file
- [ ] Utilization percentage display

## Risks

| Risk | Impact | Likelihood | Mitigation |
|------|--------|-----------|------------|
| opencascade-sys FFI doesn't compile on ARM64 | High | Medium | Fall back to shelling out to a Python/CadQuery subprocess; or use occt-sys which builds from source |
| jagua-rs API is unstable or poorly documented | Medium | Low | We have full source code; can fork and pin a specific commit |
| GLS optimizer produces poor results without tuning | Medium | Medium | Start with LBF as baseline; tune GLS parameters against ESICUP benchmarks |
| egui zoom/pan is janky for CAD use | Low | Medium | Well-documented pattern; MuginCAD proves it works |
| STEP files with complex geometry (NURBS, splines) fail to discretize | Medium | Medium | Use BRepMesh with configurable tolerance; gracefully skip unsupported geometry |

## Security

- STEP file parsing happens via OCCT (mature C++ library). Input validation is OCCT's responsibility.
- No network access. All processing is local.
- File paths are validated before read/write.
- No user-provided code execution.

## Dependencies

### External Crates
| Crate | Version | Purpose |
|-------|---------|---------|
| `eframe` | 0.31 | GUI framework |
| `egui` | 0.31 | Immediate mode UI |
| `jagua-rs` | git (main) | Collision detection engine |
| `opencascade-sys` or `occt-sys` | latest | STEP file I/O |
| `rfd` | latest | Native file dialogs |
| `serde` + `serde_json` | latest | Serialization for test data |

### System Dependencies
| Dependency | Install |
|------------|---------|
| OpenCASCADE | `brew install opencascade` |
| Rust nightly (optional, for SIMD) | `rustup install nightly` |

## Open Questions

1. **jagua-rs git dependency**: Should we depend on GitHub directly or vendor a copy? Pinning a git commit is safer but harder to update.
2. **OCCT FFI approach**: opencascade-sys vs occt-sys vs writing our own build.rs? Need to evaluate which actually compiles on ARM64 macOS.
3. **Test STEP files**: Where to get representative test files for development? Need a few small files with flat sheet-metal-like parts.
4. **Sprint scope**: This is ambitious. If we're behind, Phase 3 (STEP import) could be deferred to Sprint 2, using JSON polygon files as input for Sprint 1.
