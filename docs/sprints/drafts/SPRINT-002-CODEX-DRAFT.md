# Sprint 002: Thetta Benchmark-Grade Nesting — Codex Draft

## Overview

Sprint 002 replaces Thetta's current brute-force collision logic in `crates/thetta-nesting/src/engine.rs` with `jagua-rs` collision detection and adds a benchmark workflow based on Sparrow's ESICUP JSON inputs.

Primary outcomes:
1. Replace custom O(n^2) overlap checks with `jagua-rs` `Layout` + `CDEngine` collision queries.
2. Import all 34 benchmark instances in `research/repos/sparrow/data/input/*.json`.
3. Add a benchmark runner that reports strip length and utilization (density).
4. Keep existing LBF and GLS flows operational on top of the new engine.
5. Keep the current egui app working; benchmark support is additive.
6. Respect benchmark `allowed_orientations` constraints during placement/sampling.

Non-goals for this sprint:
- Matching Sparrow SOTA results exactly.
- Replacing Thetta UI architecture.
- Multi-quality material zones (not present in Sparrow benchmark inputs).

## Use Cases

1. UC1: Load benchmark JSON instance
- User selects `research/repos/sparrow/data/input/<instance>.json`.
- Parser reads `{ name, strip_height, items[] }` with `demand`, `allowed_orientations`, and polygon `shape`.
- Thetta expands demanded items into nestable pieces and starts optimization.

2. UC2: Run benchmark from CLI
- Developer runs benchmark command over one instance or all 34 instances.
- Runner executes LBF baseline, then GLS, then optional compression.
- Runner outputs per-instance metrics: strip length, utilization, runtime, placed/total.

3. UC3: Keep interactive nesting UI functioning
- Existing JSON loading and optimization pipeline still works.
- Collision behavior and feasible checks are now driven by `jagua-rs`.
- Progressive updates continue to flow through `NestingUpdate` channel.

4. UC4: Orientation-constrained nesting
- If benchmark item has discrete `allowed_orientations`, LBF and GLS only sample those angles.
- If orientation list is absent, continuous rotation remains available.

## Architecture

### 1. Engine replacement with jagua-rs core entities

`NestingEngine` becomes a wrapper around:
- `jagua_rs::probs::spp::entities::SPInstance`
- `jagua_rs::probs::spp::entities::SPProblem`
- `jagua_rs::entities::Layout` (inside `SPProblem`)
- `jagua_rs::collision_detection::CDEngine` (inside `Layout`)

Key `jagua-rs` APIs used:
- `Item::new(id, OriginalShape, RotationRange, min_quality, surrogate_config)`
- `Strip::new(fixed_height, cde_config, shape_modify_config, width)`
- `SPProblem::new(instance)`
- `SPProblem::place_item(SPPlacement { item_id, d_transf })`
- `SPProblem::remove_item(pkey)`
- `Layout::is_feasible()`
- `Layout::cde().detect_poly_collision(shape, filter)`
- `Layout::cde().collect_poly_collisions(shape, collector)`
- `SPProblem::fit_strip()` and `SPProblem::change_strip_width(...)`

### 2. Conversion boundary

`crates/thetta-nesting/src/conversion.rs` becomes the typed bridge between:
- `thetta_geometry::types::*` (f64-based app model)
- `jagua-rs` external/internal types (f32-based geometry and SPP entities)

Planned conversion responsibilities:
- `Polygon -> jagua_rs::io::ext_repr::ExtShape::SimplePolygon`
- Piece orientation policy -> `RotationRange::{None, Continuous, Discrete}`
- Separation gap -> `Importer::new(..., min_item_separation = Some(sheet.separation), ...)` (which maps to shape offset)
- `Placement <-> DTransformation`

### 3. Benchmark input model

Add benchmark-specific DTOs in Thetta:
- `BenchmarkInstance { name, strip_height, items: Vec<BenchmarkItem> }`
- `BenchmarkItem { id, demand, allowed_orientations, shape }`

Parsing rules based on Sparrow JSON:
- `shape.type` must support `simple_polygon` initially.
- Closed polygon input (`first == last`) is normalized to open ring for internal model.
- `allowed_orientations` values are degrees in JSON; convert to radians for `jagua-rs`.

### 4. Optimizer compatibility

LBF and GLS are retained, but collision and feasibility checks move to jagua-backed `NestingEngine` methods:
- `place_item` uses `SPProblem::place_item` and immediate collision checks via CDE.
- `force_place_item` uses replace semantics: remove existing placement key, attempt new placement (or maintain infeasible candidate through collision collector score path).
- `collision_cost` computed from number/type of hazards returned by `collect_poly_collisions`.
- `current_result` built from placed items in layout snapshots.

### 5. UI and background execution compatibility

No threading model changes are required:
- Optimizer still runs in background thread.
- `NestingUpdate::{Improved, Finished}` remains the app contract.
- `AppState::poll_updates()` and stats rendering remain unchanged except metric enrichment (strip length display for benchmark mode).

## Implementation (Phased with Tasks)

### Phase 1: Add jagua-rs dependency and engine skeleton

1. Add dependency wiring
- Update `crates/thetta-nesting/Cargo.toml` with `jagua-rs` plus required support crates (`anyhow`, possibly `slotmap` use via jagua APIs where needed).
- Pin version/path strategy for reproducibility in workspace.

2. Replace stub conversion structures
- Remove placeholder `JaguaItem`, `JaguaContainer`, `JaguaLayout` structs.
- Introduce real conversion functions to construct:
  - `ExtItem` + `ExtSPInstance` input models
  - `Importer` for item import
  - `SPInstance` + `SPProblem`

3. Rebuild `NestingEngine`
- Internal state tracks:
  - `SPProblem`
  - mapping `piece_idx -> item_id`
  - mapping `piece_idx -> Option<PItemKey>` for removal/move
- Re-implement public methods used by LBF/GLS:
  - `new`, `piece_count`, `piece_area`, `sheet_config`
  - `place_item`, `force_place_item`, `remove_item`, `placement_of`
  - `collision_cost`, `current_result`, `utilization`

4. Collision scoring policy
- Implement collector-based score:
  - `0.0` means no collision hazards.
  - positive score based on collected hazard count, with higher penalty for exterior hazard.
- Keep score monotonic so GLS can still minimize it.

### Phase 2: Sparrow benchmark JSON import

1. Add benchmark parser module (Thetta side)
- Parse `name`, `strip_height`, and `items[*]` fields.
- Expand `demand` into repeated pieces while retaining source item metadata.

2. Orientation handling
- Convert degrees to radians.
- Canonicalize angles into `[0, 2π)`.
- Deduplicate equivalent angles.
- Map to:
  - empty or `[0]` -> `RotationRange::None`
  - explicit list -> `RotationRange::Discrete(vec![...])`
  - absent -> `RotationRange::Continuous`

3. Instance conversion tests
- Add tests for at least:
  - one simple instance (`jakobs1.json`)
  - one large instance (`gardeyn9.json`)
- Validate piece count expansion by demand.
- Validate orientation mapping and polygon ring normalization.

### Phase 3: LBF and GLS migration to jagua-backed engine

1. LBF update
- Keep sort-by-area and bottom-left selection behavior.
- Rotation candidate set now comes from per-piece allowed orientations (discrete set or sampled continuous fallback).
- Placement viability determined by jagua collision path.

2. GLS update
- Keep current guided move structure and penalty updates.
- Ensure candidate generation respects allowed orientation constraints.
- Ensure `force_place_item` no longer bypasses state consistency.

3. Compression update
- Replace geometric right-edge scan with `SPProblem::fit_strip()` + strip width updates where appropriate.
- Ensure compressed result still translates into Thetta `NestingResult` for GUI.

4. Regression tests
- No-overlap invariant for feasible results.
- `remove_item` and replace-item move stability.
- LBF+GLS run does not panic on all benchmark inputs.

### Phase 4: Benchmark runner and reporting

1. Add benchmark runner entrypoint
- CLI-oriented module in `thetta-nesting` or workspace binary target.
- Modes:
  - single instance path
  - all instances in a directory

2. Metrics reporting
- Per run report:
  - `instance_name`
  - `strip_height`
  - `strip_length` (final used width)
  - `utilization = total_item_area / (strip_length * strip_height)`
  - `runtime_ms`
  - `placed/total`
- Add CSV/JSONL output option for batch comparison.

3. Baseline comparison hooks
- Add optional field for `best_known_length` (if provided manually/table-driven).
- Emit `% gap` when best-known value exists.

4. Run all 34 instances
- Validate full corpus executes without crashes.
- Capture summary table and worst/best gaps.

### Phase 5: Keep egui flow stable and expose benchmark mode

1. Preserve existing JSON flow
- Existing app load path for current format remains supported.
- Add benchmark-format detection by top-level object shape.

2. App state additions
- Add optional benchmark metadata in `AppState` (instance name, strip length).
- Display strip length and utilization in stats panel when available.

3. Non-breaking UX
- No required UI redesign.
- Benchmark mode can start as file-based auto-detect with same Start/Stop controls.

## Files Summary

Primary files to modify:
- `crates/thetta-nesting/Cargo.toml`
- `crates/thetta-nesting/src/engine.rs`
- `crates/thetta-nesting/src/conversion.rs`
- `crates/thetta-nesting/src/lbf.rs`
- `crates/thetta-nesting/src/gls.rs`
- `crates/thetta-nesting/src/compression.rs`
- `crates/thetta-nesting/src/runner.rs`
- `crates/thetta-nesting/src/lib.rs`
- `crates/thetta-app/src/state.rs`

New files/modules expected:
- `crates/thetta-nesting/src/benchmark.rs` (instance parsing + conversion)
- `crates/thetta-nesting/src/benchmark_runner.rs` (batch execution + reporting)
- `crates/thetta-nesting/src/orientation.rs` (angle canonicalization and sampling helpers)
- `docs/sprints/drafts/SPRINT-002-CODEX-DRAFT.md` (this draft)

Tests likely added:
- `crates/thetta-nesting/tests/benchmark_import.rs`
- `crates/thetta-nesting/tests/engine_jagua_integration.rs`
- `crates/thetta-nesting/tests/benchmark_runner_smoke.rs`

## Definition of Done

1. Engine replacement complete
- `engine.rs` no longer uses custom polygon overlap code paths.
- Collision checks are backed by jagua `CDEngine`.

2. Benchmark format support
- All 34 Sparrow input JSON files parse successfully.
- `allowed_orientations` are enforced during candidate generation.

3. Benchmark runner
- Single and batch modes run and print strip length + utilization.
- Results can be exported in machine-readable format (CSV or JSONL).

4. Optimizer compatibility
- LBF and GLS run end-to-end against jagua-backed engine.
- Existing `run_nesting` pipeline still emits progressive updates.

5. GUI compatibility
- Existing egui workflow still loads data and renders nesting results.
- Benchmark files can be loaded without breaking current app behavior.

6. Validation
- Automated tests cover conversion, orientations, and engine placement invariants.
- Full corpus benchmark smoke run completes without panic.

## Risks

1. API impedance risk
- `jagua-rs` favors `SPProblem`/`Layout` semantics, while current engine assumes direct random access placement by index.
- Mitigation: explicit mapping tables for `piece_idx <-> PItemKey` and strict replace/remove semantics.

2. Metric comparability risk
- Strip length definition can differ (fitted strip width vs rightmost bbox + offsets).
- Mitigation: standardize on one definition and document it in benchmark output.

3. Orientation correctness risk
- Degree/radian conversion and normalization bugs can silently degrade quality.
- Mitigation: dedicated orientation unit tests and snapshot checks per instance.

4. Performance risk
- jagua conversion overhead or frequent snapshot usage may add latency.
- Mitigation: reuse `SPProblem` state and avoid rebuilding imported items per move.

5. Numerical robustness risk
- f64 (Thetta) to f32 (jagua) conversion can alter near-touching feasibility.
- Mitigation: consistent tolerances and separation policy across parser/importer/engine.

## Security

1. Untrusted benchmark file handling
- Treat JSON as untrusted input.
- Enforce size and shape limits during parse (max vertices, max items, max demand expansion).

2. Path handling
- Benchmark runner should avoid executing or resolving external `dxf` paths from input.
- `dxf` field is metadata only for this sprint.

3. Resource controls
- Add runtime limits for batch benchmarks to prevent accidental long-running jobs.
- Keep cancellation token support wired through benchmark runs.

4. Panic containment
- Parser and conversion errors return typed `Result` errors; no `unwrap` on user-provided content paths.

## Dependencies

1. New Rust dependencies
- `jagua-rs` (collision and SPP entities)
- `anyhow` or `thiserror` for conversion/runner errors
- `serde`/`serde_json` (already present, reused for benchmark parsing)

2. Existing repo assets used
- Input instances: `research/repos/sparrow/data/input/*.json` (34 files)
- API reference implementation: `research/repos/jagua-rs/jagua-rs/src/`

3. Build/runtime constraints
- Confirm selected `jagua-rs` revision compatibility with current Rust toolchain and platform.
- If nightly-only features are required by selected config, document toolchain setup in README/bench docs.

## Open Questions

1. Engine API surface
- Should `NestingEngine` preserve exactly the current method signatures, or can we introduce explicit error-returning variants for jagua failures?

2. Collision cost function
- Do we score by hazard count only, or weighted by hazard type (exterior > piece-piece) to better guide GLS?

3. Benchmark output location
- Should benchmark artifacts live under `research/results/` or `target/benchmarks/`?

4. Comparison baseline source
- Where should best-known strip lengths be stored for `% gap` reporting (checked-in table vs generated from Sparrow records)?

5. GUI benchmark mode scope
- Sprint target: auto-detect benchmark JSON only, or add explicit benchmark panel with batch-run controls?

6. Orientation defaults for legacy input
- For current Thetta JSON format (no orientation field), should we continue continuous rotation, or introduce optional discrete constraints there too?
