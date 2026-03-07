# U-Nesting Analysis

## 1. Overview

U-Nesting is a high-performance, domain-agnostic 2D/3D spatial optimization engine written in Rust. It provides polygon nesting and 3D bin packing algorithms as a pure computation library with no domain-specific logic. It is published on crates.io (v0.2.0), built by iyulab, and designed to be consumed by applications in manufacturing, textiles, logistics, etc. It includes a C FFI layer for integration with C#, Python, and other languages.

## 2. Architecture

Workspace-based Rust monorepo with 8 crates:

```
crates/
  core/         - Shared abstractions: traits, GA/SA/BRKGA/ALNS/GDRR frameworks, geometry types, transforms
  d2/           - 2D nesting: polygon geometry, NFP computation, BLF/NFP-guided/GA/BRKGA/SA/ALNS/GDRR nesters
  d3/           - 3D bin packing: box/mesh geometry, extreme-point/GA/SA packing, physics (gravity/stability)
  cutting/      - Cutting path optimization: contour extraction, TSP-based sequencing, thermal modeling, kerf/lead-in
  ffi/          - C FFI: JSON-based API, progress callbacks, exports unesting_solve_2d / unesting_solve_3d
  python/       - PyO3-based Python bindings
  u-nesting/    - Umbrella re-export crate (the public API)
  benchmark/    - Benchmarking harness with dataset support
bindings/
  csharp/       - C# wrapper
```

**Key files:**
- `crates/core/src/ga.rs` - GA framework with configurable population, crossover rate, mutation, elitism, tournament selection
- `crates/d2/src/nfp.rs` - NFP computation: Minkowski sum (convex), convex decomposition + union (non-convex via i_overlay), and sliding algorithm (Burke 2007)
- `crates/d2/src/nester.rs` - Main solver dispatching to BLF, NFP-guided, GA, BRKGA, SA, GDRR, ALNS strategies
- `crates/d2/src/ga_nesting.rs` - GA-based nesting with NestingChromosome (permutation order + rotation indices), OX1 crossover
- `crates/ffi/src/api.rs` - C FFI: `unesting_solve_2d`, `unesting_solve_3d` with JSON I/O and progress callbacks
- `crates/cutting/src/lib.rs` - Cutting path optimization pipeline (contour -> hierarchy -> pierce -> TSP sequence)

## 3. Algorithm Details

### 2D Nesting
Seven placement strategies selectable via `Strategy` enum:

1. **BLF (Bottom-Left Fill)** - Greedy row-based placement with rotation optimization. Fastest (~338ms for 50 pieces), lowest quality (~60%).
2. **NFP-Guided** - Computes NFP between all polygon pairs to find collision-free bottom-left positions. Uses IFP (Inner Fit Polygon) for boundary containment.
3. **GA (Genetic Algorithm)** - Permutation chromosome encoding (placement order + rotation index per instance). OX1 crossover, swap/rotation mutation. Evaluated via NFP-guided placement decoding. Best overall quality (~70.6%).
4. **BRKGA (Biased Random-Key GA)** - Random-key chromosome with elite inheritance.
5. **SA (Simulated Annealing)** - Temperature-based neighborhood search with permutation solutions.
6. **GDRR (Goal-Driven Ruin & Recreate)** - Local search with random restart diversification.
7. **ALNS (Adaptive Large Neighborhood Search)** - Destroy-repair with operator selection.

NFP computation uses three methods:
- Minkowski sum for convex polygons (O(n+m))
- Convex decomposition + union via `i_overlay` for non-convex
- Sliding/orbiting algorithm (Burke et al. 2007) for complex shapes

NFP caching via `NfpCache` (Arc<RwLock<HashMap>>).

### 3D Bin Packing
- Extreme Point heuristic for fast placement
- Layer-based bottom-up packing
- GA optimization for sequence and rotation
- Physics constraints: gravity simulation, stability checking

### Cutting Path Optimization
- Contour extraction from placed geometries
- Hierarchy DAG for precedence (holes before exteriors)
- Pierce point optimization
- TSP-based sequence with nearest-neighbor + constrained 2-opt
- Thermal modeling to avoid heat concentration

## 4. Dependencies

- **i_overlay** 1.9 - Boolean polygon operations (NFP union)
- **rstar** 0.12 - R-tree spatial indexing
- **rayon** 1.10 - Parallel computation
- **serde/serde_json** - JSON serialization
- **rand** 0.9 - Randomness
- **good_lp** 1.10 - MILP solver (optional)
- **pyo3** 0.24 - Python bindings
- **libc** 0.2 - C FFI
- **criterion** 0.5 - Benchmarking
- Relies on companion libraries: u-metaheur (metaheuristic frameworks), u-geometry (computational geometry primitives)

Rust 1.82+, edition 2021. Release profile uses LTO + single codegen unit.

## 5. Strengths

- **Comprehensive algorithm portfolio** - Seven 2D strategies + three 3D strategies, allowing users to trade speed vs. quality
- **Clean architecture** - Well-separated concerns (core traits, 2D, 3D, cutting, FFI), trait-based abstractions
- **Multiple NFP methods** - Minkowski sum, decomposition, and sliding for robust handling of convex/non-convex shapes
- **Cross-language support** - C FFI, Python bindings, C# wrapper with JSON-based API
- **Production features** - Progress callbacks, cancellation, time limits, validation/filtering of out-of-bounds placements
- **Cutting path optimization** - Unique module for post-nesting CNC/laser path optimization
- **3D support** - Unusual for nesting libraries; includes physics-based constraints
- **Performance** - Parallel evaluation via Rayon, LTO-optimized release builds, NFP caching

## 6. Weaknesses

- **External library dependencies** - Relies on u-metaheur and u-geometry which are separate packages (coupling risk)
- **Complexity** - 8 crates, many algorithm variants may be hard to maintain
- **NFP for complex shapes** - Non-convex NFP via decomposition + union can be fragile at edge cases
- **BLF quality** - The simple BLF is a naive row packer, not a true bottom-left-fill with NFP guidance
- **Documentation** - Code-level docs are good but no published academic analysis of solution quality vs. benchmarks
- **No WASM target** - FFI is C-only; no web deployment path
- **Version 0.2.0** - Still early; API may not be stable

## 7. Key Code Paths

**2D nesting entry point:**
`Nester2D::solve()` in `crates/d2/src/nester.rs` -> dispatches by Strategy enum -> e.g. `run_ga_nesting()` in `crates/d2/src/ga_nesting.rs`

**NFP computation:**
`compute_nfp()` in `crates/d2/src/nfp.rs` -> checks convexity -> Minkowski sum or decomposition + union or sliding

**GA evaluation:**
`NestingChromosome` (order permutation + rotation indices) -> decoded by placing pieces in order using NFP-guided bottom-left placement -> fitness = utilization * placed_ratio

**FFI:**
`unesting_solve_2d()` in `crates/ffi/src/api.rs` -> parses JSON request -> constructs geometries/boundary/config -> calls Nester2D::solve -> returns JSON response

**Cutting path:**
`optimize_cutting_path()` in `crates/cutting/src/path.rs` -> contour extraction -> hierarchy -> pierce points -> TSP sequence -> path assembly
