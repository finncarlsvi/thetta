# Sprint 003 Intent: STEP File Import & Export Pipeline

## Seed Prompt

Implement the STEP file import/export pipeline: load STEP files via OpenCASCADE FFI, detect thickness groups, extract 2D profiles from matching solids, nest them, and export one STEP file per sheet with solids repositioned to their nested locations.

## Orientation Summary

- Sprint 001 delivered a working nesting app with custom collision engine, LBF + GLS optimizer, egui GUI (~1600 LOC)
- Sprint 002 (in progress) replaces the custom collision engine with jagua-rs, adds sparrow benchmark format support, and runs ESICUP benchmarks for quality validation
- The project's core value proposition (IDEA.md) is the full pipeline: STEP import -> thickness detection -> 2D profile extraction -> nesting -> STEP export per sheet
- `thetta-step` crate exists but is entirely stubbed (`todo!()` in both `import.rs` and `export.rs`)
- NESTING_PLAN.md Phase 1.2 and Phase 4.2 describe the STEP pipeline in detail, including the OpenCASCADE API calls needed
- The egui GUI needs a native file dialog (rfd crate) to replace the current manual path text input

## Relevant Codebase Areas

| Area | Files | Relevance |
|------|-------|-----------|
| STEP import stub | `thetta-step/src/import.rs` | Currently `todo!()` — needs full OpenCASCADE integration |
| STEP export stub | `thetta-step/src/export.rs` | Currently `todo!()` — needs STEP writer implementation |
| STEP crate root | `thetta-step/src/lib.rs` | Module declarations |
| STEP Cargo config | `thetta-step/Cargo.toml` | Needs opencascade-sys dependency |
| Geometry types | `thetta-geometry/src/types.rs` | Piece, Polygon, Placement — may need SolidRef or thickness metadata |
| App state | `thetta-app/src/state.rs` | File loading, thickness selection UI state |
| App panels | `thetta-app/src/panels.rs` | Thickness selector UI, file dialog trigger |
| App main | `thetta-app/src/main.rs` | File loading wiring, STEP vs JSON dispatch |
| App Cargo config | `thetta-app/Cargo.toml` | Needs rfd dependency for file dialogs |
| Nesting plan | `NESTING_PLAN.md` | Phase 1.2 (STEP import) and Phase 4.2 (STEP export) describe the pipeline |

## STEP Import Pipeline (from NESTING_PLAN.md)

1. `STEPControl_Reader::ReadFile()` — load STEP
2. `TransferRoots()` — parse into shapes
3. `TopExp_Explorer(TopAbs_SOLID)` — iterate all solids
4. For each solid, compute bounding box via `Bnd_Box` + `BRepBndLib::Add()`
5. Determine thickness = smallest bounding box dimension
6. Group solids by thickness (round to 0.1mm tolerance)
7. For the selected thickness, extract the largest face via `TopExp_Explorer(TopAbs_FACE)` + `BRepGProp::SurfaceProperties()`
8. Extract the outer wire of that face via `BRepTools::OuterWire()`
9. Discretize wire edges into polyline points via `GCPnts_UniformAbscissa` or `BRepMesh_IncrementalMesh`
10. Output: `Vec<Piece>` with 2D polygons

## STEP Export Pipeline (from NESTING_PLAN.md)

1. For each sheet's placed pieces, retrieve the original 3D solid
2. Apply the 2D transformation (translation + rotation) as a 3D transform on the XY plane
3. Write all transformed solids to a new STEP file via `STEPControl_Writer`

## Constraints

- Must preserve existing workspace crate structure (thetta-geometry, thetta-nesting, thetta-app, thetta-step)
- Must preserve background threading model (mpsc channel, AtomicBool cancel)
- OpenCASCADE is C++ — requires `opencascade-sys` crate for FFI (or alternative Rust bindings)
- OpenCASCADE must be installed as a system dependency (e.g., via Homebrew on macOS)
- STEP import may be slow for large files — should run on background thread with progress indication
- Must handle: multi-solid STEP files, varying thicknesses, solids with holes, non-planar faces (skip gracefully)
- GUI needs native file dialog (rfd) for both STEP load and STEP export
- Polygon discretization tolerance must balance accuracy vs vertex count

## Success Criteria

1. User can load a STEP file via native file dialog
2. App detects all thickness groups and shows them with solid counts
3. User can select a thickness; 2D profiles are extracted and displayed in viewport
4. Extracted profiles feed into the existing nesting pipeline (LBF + GLS)
5. User can export one STEP file per sheet with solids at nested positions
6. Round-trip test: import STEP -> nest -> export STEP -> reimport -> profiles match
7. Handles real-world STEP files (multi-solid, holes, varying thickness)
8. Native file dialogs for load and save operations

## Verification Strategy

- Unit tests for thickness detection on known STEP files
- Unit tests for profile extraction (known solid -> expected polygon)
- Integration test: load test STEP file -> extract profiles -> verify piece count and approximate areas
- Integration test: full round-trip (import -> nest -> export -> reimport)
- Manual verification: load real sheet-metal STEP files, inspect extracted profiles visually
- Build + test on macOS ARM64

## Uncertainty Assessment

| Factor | Level | Reason |
|--------|-------|--------|
| Correctness | High | OpenCASCADE API is complex; edge discretization, face selection, and winding order all need careful handling |
| Scope | Medium | Clear pipeline (import + export), but OpenCASCADE FFI depth is uncertain |
| Architecture | High | First C++ FFI in the project; opencascade-sys maturity/API coverage unknown; may need unsafe blocks |

## Open Questions

1. **opencascade-sys vs alternatives:** Is `opencascade-sys` the right crate? Alternatives include `opencascade` (higher-level wrapper), or vendoring OCCT and writing custom bindings. What's the maturity level?
2. **System dependency management:** How to handle OpenCASCADE installation? Homebrew (`brew install opencascade`)? Vendored? What about CI?
3. **Face selection heuristic:** "Largest face" may not always be the correct profile face. Should we offer a choice, or use a smarter heuristic (e.g., face normal aligned with thickness axis)?
4. **Polygon discretization parameters:** What tolerance/resolution for converting OCCT curves to polyline vertices? Too few points = inaccurate nesting; too many = slow collision detection.
5. **Hole handling:** How to detect and extract inner wires (holes) from the profile face? Do all OCCT face types support `OuterWire()` reliably?
6. **Large file handling:** Should STEP import run on a background thread with progress updates, similar to the nesting optimizer?
7. **Test STEP files:** Do we have representative sheet-metal STEP files for testing? Need to source or create them.
8. **SolidRef for export:** The current `Piece` type doesn't store a reference back to the original OCCT solid. How to maintain this mapping for export?
