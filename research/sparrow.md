# sparrow -- State-of-the-Art 2D Nesting Heuristic

## 1. Overview

**sparrow** is a state-of-the-art optimization heuristic for 2D irregular strip packing problems (also known as nesting problems). Given a set of irregularly shaped items and an infinite-length strip of fixed height, the goal is to pack all items into the strip while minimizing the strip width used.

- **Author**: Jeroen Gardeyn (KU Leuven, CODeS research group)
- **Paper**: *"An open-source heuristic to reboot 2D nesting research"* (arXiv:2509.13329)
- **Language**: Rust (edition 2024, requires >= 1.86)
- **License**: MIT
- **Core dependency**: Uses `jagua-rs` (v0.7.0) as its collision detection engine

sparrow is designed as an open, reproducible benchmark baseline for the 2D nesting community. It also compiles to WebAssembly (see sparroWASM).

## 2. Architecture

sparrow is a single crate (binary) with two binaries: `sparrow` (main optimizer) and `bench` (benchmark runner).

### Module Structure

```
src/
  main.rs           -- CLI entry point, parses args, runs optimize()
  lib.rs            -- Crate root, declares modules
  config.rs         -- SparrowConfig, ExplorationConfig, CompressionConfig
  consts.rs         -- GLS weights, coordinate descent params, draw options
  optimizer/
    mod.rs           -- optimize(): top-level two-phase orchestration
    explore.rs       -- exploration_phase(): shrink-and-separate loop
    compress.rs      -- compression_phase(): fine-grained compression
    separator.rs     -- Separator: parallel GLS-based separation engine
    worker.rs        -- SeparatorWorker: per-thread item mover
    lbf.rs           -- LBFBuilder: initial solution via left-bottom-fill
  quantify/
    mod.rs           -- quantify_collision_poly_poly(), quantify_collision_poly_container()
    overlap_proxy.rs -- overlap_area_proxy(): pole-based overlap estimation
    pair_matrix.rs   -- Triangular matrix for pairwise collision loss/weights
    tracker.rs       -- CollisionTracker: GLS weights + collision cache
    simd/            -- Optional SIMD-accelerated overlap computation
  sample/
    mod.rs
    search.rs        -- search_placement(): uniform sampling + coordinate descent
    uniform_sampler.rs -- UniformBBoxSampler: generates random placements
    coord_descent.rs -- refine_coord_desc(): multi-axis coordinate descent
    best_samples.rs  -- BestSamples: top-N unique sample tracker
  eval/
    mod.rs
    sample_eval.rs   -- SampleEval enum (Clear/Collision/Invalid), SampleEvaluator trait
    sep_evaluator.rs -- SeparationEvaluator: weighted loss evaluation for separation
    lbf_evaluator.rs -- LBFEvaluator: left-bottom-fill evaluation
    specialized_jaguars_pipeline.rs -- Custom collision collection with early termination
  util/
    io.rs, svg_exporter.rs, terminator.rs, ctrlc_terminator.rs, listener.rs,
    assertions.rs, bit_reversal_iterator.rs
```

## 3. Algorithm Details

### 3.1 Top-Level Two-Phase Strategy (Algorithm 11)

The `optimize()` function in `optimizer/mod.rs` orchestrates:

1. **Initial Solution**: If no warm-start is provided, an LBF (left-bottom-fill) constructor builds a feasible starting layout. Items are sorted by `convex_hull_area * diameter` (descending) and placed one by one using sampling-based placement search.

2. **Exploration Phase** (default 80% of time): Aggressively shrinks the strip and tries to resolve all collisions. Discovers good strip widths.

3. **Compression Phase** (default 20% of time): Starting from the best feasible solution, attempts fine-grained compression by shrinking by decreasing amounts.

### 3.2 Exploration Phase (Algorithm 12)

`explore.rs::exploration_phase()`:

- Starts from the current strip width.
- Calls `separator.separate()` to try to resolve all collisions (reach zero total loss).
- **On success** (feasible solution found):
  - Records the solution.
  - Shrinks strip by `shrink_step` (default 0.1%).
  - Clears the infeasible solution pool.
- **On failure** (collisions remain):
  - Adds the infeasible solution (with its total loss) to a sorted pool.
  - Selects a solution from the pool using a half-normal distribution (better solutions more likely).
  - **Disrupts** the selected solution by swapping two "large" items (items whose convex hull area is in the top percentile). Items practically contained by the swapped items are repositioned accordingly.
  - Tries separation again.
- Repeats until time runs out or `max_conseq_failed_attempts` is reached.

### 3.3 Compression Phase (Algorithm 13)

`compress.rs::compression_phase()`:

- Starts from the best feasible solution found during exploration.
- Repeatedly attempts to compress:
  1. Restore to the best solution.
  2. Shrink the strip by `r_shrink` percent at a **random horizontal split position** (items to the right of the split are shifted left).
  3. Call `separator.separate()`.
  4. If feasible (loss == 0), record as new best.
  5. If infeasible, increment failure count.
- The shrink step decays over time (time-based linear decay from `shrink_range.0` to `shrink_range.1`, default 0.05% to 0.001%) or by failure count.

### 3.4 Guided Local Search with Dynamic Weights

The **Separator** (`separator.rs`) is the core search engine. It implements Algorithm 9:

- **Strike-based termination**: Runs multiple "strikes" (default 3 in exploration, 5 in compression). Each strike allows up to `iter_no_imprv_limit` iterations without improvement.
- **Each iteration** calls `move_items_multi()` (Algorithm 10):
  - Spawns `n_workers` (default 3) parallel `SeparatorWorker` threads via rayon.
  - Each worker gets a copy of the current state and calls `move_items()` (Algorithm 5).
  - The best result (lowest weighted loss) is selected; all others are discarded.
- **GLS weight updates** (Algorithm 8) after each iteration:
  - For each pair of colliding items and each item-container collision, a weight is maintained.
  - Weights for active collisions are **increased** proportionally to how severe the collision is relative to the worst (multiplier between 1.2x and 2.0x).
  - Weights for resolved collisions **decay** by 0.95x per iteration (toward 1.0).
  - This guides the search away from persistent collisions.

### 3.5 How Items Are Repositioned

`worker.rs::move_items()` (Algorithm 5):

1. Collect all items with loss > 0 in random order.
2. For each colliding item:
   - Create a `SeparationEvaluator` that evaluates candidate positions using **weighted loss** (loss * GLS weight).
   - Call `search_placement()` (Algorithm 6):
     a. **Focused sampling** (25 samples around the item's current bounding box).
     b. **Container-wide sampling** (50 samples uniformly across the strip).
     c. **Pre-refinement**: Take the best N=3 unique samples and refine each via coordinate descent (step: 25% to 2% of item min dimension, rotation: 5 deg to 1 deg).
     d. **Final refinement**: Take the single best and refine further (step: 1% to 0.1% of item min dimension, rotation: 0.5 deg to 0.05 deg).
   - Move the item to the best found position.

**Coordinate descent** (`coord_descent.rs`): Alternates between 5 axes (horizontal, vertical, forward-diagonal, backward-diagonal, rotation wiggle). On improvement the step grows by 1.1x; on failure it shrinks by 0.5x and the axis changes randomly.

### 3.6 How It Uses jagua-rs for Collision Detection

sparrow uses jagua-rs at multiple levels:

- **`SPProblem`/`SPInstance`/`SPSolution`**: jagua-rs's strip packing problem model manages the layout, item placement/removal, strip resizing, and save/restore.
- **`CDEngine`**: Each `Layout` contains a collision detection engine. When items are placed/removed, hazards are registered/deregistered in the quadtree.
- **Surrogate-based fail-fast** (`LBFEvaluator`): During initial construction, surrogate collision detection is used first (cheap), then exact polygon collision.
- **Custom collision pipeline** (`specialized_jaguars_pipeline.rs`): During separation, sparrow uses a specialized version of `CDEngine::collect_poly_collisions()` that:
  - First checks a few large poles for early loss accumulation.
  - Traverses edges in bit-reversed order to maximize early detection.
  - Computes weighted loss incrementally and **terminates early** if the accumulated loss exceeds an upper bound (saving >90% of CPU time on dominated samples).

### 3.7 Collision Quantification

The `quantify` module implements the loss function:

- **Item-item collision** (Algorithm 4): Uses an `overlap_area_proxy` based on surrogate poles (inscribed circles). For each pair of poles from the two shapes, computes the penetration depth and multiplies by the minimum radius. This is scaled by PI and a shape-based penalty (geometric mean of sqrt of convex hull areas).
- **Item-container collision**: Based on how much of the item's bounding box extends outside the container, plus a distance penalty if fully outside.

The `CollisionTracker` stores all pairwise losses in a triangular `PairMatrix` plus container losses, each paired with a GLS weight.

## 4. Input/Output

### Input
- JSON format shared with jagua-rs (items as polygons with allowed rotations, quantities, container dimensions).
- Available from Oscar Oliveira's OR-Datasets repository.
- Can also warm-start from a previous solution JSON.

### Output
- **SVG files**: Visual + exact representation. Items are defined as `<g>` elements with `translate()` and `rotate()` transforms. Saved to `output/final_{name}.svg`.
- **JSON solution**: `output/final_{name}.json` containing the full solution state.
- Optional intermediate SVGs in `output/sols_{name}/`.
- Optional live SVG viewer via `live_svg` feature.

## 5. Strengths

- **State-of-the-art results**: Achieves record or near-record solutions on all standard benchmark instances (swim, trousers, mao, marques, dagli, albano, shirts, gardeyn).
- **Reproducible**: All benchmark solutions are provided with exact reproduction instructions (git commit, seed, toolchain version).
- **Fast**: Highly optimized Rust with optional SIMD (nightly), `target-cpu=native`, fat LTO. Achieves hundreds of thousands of collision evaluations per second.
- **Parallelized**: Multiple separator workers run in parallel via rayon with a local thread pool.
- **Robust**: Extensive `debug_assert!()` checks throughout; integration tests with assertions active.
- **WebAssembly support**: Compiles to WASM for browser-based use.
- **Clean separation of concerns**: Uses jagua-rs for all geometry/collision detection, focuses purely on optimization.
- **Continuous rotation**: Supports continuous item rotation, not just discrete angles.

## 6. Weaknesses

- **Strip packing only**: Only solves SPP (strip packing problems). Does not handle bin packing, knapsack, or multi-strip variants directly.
- **No exact/optimal solver**: Purely heuristic -- no guarantee of optimality.
- **Time-dependent quality**: Solution quality improves with more time; short runs may not produce good results. Default is 600s total.
- **Single container**: Cannot handle problems with multiple containers or heterogeneous bins.
- **No quality zones**: Does not exploit jagua-rs's quality zone features.
- **Parameter sensitivity**: While defaults are well-tuned, the numerous parameters (shrink steps, GLS weights, sample counts, coordinate descent steps) may need tuning for non-standard instances.
- **Memory**: Cloning solutions and layouts frequently for save/restore may be memory-intensive for very large instances.

## 7. Key Code Paths

| Purpose | File |
|---------|------|
| Entry point | `/Users/finn.carlsvi/thetta/research/repos/sparrow/src/main.rs` |
| Two-phase orchestration | `/Users/finn.carlsvi/thetta/research/repos/sparrow/src/optimizer/mod.rs` |
| Exploration phase | `/Users/finn.carlsvi/thetta/research/repos/sparrow/src/optimizer/explore.rs` |
| Compression phase | `/Users/finn.carlsvi/thetta/research/repos/sparrow/src/optimizer/compress.rs` |
| Separator (GLS engine) | `/Users/finn.carlsvi/thetta/research/repos/sparrow/src/optimizer/separator.rs` |
| Per-thread worker | `/Users/finn.carlsvi/thetta/research/repos/sparrow/src/optimizer/worker.rs` |
| Initial solution (LBF) | `/Users/finn.carlsvi/thetta/research/repos/sparrow/src/optimizer/lbf.rs` |
| Placement search | `/Users/finn.carlsvi/thetta/research/repos/sparrow/src/sample/search.rs` |
| Coordinate descent | `/Users/finn.carlsvi/thetta/research/repos/sparrow/src/sample/coord_descent.rs` |
| Collision quantification | `/Users/finn.carlsvi/thetta/research/repos/sparrow/src/quantify/mod.rs` |
| Overlap proxy (poles) | `/Users/finn.carlsvi/thetta/research/repos/sparrow/src/quantify/overlap_proxy.rs` |
| GLS weights + tracking | `/Users/finn.carlsvi/thetta/research/repos/sparrow/src/quantify/tracker.rs` |
| Separation evaluator | `/Users/finn.carlsvi/thetta/research/repos/sparrow/src/eval/sep_evaluator.rs` |
| Custom jagua-rs pipeline | `/Users/finn.carlsvi/thetta/research/repos/sparrow/src/eval/specialized_jaguars_pipeline.rs` |
| Configuration + defaults | `/Users/finn.carlsvi/thetta/research/repos/sparrow/src/config.rs` |
| Constants (GLS, CD) | `/Users/finn.carlsvi/thetta/research/repos/sparrow/src/consts.rs` |
