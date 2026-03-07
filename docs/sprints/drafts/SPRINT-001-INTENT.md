# Sprint 001 Intent: Thetta MVP — STEP Import + Nesting Engine + GUI

## Seed Prompt

Build the first working version of Thetta: a macOS desktop application that imports STEP files, extracts 2D profiles by thickness, nests them into rectangular sheets using a GLS optimizer backed by jagua-rs, and displays the result in an interactive 2D viewport.

## Orientation Summary

- **Current state:** Rust workspace with 4 crates (thetta-app, thetta-nesting, thetta-step, thetta-geometry) all containing stubs/skeletons. The egui app compiles and shows an empty window.
- **Key dependencies to integrate:** jagua-rs (collision detection, MIT licensed, cloned at research/repos/jagua-rs/), OpenCASCADE via opencascade-sys for STEP I/O.
- **Design decisions locked:** Continuous rotation, jagua-rs for collision detection, GLS optimizer (sparrow-style), no part-in-part, time-based optimization with default limit, sequential sheet filling.
- **Reference implementations available locally:** sparrow (GLS optimizer), jagua-rs/lbf (left-bottom-fill reference), u-nesting (7 algorithms), SVGnest/Deepnest (NFP+GA).
- **Detailed nesting plan exists:** NESTING_PLAN.md with 4 phases, algorithm pseudocode, and architecture.

## Relevant Codebase Areas

| Area | Files | Status |
|------|-------|--------|
| Geometry types | `crates/thetta-geometry/src/types.rs` | Basic types defined |
| STEP import | `crates/thetta-step/src/import.rs` | Stub with `todo!()` |
| STEP export | `crates/thetta-step/src/export.rs` | Stub with `todo!()` |
| Nesting optimizer | `crates/thetta-nesting/src/optimizer.rs` | Stub with `todo!()` |
| GUI app | `crates/thetta-app/src/main.rs` | Empty egui window |
| jagua-rs reference | `research/repos/jagua-rs/` | Full source, MIT license |
| sparrow reference | `research/repos/sparrow/` | Full source, MIT license |

## Constraints

- **Platform:** macOS Apple Silicon primary target
- **Language:** Pure Rust (opencascade-sys FFI for STEP only)
- **GUI:** egui/eframe with wgpu backend
- **Performance:** Nesting must run on background thread, send progressive updates to UI
- **Sheet defaults:** 2440mm x 1220mm, 2mm separation
- **Rotation:** Continuous [0, 2π)

## Success Criteria

1. User can load a STEP file via file dialog
2. App detects and shows thickness options, auto-selects most common
3. 2D profiles are extracted and displayed in viewport with zoom/pan
4. GLS optimizer runs in background, progressively improves nesting
5. Viewport updates in real-time as optimizer finds better solutions
6. Multiple sheets are created when pieces don't fit on one
7. User can stop/resume optimization
8. User can delete individual shapes
9. User can export STEP files per sheet

## Verification Strategy

- Unit tests for geometry types and polygon operations
- Integration test: load a known STEP file → verify correct number of pieces extracted
- Integration test: nest a set of test polygons → verify no overlaps via jagua-rs
- Visual verification: run the app and inspect nesting quality
- Benchmark: measure collision queries/sec to verify jagua-rs integration performance

## Uncertainty Assessment

| Factor | Level | Reasoning |
|--------|-------|-----------|
| Correctness | Medium | GLS algorithm well-documented in sparrow, but tuning needed |
| Scope | High | Full pipeline from STEP to GUI is ambitious for one sprint |
| Architecture | Medium | jagua-rs integration is novel; OCCT FFI is uncharted |

## Open Questions

1. Should Sprint 1 tackle the full pipeline or focus on a subset (e.g., skip STEP, use hardcoded test polygons)?
2. How to handle OCCT FFI — use opencascade-sys directly or start with a simpler geometry input format (SVG/JSON)?
3. Should we implement the full GLS or start with a simpler LBF (left-bottom-fill) heuristic first?
4. What's the MVP for the GUI — minimal controls or polished panels?
