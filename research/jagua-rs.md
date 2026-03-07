# jagua-rs -- Fast Collision Detection Engine for 2D Nesting

## 1. Overview

**jagua-rs** is a collision detection engine (CDE) designed as a reusable backend for optimization algorithms that solve 2D irregular cutting and packing (nesting) problems. Its purpose is to decouple the geometric feasibility problem from the combinatorial optimization problem.

- **Author**: Jeroen Gardeyn (KU Leuven, CODeS research group)
- **Paper**: *"Decoupling Geometry from Optimization in 2D Irregular Cutting and Packing Problems: an Open-Source Collision Detection Engine"* (INFORMS Journal on Computing, DOI: 10.1287/ijoc.2024.1025; freely available at arXiv:2508.08341)
- **Language**: Rust (edition 2024), published on crates.io as `jagua-rs` v0.7.0
- **License**: MPL-2.0
- **Key claim**: Resolves millions of collision queries per second

The library answers the central question: *"Can I place this item here without causing any collisions? And if not, which entities would collide with it?"*

## 2. Architecture

### Workspace Structure

The repository is a Cargo workspace with two crates:

```
jagua-rs/          -- The core library (collision detection engine)
  Cargo.toml       -- Features: spp, bpp, mspp
  src/
    lib.rs
    collision_detection/   -- CDEngine, quadtree, hazards
    entities/              -- Layout, Item, Container, PlacedItem
    geometry/              -- Primitives, transformations, fail-fast surrogates
    io/                    -- JSON import/export, SVG generation
    probs/                 -- Problem variants (spp/, bpp/, mspp/)
    util/                  -- Assertions, floating-point helpers

lbf/               -- Reference optimizer (left-bottom-fill heuristic)
  src/
    main.rs, lib.rs
    opt/            -- LBF algorithm for SPP and BPP
    samplers/       -- Uniform and local-search samplers
    config.rs       -- LBFConfig
  benches/          -- Performance benchmarks (ci_bench, quadtree, fail_fast, edge_sensitivity)
```

### Key Modules in `jagua-rs`

| Module | Purpose |
|--------|---------|
| `collision_detection::CDEngine` | Central engine: registers/deregisters hazards, answers detect/collect queries |
| `collision_detection::quadtree` | Spatial indexing via adaptive quadtree |
| `collision_detection::hazards` | Hazard model, filters, collectors |
| `entities::Layout` | Dynamic placement state with integrated CDE |
| `entities::Item` | Item definition with original and CD shapes |
| `entities::Container` | Container with base CDE, holes, quality zones |
| `geometry::primitives` | Point, Edge, Rect, Circle, SPolygon |
| `geometry::fail_fast` | SPSurrogate (poles + piers), pole/pier generation |
| `geometry::transformation` | 2D affine transformations (rotation + translation) |
| `probs::spp` | Strip Packing Problem: SPInstance, SPProblem, SPSolution |
| `probs::bpp` | Bin Packing Problem variant |
| `probs::mspp` | Multi-Strip Packing Problem variant |
| `io` | JSON import/export, SVG rendering |

## 3. Algorithm Details

### 3.1 Trigonometric Approach (No NFP)

Unlike traditional nesting engines that precompute No-Fit Polygons (NFPs), jagua-rs uses a **direct trigonometric approach**:

- **Edge-edge intersection tests**: The core collision primitive. Two edges (line segments) are tested for intersection using the parametric line-line intersection formula from linear algebra. The `Edge::collides_with(&Edge)` method computes parameters `t` and `u` along each edge; intersection occurs when both are in `[0, 1]`.
- **Point-in-polygon**: Uses ray casting (horizontal ray to the right, counting edge intersections) to determine containment.
- **All geometric entities are tested directly** against each other -- there is no precomputation of NFPs or Minkowski sums.

This approach is:
- **Insensitive to shape complexity**: Performance does not degrade with more vertices (thanks to the quadtree).
- **Supports continuous rotation**: No need to precompute NFPs for each rotation angle.
- **Numerically robust**: Careful handling of floating-point edge cases (vertices on rays, near-parallel edges, etc.) using `FPA` (floating-point-aware) comparisons.

### 3.2 Surrogates (Fail-Fast Collision Detection)

Each `SPolygon` can have an `SPSurrogate` -- a lightweight internal approximation consisting of:

**Poles** (inscribed circles):
- The first pole is the Pole of Inaccessibility (PoI) -- the largest inscribed circle, computed via a quadtree-based search inspired by Mapbox's `polylabel` algorithm.
- Additional poles are generated iteratively: each new pole is the largest circle that fits inside the polygon while being outside all existing poles.
- Generation is controlled by `n_pole_limits`: tuples of `(max_poles, coverage_threshold)`. For example, `[(64, 0.0), (16, 0.8), (8, 0.9)]` means: generate up to 64 poles if coverage < 80%, stop at 16 if coverage >= 80%, stop at 8 if coverage >= 90%.
- Poles serve dual purposes:
  1. **Fail-fast detection**: The first `n_ff_poles` are tested for collisions before checking edges. A circle-based collision check is much cheaper than edge intersection.
  2. **Overlap quantification** (used by sparrow): The full set of poles provides a proxy for overlap area between two shapes.

**Piers** (inscribed line segments):
- Line segments fully contained within the polygon, generated to cover areas poorly represented by poles.
- Created by casting rays at multiple angles across the shape, clipping them to the polygon boundary, then greedily selecting those that minimize a loss function measuring "uncovered" area.
- Used alongside poles for fail-fast collision detection (`n_ff_piers` controls how many are checked).

**Configuration** (`SPSurrogateConfig`):
```rust
SPSurrogateConfig {
    n_pole_limits: [(64, 0.0), (16, 0.8), (8, 0.9)],  // pole generation limits
    n_ff_poles: 1,   // poles checked during fail-fast
    n_ff_piers: 0,   // piers checked during fail-fast
}
```

### 3.3 Quadtree-Based Spatial Indexing

The CDE uses an adaptive quadtree (`QTNode`) for spatial indexing:

**Structure**:
- Each node has a bounding box, a level (0 = leaf), optional 4 children (Cartesian quadrants), and a list of `QTHazard`s.
- Maximum depth is configurable (`quadtree_depth`, default 4-5).
- Nodes expand (generate children) lazily: only when a `Partial` hazard is registered.
- Nodes collapse (drop children) when no partial hazards remain.

**Hazard presence in nodes** (`QTHazPresence`):
- `None`: The hazard has no effect in this node.
- `Entire`: The hazard covers the entire node. Any entity in this node automatically collides.
- `Partial(QTHazPartial)`: The hazard's edges partially overlap the node. The `QTHazPartial` stores the subset of edges active in this node plus a tight bounding box for fail-fast rejection.

**Hazard constriction**: When a hazard is registered, it is propagated down the tree. At each level, the hazard is constricted to each child's bounding box:
- Edges are tested against each quadrant; only intersecting edges are kept.
- Quadrants with no intersecting edges are classified as `Entire` or `None` based on containment checks (using centroid point-in-polygon test), with neighbor inference to avoid redundant checks.

**Collision detection** (`QTNode::collides()`):
1. Check the strongest hazard in the node.
2. If `Entire`: immediate collision.
3. If `Partial`: either recurse into children (if they exist and edge count exceeds `cd_threshold`) or test the entity against each hazard's edges directly.
4. `cd_threshold` (default 16): if a node has fewer than this many active edges, perform edge tests directly rather than recursing further.

**Collision collection** (`QTNode::collect_collisions()`):
- Similar traversal, but collects *all* colliding hazards rather than stopping at the first.
- Uses `HazardCollector` trait to avoid re-checking already-detected hazards.

**Virtual root optimization**: For a given shape's bounding box, the CDE finds the lowest quadtree node that fully contains it (`get_virtual_root()`), avoiding unnecessary traversal from the actual root.

### 3.4 How It Achieves Millions of Queries/Second

The combination of several techniques:

1. **Quadtree spatial partitioning**: Narrows the search space dramatically. Only edges near the query shape are tested.
2. **Virtual root**: Starts collision checks from the deepest containing node, skipping upper levels.
3. **Surrogate fail-fast**: Cheap circle/edge tests reject most placements before expensive polygon tests.
4. **Bounding box fail-fast**: `QTHazPartial` stores a tight bounding box; entities not colliding with it skip all edge tests for that hazard.
5. **`cd_threshold`**: Avoids the overhead of further tree traversal when few edges remain.
6. **Hazard collector deduplication**: Once a hazard is detected, it is skipped in subsequent checks.
7. **Edge-quadrant optimization**: The `Edge::collides_with_quadrants()` method efficiently determines which quadrants an edge intersects using shared-edge analysis (bisectors), often resolving all 4 quadrants with 2-3 intersection tests instead of 4 independent checks.
8. **Snapshot/restore via diff**: `CDEngine::restore()` computes the diff between current and target state, only adding/removing changed hazards rather than rebuilding from scratch.
9. **Pure Rust, no allocations on hot path**: Core collision detection methods operate on stack-allocated primitives.

## 4. API

### Core Types

```rust
// Create engine
let cde = CDEngine::new(bbox, static_hazards, config);

// Register/deregister dynamic hazards (placed items)
cde.register_hazard(hazard);
cde.deregister_hazard_by_entity(entity);

// Binary collision detection
let collides: bool = cde.detect_poly_collision(&shape, &filter);
let collides: bool = cde.detect_surrogate_collision(&surrogate, &transform, &filter);

// Collision collection (find all colliding hazards)
cde.collect_poly_collisions(&shape, &mut collector);
cde.collect_surrogate_collisions(&surrogate, &transform, &mut collector);

// Containment check
let contained: bool = cde.detect_containment_collision(&shape, &haz_shape, entity);

// Save/restore
let snapshot = cde.save();
cde.restore(&snapshot);
```

### Layout API (Higher Level)

```rust
let mut layout = Layout::new(container);
let pk: PItemKey = layout.place_item(&item, d_transformation);
let removed: PlacedItem = layout.remove_item(pk);
let cde: &CDEngine = layout.cde();
let snapshot = layout.save();
layout.restore(&snapshot);
```

### Problem API (Strip Packing)

```rust
let mut prob = SPProblem::new(instance);
prob.place_item(SPPlacement { item_id, d_transf });
prob.remove_item(pk);
prob.change_strip_width(new_width);
prob.fit_strip();
let sol: SPSolution = prob.save();
prob.restore(&sol);
```

### Hazard Filters

Filters allow specific hazards to be ignored during queries:
- `NoFilter`: Nothing is ignored.
- `HazKey` (as filter): Ignores the hazard with that key (used to exclude "self" when checking an item's collisions).
- `HazKeyFilter`: Ignores a set of hazard keys.
- Any `HazardCollector`: Already-detected hazards are skipped.

### Hazard Entities

```rust
enum HazardEntity {
    PlacedItem { id, dt, pk },  // Another item in the layout
    Exterior,                    // Outside the container boundary
    Hole { idx },               // A hole in the container
    InferiorQualityZone { quality, idx },  // Quality zone
}
```

## 5. Strengths and Weaknesses

### Strengths

- **Very fast**: Millions of collision queries per second on standard hardware; performance is tracked via CI benchmarks.
- **No NFP precomputation**: Avoids the combinatorial explosion of precomputing NFPs for all item pairs and rotations. Supports continuous rotation natively.
- **Shape-agnostic**: Performance is insensitive to polygon complexity thanks to quadtree partitioning.
- **Clean abstraction**: Hazards unify all spatial constraints (items, container boundary, holes, quality zones) into a single model.
- **Adaptable**: New problem variants can be added by implementing `Instance`, `Problem`, and `Solution`. Custom constraints via new `Hazard` types and `HazardFilter`s.
- **Robust floating-point handling**: Careful treatment of edge cases with `FPA` comparisons, `almost_relation_to()` for bounding box checks, special handling for ray-casting through vertices.
- **Full save/restore**: Efficient snapshot mechanism enables optimization algorithms to freely explore and backtrack.
- **WebAssembly support**: Compiles to WASM for browser-based applications.
- **Polygon simplification**: Integrated preprocessor simplifies complex polygons while preserving feasibility guarantees.
- **Well-documented**: Extensive rustdoc with auto-deployed documentation.
- **Proven in practice**: Used as the engine behind sparrow, the current state-of-the-art nesting heuristic.

### Weaknesses

- **No minimum-distance proximity grid**: Unlike some engines, jagua-rs does not provide a "hazard proximity grid" that estimates the closest distance to any hazard at arbitrary points. The README and docs mention this as a feature in the SVG options (`hazard_proximity_grid`) for visualization, but it is not a core query primitive exposed to the optimizer.
- **Approximate surrogates**: Pole-based surrogates are an approximation. Items with long thin features may be poorly represented by circles, reducing fail-fast effectiveness (though piers help).
- **Quadtree rebuild on container change**: When the strip width changes, the layout's CDE is rebuilt from scratch (all hazards re-registered), which can be expensive for large instances.
- **No incremental edge tests**: When an item moves slightly, the entire collision check is re-run. There is no incremental update for small perturbations.
- **Single-threaded CDE**: The collision detection engine itself is not thread-safe; parallelism must be achieved by cloning (as sparrow does with its workers).
- **Memory overhead**: Each `Layout` clone includes the full quadtree and hazard map. Multiple parallel workers multiply this cost.
- **No 3D support**: Purely 2D.

## 6. Key Code Paths

| Purpose | File |
|---------|------|
| CDE: engine, config, snapshot | `/Users/finn.carlsvi/thetta/research/repos/jagua-rs/jagua-rs/src/collision_detection/cd_engine.rs` |
| Quadtree node (detect + collect) | `/Users/finn.carlsvi/thetta/research/repos/jagua-rs/jagua-rs/src/collision_detection/quadtree/qt_node.rs` |
| Quadtree hazard + presence | `/Users/finn.carlsvi/thetta/research/repos/jagua-rs/jagua-rs/src/collision_detection/quadtree/qt_hazard.rs` |
| Partial hazard (edge subset) | `/Users/finn.carlsvi/thetta/research/repos/jagua-rs/jagua-rs/src/collision_detection/quadtree/qt_partial_hazard.rs` |
| Edge/Circle quadrant queries | `/Users/finn.carlsvi/thetta/research/repos/jagua-rs/jagua-rs/src/collision_detection/quadtree/qt_traits.rs` |
| Hazard model | `/Users/finn.carlsvi/thetta/research/repos/jagua-rs/jagua-rs/src/collision_detection/hazards/hazard.rs` |
| Hazard filters | `/Users/finn.carlsvi/thetta/research/repos/jagua-rs/jagua-rs/src/collision_detection/hazards/filter.rs` |
| Hazard collectors | `/Users/finn.carlsvi/thetta/research/repos/jagua-rs/jagua-rs/src/collision_detection/hazards/collector.rs` |
| Layout (place/remove/save/restore) | `/Users/finn.carlsvi/thetta/research/repos/jagua-rs/jagua-rs/src/entities/layout.rs` |
| Item definition | `/Users/finn.carlsvi/thetta/research/repos/jagua-rs/jagua-rs/src/entities/item.rs` |
| SPolygon (vertices, edges, POI) | `/Users/finn.carlsvi/thetta/research/repos/jagua-rs/jagua-rs/src/geometry/primitives/simple_polygon.rs` |
| SPSurrogate (poles + piers) | `/Users/finn.carlsvi/thetta/research/repos/jagua-rs/jagua-rs/src/geometry/fail_fast/sp_surrogate.rs` |
| Pole generation (PoI algorithm) | `/Users/finn.carlsvi/thetta/research/repos/jagua-rs/jagua-rs/src/geometry/fail_fast/pole.rs` |
| Pier generation | `/Users/finn.carlsvi/thetta/research/repos/jagua-rs/jagua-rs/src/geometry/fail_fast/piers.rs` |
| SPP problem model | `/Users/finn.carlsvi/thetta/research/repos/jagua-rs/jagua-rs/src/probs/spp/entities/problem.rs` |
| Library root | `/Users/finn.carlsvi/thetta/research/repos/jagua-rs/jagua-rs/src/lib.rs` |
| LBF reference optimizer | `/Users/finn.carlsvi/thetta/research/repos/jagua-rs/lbf/src/opt/lbf_spp.rs` |
| CI benchmarks | `/Users/finn.carlsvi/thetta/research/repos/jagua-rs/lbf/benches/ci_bench.rs` |
