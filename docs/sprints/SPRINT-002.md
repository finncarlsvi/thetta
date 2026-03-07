# Sprint 002: Benchmark-Grade Nesting via jagua-rs Integration

## Overview

Replace the custom O(n²) brute-force collision engine with jagua-rs's quadtree-backed collision detection, import sparrow's benchmark JSON format (34 ESICUP instances), and add a benchmark runner + GUI benchmark panel that reports strip length and utilization. All pieces placed with zero overlap on every benchmark instance.

**Scope:** Fork jagua-rs into workspace, rewrite collision engine, sparrow JSON import, benchmark CLI + GUI panel, discrete orientation support.
**Deferred:** STEP file import/export, multi-sheet support, matching sparrow's quality results.

## Use Cases

### UC1: Load Sparrow Benchmark Instance
- User provides a sparrow-format JSON file (e.g., `jakobs1.json`)
- System parses `{name, strip_height, items[{id, demand, allowed_orientations, shape}]}`
- Items expanded by `demand` into individual pieces with unique `PieceId`s
- `strip_height` → sheet height; width initialized to area-based lower bound
- Nesting starts automatically (LBF then GLS)

### UC2: Load Native Format (Preserved)
- Existing `Vec<Piece>` JSON format continues to work
- Auto-detects format: top-level object with `name` + `items` = sparrow; array = native
- Native format uses `SheetConfig` defaults (2440×1220mm), continuous rotation

### UC3: Run Benchmark via CLI
- `cargo run --release -p thetta-nesting --bin benchmark -- --input path/to/jakobs1.json --time 60`
- Runs LBF baseline → GLS optimizer → compression
- Prints: instance name, pieces placed/total, strip length, utilization %, elapsed time
- `--all` flag runs all 34 instances in `research/repos/sparrow/data/input/`
- `--csv results.csv` for machine-readable output
- Optional comparison column against known-best strip lengths

### UC4: Benchmark in GUI
- Benchmark JSON loads via existing file path input (auto-detected)
- Sheet height set from `strip_height`; width auto-sizes
- Stats panel shows strip length (used width) alongside utilization
- Viewport draws used-width indicator line

### UC5: Existing GUI Workflow (Preserved)
- All Sprint 001 use cases continue working
- Background threading unchanged (mpsc channel, AtomicBool cancel)

### UC6: Discrete Orientation Support
- Benchmark items with `allowed_orientations: [0, 90, 180, 270]` → LBF/GLS only sample those angles
- Native format items → continuous rotation `[0, 2π)`
- Empty orientation list `[]` → no rotation allowed

## Architecture

### Crate Structure (Updated)

```
thetta/
├── Cargo.toml                    (workspace)
├── crates/
│   ├── thetta-geometry/          (2D types, polygon ops, serde)
│   ├── thetta-nesting/           (LBF + GLS optimizer, jagua-rs wrapper, benchmark CLI)
│   ├── thetta-app/               (egui GUI, viewport, panels, state)
│   ├── thetta-step/              (STUB)
│   └── jagua-rs/                 (FORKED — collision detection engine)
├── test_data/                    (JSON polygon files)
├── research/repos/sparrow/       (benchmark instances)
```

### Engine Architecture Change

```
Current:                             New:
NestingEngine                        NestingEngine
  ├─ pieces: Vec<Piece>                ├─ instance: SPInstance (jagua-rs)
  ├─ inflated_items: Vec<JaguaItem>    ├─ problem: SPProblem (jagua-rs)
  ├─ placements: Vec<Option<..>>       ├─ pk_map: Vec<Option<PItemKey>>
  └─ collision via polygons_overlap()  └─ collision via CDEngine queries
```

Key change: jagua-rs's `CDEngine` uses a quadtree with surrogates for fast collision detection. The `Layout` manages hazard registration/deregistration automatically when items are placed or removed.

**Critical integration detail:** jagua-rs applies a `centering_transformation` to each item (translates centroid to origin). All `DTransformation` values must be in this internal coordinate space. Use `ext_to_int_transformation()` for conversion.

### Threading Model (Unchanged)

```
Main Thread (egui or CLI)            Background Thread
┌───────────────────┐               ┌─────────────────────┐
│ - Renders UI /    │ ◀── channel ──│ - Owns jagua-rs     │
│   prints results  │               │   SPProblem+Layout  │
│ - Polls updates   │ ── cancel ──▶ │ - Runs LBF then     │
│   each frame      │   AtomicBool  │   GLS optimizer     │
└───────────────────┘               └─────────────────────┘
```

## Implementation

### Phase 1: Fork jagua-rs and Engine Rewrite

#### Task 1.1: Fork jagua-rs into workspace
- Copy `research/repos/jagua-rs/jagua-rs/` → `crates/jagua-rs/`
- Add to workspace `members` in root `Cargo.toml`
- Enable `spp` feature (SPP problem types are feature-gated)
- Verify `cargo build` with the local crate
- Add jagua-rs's dependencies to workspace: `ordered-float`, `slotmap`, `ndarray`, `rayon`, `log`

#### Task 1.2: Rewrite conversion layer
**File:** `crates/thetta-nesting/src/conversion.rs` (REWRITE)

Replace stub types with real jagua-rs conversions:
```rust
pub fn pieces_to_sp_instance(
    pieces: &[Piece],
    sheet: &SheetConfig,
    orientations: Option<&[Vec<f64>]>,
) -> Result<SPInstance>;
```

Must handle:
- Our `Polygon` (`Vec<Point2D>`) → jagua-rs `ExtSPolygon` (`Vec<(f32, f32)>`)
- f64 → f32 coordinate conversion
- Strip closing: sparrow polygons may have first==last vertex (strip it)
- `allowed_orientations` → `RotationRange::{None, Discrete, Continuous}`
- Separation distance via `ShapeModifyConfig.offset = Some(separation / 2.0)`
- Item IDs must be consecutive from 0 (renumber if needed)
- Use jagua-rs `Importer` which applies `centering_transformation` internally

#### Task 1.3: Implement HazardCollector for collision cost
**File:** `crates/thetta-nesting/src/hazard_counter.rs` (NEW)

`HazardCollector` extends `HazardFilter` — must implement both traits:
```rust
struct CountingCollector {
    count: usize,
    exclude: Option<HazKey>,
}
impl HazardFilter for CountingCollector { ... }
impl HazardCollector for CountingCollector { ... }
```
This gives GLS a numeric collision cost (number of colliding hazards).

#### Task 1.4: Rewrite NestingEngine
**File:** `crates/thetta-nesting/src/engine.rs` (REWRITE)

```rust
pub struct NestingEngine {
    pieces: Vec<Piece>,
    sheet: SheetConfig,
    instance: SPInstance,
    problem: SPProblem,
    pk_map: Vec<Option<PItemKey>>,  // piece_idx → placement key
    orientations: Vec<RotationRange>,
}
```

Preserved public API:
- `new(pieces, sheet)` / `new_with_orientations(pieces, sheet, orientations)`
- `piece_count()`, `piece_area(idx)`, `sheet_config()`
- `place_item(idx, x, y, rotation) → bool` — check collision first, then place
- `force_place_item(idx, x, y, rotation)` — **remove existing placement first** (avoids demand underflow), then place unconditionally
- `remove_item(idx)` — deregister via stored `PItemKey`
- `placement_of(idx)`, `collision_cost(idx, x, y, rotation)`
- `current_result()`, `utilization()`

New strip packing methods:
- `strip_width() → f64`
- `fit_strip()` — **guard: skip if no items placed** (avoids panic)
- `change_strip_width(width: f64)`

`collision_cost()` implementation:
1. Transform item's `SPolygon` via `DTransformation`
2. Create `CountingCollector` excluding the item's own `HazKey` (via `haz_key_from_pi_key`)
3. Call `cde().collect_poly_collisions(shape, &mut collector)`
4. Return collector count as f64

#### Task 1.5: Adapt LBF heuristic
**File:** `crates/thetta-nesting/src/lbf.rs` (MODIFY)
- Accept `orientations: Option<&[RotationRange]>` in config
- When discrete, sample only from allowed angles instead of uniform `[0, TAU)`
- Call `engine.fit_strip()` after all pieces placed

#### Task 1.6: Adapt GLS optimizer
**File:** `crates/thetta-nesting/src/gls.rs` (MODIFY)
- Accept orientations per piece
- `uniform_sample` respects discrete orientations
- `coordinate_descent_refine` snaps rotation to nearest allowed angle in discrete mode
- Call `engine.fit_strip()` when feasible solution found

#### Task 1.7: Adapt sampler
**File:** `crates/thetta-nesting/src/sampler.rs` (MODIFY)
- Add `fn discrete_rotation_sample(rng, sheet, orientations: &[f64]) → Sample`
- Refinement snaps rotation to nearest allowed orientation when in discrete mode

#### Task 1.8: Adapt compression
**File:** `crates/thetta-nesting/src/compression.rs` (MODIFY)
- Use `engine.change_strip_width()` instead of creating new engine each iteration
- Guard `fit_strip()` calls with empty-placement check

### Phase 2: Sparrow Benchmark Import

#### Task 2.1: Benchmark parser
**File:** `crates/thetta-nesting/src/benchmark.rs` (NEW)

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

`fn load_benchmark(path: &Path) → Result<(Vec<Piece>, SheetConfig, BenchmarkMeta)>`:
- Parse sparrow JSON
- Expand `demand > 1` into individual pieces with unique sequential IDs
- Set `SheetConfig.height = strip_height`, width = `total_area / strip_height`
- Normalize orientations: degrees → radians, canonicalize to `[0, 2π)`, deduplicate
- Map `[]` → None (no rotation), explicit list → Discrete, absent → Continuous
- Validate: `strip_height > 0`, `demand ≥ 0`, polygon has ≥ 3 vertices

#### Task 2.2: Known-best results table
**File:** `crates/thetta-nesting/src/benchmark.rs` (EXTEND)

Hardcoded lookup table of best-known strip lengths (from sparrow records):
```rust
pub fn known_best(instance: &str) → Option<f64>;
```
Values extracted from sparrow SVG viewBox widths. Used for comparison reporting.

### Phase 3: Benchmark Runner

#### Task 3.1: CLI benchmark binary
**File:** `crates/thetta-nesting/src/bin/benchmark.rs` (NEW)

Uses `clap` for CLI args:
```
thetta-benchmark --input data/input/jakobs1.json --time 60
thetta-benchmark --all --time 120 --csv results.csv
```

Output:
```
Instance    | Pieces | Strip Length | Utilization | Best Known | Ratio | Time (s)
------------|--------|-------------|-------------|------------|-------|--------
jakobs1     | 25/25  | 13.42       | 82.3%       | 11.50      | 1.17  | 60.0
```

#### Task 3.2: Benchmark in nesting Cargo.toml
**File:** `crates/thetta-nesting/Cargo.toml`
- Add `[[bin]] name = "benchmark" path = "src/bin/benchmark.rs"`
- Add `clap` dependency with `derive` feature

### Phase 4: GUI Integration

#### Task 4.1: Benchmark loading in app state
**File:** `crates/thetta-app/src/state.rs` (MODIFY)
- Add `load_benchmark_file(&mut self, path: &Path) → Result<usize, String>`
- Auto-detect sparrow format by checking for `name` + `items` keys
- Store `allowed_orientations` in state for optimizer
- Store benchmark metadata (name, strip_height)

#### Task 4.2: Side panel updates
**File:** `crates/thetta-app/src/panels.rs` (MODIFY)
- When benchmark loaded, show instance name and strip height
- Stats panel: add "Strip Length" metric showing used width

#### Task 4.3: Viewport strip indicator
**File:** `crates/thetta-app/src/viewport.rs` (MODIFY)
- Draw used-width indicator line (dashed or colored differently)
- Sheet boundary still drawn at full configured width

### Phase 5: Testing and Validation

#### Task 5.1: Conversion unit tests
- Piece → jagua-rs Item → verify vertex count and approximate area
- Separation inflation produces larger bounding box
- `allowed_orientations` maps to correct `RotationRange` variant
- f64→f32 preserves geometry within tolerance
- Item ID renumbering is consecutive from 0

#### Task 5.2: Engine integration tests
**File:** `crates/thetta-nesting/tests/engine_integration.rs` (NEW)
- Place single rectangle → no collision
- Place two overlapping rectangles → collision detected
- Place and remove item → collision clears
- `force_place_item` with existing placement → old placement removed, new one registered
- Load jakobs1 → all 25 items parse correctly
- LBF on jakobs1 → all pieces placed with zero overlap

#### Task 5.3: Benchmark smoke test
**File:** `crates/thetta-nesting/tests/benchmark_smoke.rs` (NEW)
- Load each of 34 benchmark instances → no parse errors
- Run LBF on 3 small instances (jakobs1, jakobs2, shapes0) → all pieces placed
- Strip length is finite and positive
- Utilization is between 0 and 1

## Files Summary

### New Files

| File | Purpose | Est. LOC |
|------|---------|----------|
| `crates/jagua-rs/` | Forked collision detection engine | ~8000 (existing) |
| `thetta-nesting/src/benchmark.rs` | Sparrow JSON parsing + known results | 220 |
| `thetta-nesting/src/bin/benchmark.rs` | CLI benchmark runner | 150 |
| `thetta-nesting/src/hazard_counter.rs` | HazardCollector impl for collision cost | 60 |
| `thetta-nesting/tests/engine_integration.rs` | Engine integration tests | 120 |
| `thetta-nesting/tests/benchmark_smoke.rs` | Benchmark smoke tests | 80 |
| **Total new (excl. fork)** | | **~630** |

### Modified Files

| File | Changes | Est. Delta |
|------|---------|------------|
| `Cargo.toml` | Workspace members + deps | +15 |
| `thetta-nesting/Cargo.toml` | jagua-rs dep, clap, serde_json, `[[bin]]` | +15 |
| `thetta-nesting/src/lib.rs` | New module declarations | +3 |
| `thetta-nesting/src/conversion.rs` | Full rewrite: real jagua-rs conversion | ~150 (rewrite) |
| `thetta-nesting/src/engine.rs` | Full rewrite: jagua-rs CDEngine wrapper | ~280 (rewrite) |
| `thetta-nesting/src/lbf.rs` | Discrete orientation support, fit_strip | +25 |
| `thetta-nesting/src/gls.rs` | Discrete orientations, fit_strip | +30 |
| `thetta-nesting/src/sampler.rs` | Discrete rotation sampling + snap | +35 |
| `thetta-nesting/src/compression.rs` | Use engine strip width changes | ~30 (rewrite) |
| `thetta-nesting/src/runner.rs` | Pass orientations through | +15 |
| `thetta-app/src/state.rs` | Benchmark loading, orientation storage, strip length | +50 |
| `thetta-app/src/panels.rs` | Strip length display, benchmark metadata | +25 |
| `thetta-app/src/viewport.rs` | Used-width indicator line | +15 |
| `thetta-geometry/src/types.rs` | Strip length field in NestingResult | +5 |
| **Total modified** | | **~690 delta** |

### Estimated Total: ~1320 LOC (new + delta, excluding jagua-rs fork)

## Definition of Done

### P0 — Must Ship
- [ ] jagua-rs forked into `crates/jagua-rs/`, builds as workspace member
- [ ] `cargo build --release` succeeds on macOS ARM64
- [ ] jagua-rs `CDEngine` replaces all brute-force collision checks in engine.rs
- [ ] Can load any of 34 sparrow benchmark JSON files without error
- [ ] LBF places all pieces from `jakobs1.json` with zero collision (25/25 placed)
- [ ] GLS optimizer runs against jagua-rs-backed engine, sends progressive updates
- [ ] `cargo run --release -p thetta-nesting --bin benchmark -- --input jakobs1.json --time 60` produces output
- [ ] All 34 benchmark instances complete without panics (LBF baseline)
- [ ] Existing egui GUI launches and works for native JSON format (no regressions)
- [ ] Discrete `allowed_orientations` from benchmark format respected by LBF and GLS
- [ ] `fit_strip()` guarded against empty layout

### P1 — Should Ship
- [ ] Benchmark `--all` mode runs all 34 instances
- [ ] CSV output from benchmark runner
- [ ] Known-best comparison column (ratio against published sparrow results)
- [ ] GUI benchmark panel: auto-detect sparrow JSON, show instance name + strip length in stats
- [ ] Used-width indicator line in viewport for strip packing mode
- [ ] Compression phase uses `engine.change_strip_width()` efficiently
- [ ] Strip length reported in `NestingResult`

### P2 — Stretch
- [ ] All pieces placed on every benchmark instance (GLS achieves feasibility)
- [ ] Results within 50% of sparrow's published best on 5+ instances
- [ ] SVG solution export
- [ ] Performance: LBF on jakobs1 completes in under 5 seconds (release mode)

### Cut if Behind Schedule
- Known-best comparison table (Task 2.2)
- GUI benchmark panel (Tasks 4.1–4.3)
- Stretch goals

## Risks

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| **centering_transformation mismatch**: jagua-rs centers items at origin; placements in wrong coordinate space | High | High | Explicit task (1.2) to use `ext_to_int_transformation()`. Integration test verifies placement coordinates. |
| **`fit_strip()` panic on empty layout**: `.unwrap()` on empty iterator | High | Medium | Guard all `fit_strip()` calls with `placed_items.is_empty()` check |
| **Item ID consecutive-from-zero**: `SPInstance::new()` asserts consecutive IDs | High | Medium | Conversion layer renumbers items. Test with non-consecutive benchmark IDs. |
| **Demand accounting underflow**: `register_included_item` decrements demand count | Medium | Medium | `force_place_item` always removes old placement first, then places new |
| **HazardCollector trait complexity**: Extends HazardFilter, requires multi-method impl | Medium | Low | Dedicated task (1.3) with explicit acceptance criteria |
| **f64→f32 precision loss** | Low | Low | Benchmark coordinates are small (0-200 range). Convert at boundary only. |
| **Fork maintenance burden** | Low | Low | Pin to specific commit. Only modify if API changes needed. |
| **rayon transitive dependency** | Low | Low | jagua-rs SPP import uses rayon. Will be pulled in automatically via fork. |

## Security

- No network access at runtime. Fully local application.
- jagua-rs is forked locally — no git fetch at build time.
- Benchmark JSON treated as untrusted input: validate `strip_height > 0`, `demand` bounds, polygon vertex count limits.
- `dxf` field in benchmark JSON is metadata only — never resolved as a file path.
- File I/O via CLI argument or GUI text input. No arbitrary path traversal.
- JSON parsed via serde (safe, no arbitrary code execution).
- Panic containment: all parser/conversion errors return `Result`, no `unwrap` on user input paths.

## Dependencies

### New Rust Crates

| Crate | Purpose |
|-------|---------|
| `jagua-rs` (local fork) | Collision detection engine |
| `clap` | CLI argument parsing for benchmark binary |
| `serde_json` | Benchmark JSON parsing (in thetta-nesting) |

### Transitive (via jagua-rs fork)

| Crate | Purpose |
|-------|---------|
| `ordered-float` | `NotNan<f32>` in `DTransformation` |
| `slotmap` | `PItemKey` placement tracking |
| `ndarray` | Internal geometry operations |
| `rayon` | Parallel item import |
| `log` | Logging |

### Existing (Unchanged)

| Crate | Purpose |
|-------|---------|
| `eframe` / `egui` | GUI framework |
| `serde` | Serialization |
| `rand` | RNG for optimizer sampling |

## Open Questions

1. **SPProblem vs direct Layout+CDE:** Sprint uses `SPProblem` which provides `fit_strip()` and `change_strip_width()`. If API friction is too high, can fall back to direct `Layout` + `CDEngine` usage — but would need to reimplement strip management.

2. **jagua-rs fork scope:** Do we fork the entire repo or just the `jagua-rs` library crate? Recommendation: just the library crate (`jagua-rs/jagua-rs/`), not the binaries or examples.

3. **Benchmark instance units:** ESICUP instances use abstract units (not mm). Our native format uses mm. Benchmark mode should display raw units without mm suffix. Need to handle this in the stats panel.

4. **`simplify_tolerance` and `narrow_concavity_cutoff`:** jagua-rs `Importer` accepts these parameters. What values should we use? Recommendation: start with `None` (no simplification) for maximum fidelity, tune later if performance requires it.
