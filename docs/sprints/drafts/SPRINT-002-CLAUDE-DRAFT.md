# Sprint 002: Benchmark-Grade Nesting via jagua-rs Integration

## Overview

Replace the custom O(n^2) brute-force collision engine with jagua-rs's quadtree-backed collision detection engine, import sparrow's benchmark JSON format (34 ESICUP instances), and add a benchmark runner that reports strip length and utilization. The existing egui GUI continues working unchanged; benchmark mode is additive.

**Scope:** jagua-rs collision engine integration, sparrow JSON import, benchmark CLI runner, LBF/GLS adapted to new engine, strip packing mode.
**Deferred:** STEP file import/export, multi-sheet support, matching sparrow's two-phase explore/compress algorithm exactly.

## Use Cases

### UC1: Load Sparrow Benchmark Instance
- User provides a sparrow-format JSON file (e.g., `jakobs1.json`)
- System parses `{name, strip_height, items[{id, demand, allowed_orientations, shape}]}`
- Items are expanded by `demand` count into individual pieces
- `strip_height` becomes the sheet height; width is initialized to the area-based lower bound (total item area / strip height)
- Pieces are available for nesting via either GUI or benchmark runner

### UC2: Load Native Format (Preserved)
- Existing `Vec<Piece>` JSON format continues to work
- GUI file dialog auto-detects format by checking for top-level `name` + `items` keys (sparrow) vs array-of-objects (native)
- Native format uses `SheetConfig` defaults (2440x1220mm)

### UC3: Run Benchmark via CLI
- `cargo run --release -p thetta-nesting --bin benchmark -- --input path/to/jakobs1.json --time 60`
- Runs LBF baseline, then GLS optimizer with compression for the given time limit
- Prints: instance name, strip length, utilization %, pieces placed/total, elapsed time
- Optional `--all` flag to run all 34 instances in `research/repos/sparrow/data/input/`
- Results printed as a table and optionally written to CSV

### UC4: Run Benchmark via GUI
- "Load Benchmark" button in side panel accepts sparrow JSON files
- Sheet height is set from `strip_height`; width auto-sizes
- Nesting runs as usual (LBF then GLS with compression)
- Stats panel additionally shows strip length (used width) alongside utilization

### UC5: Existing GUI Workflow (Preserved)
- All Sprint 001 use cases continue to work: load native JSON, configure sheet, run nesting, select/delete pieces, viewport navigation
- Background threading model unchanged (mpsc channel, AtomicBool cancel)

## Architecture

### Crate Structure (Unchanged)

```
thetta/
+-- Cargo.toml                    (workspace)
+-- crates/
|   +-- thetta-geometry/          (2D types, polygon ops, serde)
|   +-- thetta-nesting/           (LBF + GLS optimizer, jagua-rs integration, benchmark CLI)
|   +-- thetta-app/               (egui GUI, viewport, panels, state)
|   +-- thetta-step/              (STUB)
+-- test_data/                    (JSON polygon files)
+-- research/repos/sparrow/       (benchmark instances)
```

### Dependency Graph (Updated)

```
thetta-app
+-- thetta-nesting
|   +-- thetta-geometry
|   +-- jagua-rs (git dependency)  <-- NEW real integration
|   +-- serde_json                 <-- NEW for benchmark loading
+-- thetta-geometry
+-- eframe / egui
+-- rfd (file dialogs)
+-- serde_json
```

### Data Flow (Updated)

```
Sparrow JSON ----+                  Native JSON
                 |                      |
                 v                      v
          sparrow_import()        serde::from_str()
                 |                      |
                 v                      v
            Vec<Piece> + SheetConfig (strip_height)
                          |
                          v
                 +------------------+     +------------------+
                 |  NestingEngine   |---->|    jagua-rs      |
                 |  (wraps SPP or   |<----|  CDEngine +      |
                 |   Layout+CDE)    |     |  Layout          |
                 +--------+---------+     +------------------+
                          |
                    mpsc::channel
                          |
                 +--------v---------+
                 |   egui App /     |
                 |   Benchmark CLI  |
                 +------------------+
```

### Engine Architecture Change

The current `NestingEngine` wraps stub types (`JaguaItem`, `JaguaContainer`) and performs brute-force polygon overlap checks in `engine.rs`. The new engine wraps jagua-rs's `Layout` and `CDEngine` directly:

```
Current:                             New:
NestingEngine                        NestingEngine
  +- pieces: Vec<Piece>                +- instance: SPInstance (jagua-rs)
  +- inflated_items: Vec<JaguaItem>    +- problem: SPProblem (jagua-rs)
  +- placements: Vec<Option<..>>       +- placements: HashMap<usize, PItemKey>
  +- collision via polygons_overlap()  +- collision via CDEngine queries
```

Key difference: jagua-rs's `CDEngine` uses a quadtree with surrogates for O(log n) collision detection instead of O(n^2) pairwise polygon checks. The `Layout` manages hazard registration/deregistration automatically when items are placed or removed.

### Threading Model (Unchanged)

```
Main Thread (egui or CLI)            Background Thread
+-------------------+               +---------------------+
| - Renders UI /    | <-- channel -- | - Owns jagua-rs     |
|   prints results  |               |   SPProblem+Layout  |
| - Polls updates   | -- cancel --> | - Runs LBF then     |
|   each frame      |   AtomicBool  |   GLS optimizer     |
+-------------------+               +---------------------+
```

## Implementation

### Phase 1: jagua-rs Integration Layer

#### Task 1.1: Add jagua-rs dependency
**File:** `crates/thetta-nesting/Cargo.toml`
- Add `jagua-rs` as a git dependency pointing to the local checkout or GitHub
- Add `serde_json` for benchmark file loading
- Add `ordered-float` (required by jagua-rs `DTransformation`)
- Add `anyhow` for error propagation from jagua-rs

#### Task 1.2: Sparrow JSON format types
**File:** `crates/thetta-nesting/src/benchmark.rs` (NEW)

Define serde-compatible types matching the sparrow/jagua-rs input format:

```rust
#[derive(Deserialize)]
pub struct BenchmarkInstance {
    pub name: String,
    pub items: Vec<BenchmarkItem>,
    pub strip_height: f64,
}

#[derive(Deserialize)]
pub struct BenchmarkItem {
    pub id: u64,
    pub demand: u64,
    pub allowed_orientations: Option<Vec<f64>>,
    pub shape: BenchmarkShape,
}
```

Provide `fn load_benchmark(path: &Path) -> Result<(Vec<Piece>, SheetConfig, BenchmarkMeta)>` that:
- Parses the sparrow JSON
- Expands `demand > 1` items into individual `Piece` entries with unique `PieceId`s
- Sets `SheetConfig.height = strip_height`, `SheetConfig.width = total_area / strip_height`
- Returns `BenchmarkMeta { name, allowed_orientations_per_item }` for orientation handling

#### Task 1.3: Rewrite conversion layer
**File:** `crates/thetta-nesting/src/conversion.rs` (REWRITE)

Replace the stub types with real jagua-rs conversions:

```rust
pub fn pieces_to_sp_instance(
    pieces: &[Piece],
    sheet: &SheetConfig,
    orientations: Option<&[Vec<f64>]>,
) -> Result<SPInstance>;

pub fn piece_to_jagua_item(
    piece: &Piece,
    separation: f64,
    orientations: Option<&[f64]>,
    config: &CDEConfig,
) -> Result<Item>;

pub fn extract_placements(
    problem: &SPProblem,
    pieces: &[Piece],
) -> Vec<Placement>;
```

Must handle:
- Our `Polygon` (outer: `Vec<Point2D>`) to jagua-rs `ExtSPolygon` (`Vec<(f32, f32)>`)
- f64 to f32 coordinate conversion (jagua-rs uses f32 throughout)
- Winding order normalization (jagua-rs expects CCW, same as ours)
- Strip closing: sparrow polygons may have first==last vertex (jagua-rs strips this)
- `allowed_orientations` from benchmark -> `RotationRange::Discrete` or `RotationRange::Continuous`
- Separation distance via `ShapeModifyConfig.offset = Some(separation / 2.0)`

#### Task 1.4: Rewrite NestingEngine
**File:** `crates/thetta-nesting/src/engine.rs` (REWRITE)

Replace the brute-force engine with a jagua-rs-backed engine. The public API is preserved to minimize changes in GLS and LBF:

```rust
pub struct NestingEngine {
    instance: SPInstance,
    problem: SPProblem,
    pieces: Vec<Piece>,
    item_placement_keys: Vec<Option<PItemKey>>,
}

impl NestingEngine {
    pub fn new(pieces: &[Piece], sheet: &SheetConfig) -> Self;
    pub fn new_with_orientations(
        pieces: &[Piece], sheet: &SheetConfig,
        orientations: Option<&[Vec<f64>]>,
    ) -> Self;

    pub fn piece_count(&self) -> usize;
    pub fn piece_area(&self, idx: usize) -> f64;
    pub fn sheet_config(&self) -> &SheetConfig;

    // Core API (preserved signatures)
    pub fn place_item(&mut self, idx: usize, x: f64, y: f64, rotation: f64) -> bool;
    pub fn force_place_item(&mut self, idx: usize, x: f64, y: f64, rotation: f64);
    pub fn remove_item(&mut self, idx: usize);
    pub fn placement_of(&self, idx: usize) -> Option<&Placement>;
    pub fn collision_cost(&self, idx: usize, x: f64, y: f64, rotation: f64) -> f64;
    pub fn current_result(&self) -> NestingResult;
    pub fn utilization(&self) -> f64;

    // New: strip packing specific
    pub fn strip_width(&self) -> f64;
    pub fn fit_strip(&mut self);
    pub fn change_strip_width(&mut self, width: f64);
}
```

Implementation notes for key methods:
- `collision_cost()`: Temporarily transform the item's `SPolygon`, query `CDEngine::detect_poly_collision()` with a filter excluding the item's own hazard. Count collisions via `collect_poly_collisions()` for a numeric cost (number of colliding hazards).
- `place_item()`: Check collision first via `collision_cost() == 0.0`, then call `problem.place_item(SPPlacement{...})`. Store returned `PItemKey`.
- `force_place_item()`: Call `problem.place_item()` unconditionally. The CDE will register the hazard even if it overlaps.
- `remove_item()`: Call `problem.remove_item(pk)` using stored `PItemKey`.
- `fit_strip()`: Delegate to `SPProblem::fit_strip()`.

#### Task 1.5: Adapt LBF heuristic
**File:** `crates/thetta-nesting/src/lbf.rs` (MODIFY)

Minimal changes needed since the `NestingEngine` API is preserved:
- When benchmark orientations are provided, sample only from `allowed_orientations` instead of uniform `[0, 2pi)`
- Add `allowed_orientations: Option<&[Vec<f64>]>` to `LbfConfig`
- Use `engine.fit_strip()` after all pieces are placed to report tight strip width

#### Task 1.6: Adapt GLS optimizer
**File:** `crates/thetta-nesting/src/gls.rs` (MODIFY)

Minimal changes needed:
- When benchmark orientations are provided, `uniform_sample` must respect `allowed_orientations` per piece
- Add `allowed_orientations` field to `GLSOptimizer` or pass through `OptimizerConfig`
- After finding a feasible solution, call `engine.fit_strip()` before reporting

#### Task 1.7: Adapt sampler
**File:** `crates/thetta-nesting/src/sampler.rs` (MODIFY)

- Add `fn discrete_rotation_sample(rng, sheet, orientations: &[f64]) -> Sample`
- `coordinate_descent_refine` should snap rotation to nearest allowed orientation when in discrete mode

#### Task 1.8: Adapt compression
**File:** `crates/thetta-nesting/src/compression.rs` (MODIFY)

- Use `engine.change_strip_width()` instead of creating a new engine each iteration
- Use `engine.fit_strip()` for final tightening

### Phase 2: Benchmark Runner

#### Task 2.1: Benchmark binary
**File:** `crates/thetta-nesting/src/bin/benchmark.rs` (NEW)

```rust
fn main() {
    // Parse CLI args: --input, --time, --all, --csv
    // Load benchmark instance(s)
    // Run nesting pipeline (LBF + GLS + compression)
    // Print results table
}
```

CLI interface:
```
thetta-benchmark --input data/input/jakobs1.json --time 60
thetta-benchmark --all --time 120 --csv results.csv
```

Output format:
```
Instance    | Pieces | Strip Length | Utilization | Time (s)
------------|--------|-------------|-------------|--------
jakobs1     | 25     | 13.42       | 82.3%       | 60.0
jakobs2     | 25     | 28.50       | 79.1%       | 60.0
```

#### Task 2.2: Benchmark result comparison
**File:** `crates/thetta-nesting/src/benchmark.rs` (EXTEND)

- Parse sparrow's SVG solution files from `research/repos/sparrow/data/records/` to extract strip widths (from SVG viewBox or `<rect>` element)
- `fn load_known_results() -> HashMap<String, f64>` returns best known strip lengths
- Benchmark runner optionally compares and prints `Ours / Best Known` ratio

### Phase 3: GUI Integration

#### Task 3.1: Benchmark loading in app state
**File:** `crates/thetta-app/src/state.rs` (MODIFY)

- Add `pub fn load_benchmark_file(&mut self, path: &Path) -> Result<usize, String>`
- Auto-detects sparrow format, sets `sheet_config.height` from `strip_height`
- Sets `sheet_config.width` to area-based estimate
- Stores `allowed_orientations` in state for the optimizer

#### Task 3.2: Side panel updates
**File:** `crates/thetta-app/src/panels.rs` (MODIFY)

- Add "Load Benchmark" button alongside existing "Load" button
- When benchmark is loaded, show instance name and strip height
- Stats panel: add "Strip Length" metric showing used width (rightmost piece edge)

#### Task 3.3: Strip width display
**File:** `crates/thetta-app/src/viewport.rs` (MODIFY)

- When in strip packing mode, draw a second vertical line at the "used width" boundary
- Sheet boundary still drawn at full configured width

### Phase 4: Testing and Validation

#### Task 4.1: Conversion unit tests
**File:** `crates/thetta-nesting/src/conversion.rs`
- Round-trip: `Piece` -> jagua-rs `Item` -> verify polygon vertex count and approximate area
- Verify separation inflation produces larger bounding box
- Verify `allowed_orientations` maps to correct `RotationRange` variant
- Verify f64->f32 conversion preserves geometry within tolerance

#### Task 4.2: Engine integration tests
**File:** `crates/thetta-nesting/tests/engine_integration.rs` (NEW)
- Place a single rectangle in an empty strip -> no collision
- Place two overlapping rectangles -> collision detected
- Place and remove an item -> collision clears
- Load `jakobs1.json` -> all 25 items parse correctly
- LBF on `jakobs1.json` -> all pieces placed with zero overlap

#### Task 4.3: Benchmark smoke test
**File:** `crates/thetta-nesting/tests/benchmark_smoke.rs` (NEW)
- Load each of the 34 benchmark instances -> no parse errors
- Run LBF on 3 small instances (jakobs1, jakobs2, shapes0) -> all pieces placed
- Verify strip length is finite and positive
- Verify utilization is between 0 and 1

## Files Summary

### New Files

| File | Purpose | Est. LOC |
|------|---------|----------|
| `thetta-nesting/src/benchmark.rs` | Sparrow JSON parsing, benchmark metadata, known results | 200 |
| `thetta-nesting/src/bin/benchmark.rs` | CLI benchmark runner binary | 150 |
| `thetta-nesting/tests/engine_integration.rs` | Engine integration tests | 120 |
| `thetta-nesting/tests/benchmark_smoke.rs` | Benchmark loading smoke tests | 80 |
| **Total new** | | **~550** |

### Modified Files

| File | Changes | Est. Delta |
|------|---------|------------|
| `Cargo.toml` | Workspace dependency for jagua-rs | +5 |
| `thetta-nesting/Cargo.toml` | Add jagua-rs, serde_json, anyhow, ordered-float; add `[[bin]]` for benchmark | +15 |
| `thetta-nesting/src/lib.rs` | Add `pub mod benchmark;` | +1 |
| `thetta-nesting/src/conversion.rs` | Full rewrite: stub types -> real jagua-rs conversion | ~150 (rewrite) |
| `thetta-nesting/src/engine.rs` | Full rewrite: brute-force -> jagua-rs CDEngine wrapper | ~250 (rewrite) |
| `thetta-nesting/src/lbf.rs` | Add discrete orientation support, strip fitting | +20 |
| `thetta-nesting/src/gls.rs` | Add discrete orientation support, strip fitting | +25 |
| `thetta-nesting/src/sampler.rs` | Add `discrete_rotation_sample`, orientation-aware refinement | +30 |
| `thetta-nesting/src/compression.rs` | Use engine strip width changes instead of rebuilding | +10 (net, rewrite ~30) |
| `thetta-nesting/src/runner.rs` | Pass orientations through to engine/optimizer | +10 |
| `thetta-app/src/state.rs` | Add `load_benchmark_file`, orientation storage, strip length stat | +40 |
| `thetta-app/src/panels.rs` | Benchmark load button, strip length display | +20 |
| `thetta-app/src/viewport.rs` | Used-width indicator line | +10 |
| `thetta-geometry/src/types.rs` | Add `strip_length` field to `Sheet` or `NestingResult` | +5 |
| **Total modified** | | **~590 delta** |

### Estimated Total Effort: ~1140 LOC (new + delta)

## Definition of Done

### P0 -- Must Ship
- [ ] `cargo build --release` succeeds with jagua-rs dependency on macOS ARM64
- [ ] jagua-rs `CDEngine` replaces all brute-force collision checks in `engine.rs`
- [ ] Can load any of the 34 sparrow benchmark JSON files without error
- [ ] LBF places all pieces from `jakobs1.json` with zero collision (verified by jagua-rs CDEngine)
- [ ] GLS optimizer runs against jagua-rs-backed engine, sends progressive updates
- [ ] `cargo run --release -p thetta-nesting --bin benchmark -- --input jakobs1.json --time 60` produces strip length + utilization output
- [ ] Existing egui GUI launches and works for native JSON format (no regressions)
- [ ] Strip length and utilization reported for benchmark instances
- [ ] All 34 benchmark instances complete without panics

### P1 -- Should Ship
- [ ] Benchmark runner `--all` mode runs all 34 instances
- [ ] CSV output from benchmark runner
- [ ] Results comparison against sparrow's published best (ratio column)
- [ ] GUI "Load Benchmark" button for sparrow JSON files
- [ ] Used-width indicator line in viewport for strip packing mode
- [ ] `allowed_orientations` from benchmark format respected by LBF and GLS (discrete rotation sampling)
- [ ] Compression phase uses `engine.change_strip_width()` for efficient re-optimization

### P2 -- Stretch
- [ ] Results within 20% of sparrow's published best on 5+ instances
- [ ] Benchmark results visualization in GUI (table view)
- [ ] SVG solution export matching sparrow's format
- [ ] SIMD feature flag passthrough to jagua-rs for maximum performance

### Cut if Behind Schedule
- Known-results comparison (Task 2.2)
- GUI benchmark panel (Task 3.2-3.3)
- Stretch goals

## Risks

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| jagua-rs API mismatch: `collision_cost()` needs to query CDE for a specific item without deregistering it | High | Medium | jagua-rs `detect_poly_collision` takes a `HazardFilter` that can exclude the item's own hazard key. Verified in source. |
| jagua-rs uses f32, our types use f64 -- precision loss | Medium | Low | Benchmark instances use small coordinates (0-100 range). Conversion is safe. Long-term may want to migrate our types to f32. |
| `force_place_item` with overlaps: jagua-rs Layout registers hazards for every placed item but does not prevent overlaps | Medium | Medium | Confirmed: `Layout::place_item` does not check collisions; it just registers the hazard. GLS can place overlapping items and query costs separately. |
| Nightly Rust required for jagua-rs SIMD features | Low | Low | SIMD is optional (feature-gated). Stable Rust works without it, just slower. |
| Benchmark quality far below sparrow | Medium | High | Expected: sparrow uses a highly tuned two-phase algorithm. Our GLS is simpler. Acceptable for Sprint 002; quality improvements are future work. Target: within 50% of sparrow initially, improve in Sprint 003. |
| jagua-rs git dependency breaks offline builds | Low | Medium | Pin to a specific commit hash. Consider vendoring if needed. |
| `CDEngine::detect_poly_collision` does not return a numeric cost, only bool | Medium | Low | Use `collect_poly_collisions` with a counting collector to get number of colliding hazards as cost. Alternatively, temporarily remove the item's hazard, query, then re-register. |

## Security

- No network access at runtime. Fully local application.
- jagua-rs is fetched at build time only (git dependency). Pin to a specific commit.
- File I/O via native OS dialog (GUI) or CLI argument (benchmark). No arbitrary path traversal.
- JSON parsed via serde (safe, no arbitrary code execution).
- Benchmark runner reads only from specified paths; writes only CSV to specified output path.
- No user-provided code, scripts, or macros.

## Dependencies

### New Rust Crates

| Crate | Purpose |
|-------|---------|
| `jagua-rs` | Collision detection engine (git dependency, pinned commit) |
| `serde_json` | Benchmark JSON parsing (in thetta-nesting) |
| `anyhow` | Error propagation from jagua-rs |
| `ordered-float` | Required by jagua-rs `DTransformation` (`NotNan<f32>`) |
| `clap` | CLI argument parsing for benchmark binary |

### Existing Crates (Unchanged)

| Crate | Purpose |
|-------|---------|
| `eframe` / `egui` | GUI framework |
| `rfd` | Native macOS file dialogs |
| `serde` | Serialization |
| `rand` | RNG for optimizer sampling |

### System Dependencies

None. jagua-rs is pure Rust.

## Open Questions

1. **Library dependency vs fork:** Should jagua-rs be a git dependency or forked into the workspace? Git dependency is simpler but creates a network requirement at build time. Forking allows local modifications but increases maintenance burden. **Recommendation:** Start with git dependency pinned to a specific commit; fork later only if API modifications are needed.

2. **SPP vs direct Layout+CDE usage:** jagua-rs offers `SPProblem` (manages strip width, item demands) vs using `Layout` + `CDEngine` directly. SPProblem provides `fit_strip()` and `change_strip_width()` which we need. **Recommendation:** Use `SPProblem` for the engine wrapper; it maps cleanly to our strip packing use case.

3. **`collision_cost` implementation strategy:** jagua-rs provides bool collision detection (`detect_poly_collision`) and collection-based detection (`collect_poly_collisions`). For GLS, we need a numeric cost. Options: (a) count colliding hazards via collector, (b) temporarily remove item, query, re-add. **Recommendation:** Option (a) -- use a counting `HazardCollector` implementation. This avoids the overhead of CDE hazard deregistration/re-registration per query.

4. **allowed_orientations vs continuous rotation:** Benchmark instances specify discrete orientations (e.g., `[0, 90, 180, 270]`). Our current optimizer uses continuous rotation. Options: (a) always use discrete when loading benchmarks, continuous for native format, (b) always use continuous and ignore `allowed_orientations`. **Recommendation:** Option (a). Respecting discrete orientations is important for benchmark comparability. The sampler and refiner need to be orientation-aware.

5. **f64 vs f32 type system:** jagua-rs uses f32 throughout; our geometry types use f64. Full migration to f32 is disruptive. **Recommendation:** Convert at the boundary in `conversion.rs`. Accept precision loss for now; it is acceptable for the coordinate ranges in ESICUP benchmarks (typically 0-200).

6. **Acceptable quality target:** What strip length quality is acceptable for Sprint 002? sparrow achieves near-optimal results after 600s. **Recommendation:** Target "all pieces placed" as P0. Target "within 50% of sparrow's best" as P1. Quality improvements are Sprint 003 scope.

7. **Benchmark runner location:** Should the benchmark binary live in `thetta-nesting` or get its own crate? **Recommendation:** `thetta-nesting/src/bin/benchmark.rs` as an additional binary target. It only depends on `thetta-nesting` and `thetta-geometry`, no GUI dependencies.
