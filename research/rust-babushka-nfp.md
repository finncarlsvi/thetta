# Rust-Babushka-NFP Analysis

## 1. Overview

Babushka is a Rust library for 2D geometry algorithms focused on bin packing and nesting. Published on crates.io as `babushka` (v0.1.10), it provides 2D geometry primitives, a No-Fit Polygon (NFP) implementation inspired by SVGNest, hierarchical piece representations (polygons with holes), and optional rasterization. The library is authored by davefol. It is a relatively small, focused library compared to U-Nesting.

## 2. Architecture

Single-crate Rust library with flat module structure:

```
src/
  lib.rs              - Module declarations
  point.rs            - Point2D trait (generic over numeric type)
  segment.rs          - Segment trait with distance/intersection methods
  polygon.rs          - Polygon trait (vertices, segments, area, containment, intersection)
  bounding_box.rs     - AABB bounding box
  multi_polygon.rs    - MultiPolygon (polygon with holes - "piece")
  polygon_graph.rs    - Graph-based polygon representation via petgraph
  no_fit_polygon.rs   - NFP computation (orbiting/sliding algorithm)
  clip.rs             - Polygon clipping operations
  utils.rs            - Utility functions
  kernelf64/          - Concrete f64 implementations
    mod.rs
    point2d.rs        - f64 Point2D
    polygon.rs        - f64 Polygon with offset/rotation support
    segment.rs        - f64 Segment
  nesting/
    mod.rs            - Nesting module
    genetic.rs        - Genetic algorithm for irregular bin packing
    problem/
      mod.rs
      irregular_bin_packing.rs  - Problem/solution types
  parsers/
    mod.rs
    terashima.rs      - Terashima test file format parser
  raster.rs           - Optional rasterization (behind "raster" feature)
examples/             - Multiple visualization examples (NFP, GIF output, etc.)
build.rs              - Uses cc + bindgen (C library binding, likely Clipper)
```

**Key files:**
- `src/no_fit_polygon.rs` - Core NFP algorithm (~785 lines)
- `src/nesting/genetic.rs` - GA-based bin packing solver
- `src/polygon.rs` - Polygon trait definitions
- `src/kernelf64/polygon.rs` - Concrete polygon with offset, rotation, translation

## 3. Algorithm Details

### NFP Computation (Orbiting/Sliding Algorithm)
The NFP implementation follows SVGNest's approach:
1. **Start point selection**: Place orbiting polygon so its max-Y vertex touches stationary polygon's min-Y vertex (exterior NFP) or search for interior start points
2. **Touching detection**: Three types - vertex-vertex (Type A), point-on-edge (Type B), edge-on-point (Type C)
3. **Vector generation**: From touching points, generate candidate translation vectors from both polygon edges
4. **Slide distance**: Compute maximum safe slide distance along each vector using `slide_distance_on_polygon()`
5. **Translation selection**: Choose vector with maximum slide distance
6. **Loop detection**: Track reference point; terminate when it returns to start
7. **Multiple NFPs**: If `search_edges=true`, searches for additional NFP components (interior holes)

The algorithm handles:
- Convex and non-convex polygons
- Inside NFP (for container fitting)
- Rotation (via polygon offset/rotation transformations)

### Genetic Algorithm
`GeneticIrregularBinPacker`:
- Chromosome: order (permutation of piece indices) + rotation per piece
- Initial population seeded by area-decreasing heuristic then mutated
- Mutation: adjacent swap (0.01 * mutation_rate probability) + random rotation change
- **Placement function is incomplete** (`place()` ends with `unimplemented!()`) - only first piece placement is coded
- NFP cache by (piece pair, rotations, inside flag)
- Builder pattern for construction

### Problem Representation
- `IrregularBinPackingProblem`: bin (MultiPolygon) + piece descriptions
- `PieceDescription`: piece (MultiPolygon), allowed rotations, instance count
- `IrregularBinPackingSolution`: list of placements (bin_id, piece_id, location, rotation)

## 4. Dependencies

- **anyhow** 1.0 - Error handling with backtraces
- **approx** 0.5 - Approximate floating-point comparison (critical for NFP)
- **itertools** 0.13 - Iterator combinatorics (cartesian_product for touching detection)
- **num-traits** 0.2 - Generic numeric traits
- **petgraph** 0.6 - Graph data structure for polygon representation
- **libc** 0.2 - C interop
- **rand** 0.8 + rand_chacha 0.3 - Deterministic RNG

Build deps: **cc** 1.0 + **bindgen** 0.65 (C library compilation, likely for Clipper)

Optional: **minifb** (window display), **font8x8** (rasterization text), **gif** (GIF output for examples)

## 5. Strengths

- **Generic numeric type support** - Point2D/Polygon traits are generic over any Float type, not locked to f64
- **Clean NFP implementation** - Direct port of SVGNest's well-tested orbiting algorithm with clear structure
- **Good test coverage** - Unit tests for NFP with specific expected coordinates including rotated cases
- **Hierarchical pieces** - MultiPolygon supports polygons with holes (inner contours)
- **Terashima format support** - Can parse standard nesting benchmark files
- **Visualization examples** - Multiple examples with GIF output showing NFP computation visually
- **Deterministic RNG** - ChaCha8Rng for reproducible results

## 6. Weaknesses

- **Incomplete GA solver** - The `place()` function in `genetic.rs` is unfinished (`unimplemented!()`) - this is a critical gap; the library cannot actually solve nesting problems end-to-end
- **No parallelism** - Single-threaded only; no rayon or async
- **O(n*m) touching detection** - The NFP algorithm uses cartesian_product of all segment pairs, which is O(n*m) per iteration
- **Limited placement strategies** - Only the incomplete GA; no BLF, SA, or other heuristics
- **No boundary handling** - The solver doesn't handle sheet/boundary fitting (IFP) properly
- **Small library** - Minimal features beyond NFP computation
- **Older dependencies** - rand 0.8 (current is 0.9), bindgen 0.65
- **No documentation on crates.io** - docs.rs badge points to num-traits, not babushka

## 7. Key Code Paths

**NFP computation:**
`ComputeNoFitPolygon::no_fit_polygon()` in `src/no_fit_polygon.rs`:
1. Normalize offsets to zero
2. Find start point (min-Y of stationary - max-Y of orbiting for exterior)
3. Loop: detect touchings -> generate vectors -> compute slide distances -> translate -> check loop closure
4. Search for additional NFP components if `search_edges=true`

**GA setup:**
`GeneticIrregularBinPacker::new()` in `src/nesting/genetic.rs`:
1. Sort pieces by area (descending)
2. Create adam individual with sorted order + random rotations
3. Populate by mutating adam

**Polygon operations:**
- `Polygon::intersects_polygon()` - segment intersection checking
- `Polygon::slide_distance_on_polygon()` - safe slide distance computation
- `Point2D::in_polygon()` - point-in-polygon test
- `Polygon::area()` - signed area via shoelace formula
