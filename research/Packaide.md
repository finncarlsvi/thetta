# Packaide

## Overview

A fast, exact-geometry 2D nesting library built in C++17 with CGAL and exposed to Python via Boost.Python. Packaide computes No-Fit Polygons using CGAL's Minkowski sum implementation, guaranteeing exact arithmetic through the `Exact_predicates_exact_constructions_kernel`. It takes SVG files as input, discretizes curves into polygons, applies configurable dilation for spacing, and packs pieces onto rectangular sheets using a decreasing-area first-fit heuristic. The library supports NFP caching and polygon persistence across runs, making it suitable for interactive/iterative nesting workflows.

## Architecture

```
.
+-- include/packaide/
|   +-- packing.hpp            # Core packing engine (~245 lines)
|   +-- no_fit_polygon.hpp     # NFP via CGAL Minkowski sums (~135 lines)
|   +-- primitives.hpp         # Data types and CandidatePoints (~219 lines)
|   +-- persistence.hpp        # NFP/polygon cache and state (~92 lines)
+-- src/
|   +-- bindings.cpp           # Boost.Python bindings (~177 lines)
+-- python/packaide/
|   +-- packaide.py            # Python frontend: SVG parsing, pack() (~307 lines)
|   +-- __init__.py
+-- CMakeLists.txt             # Build config: C++17, CGAL, Boost.Python
+-- setup.py                   # Python package setup
```

### Key Files

- **`include/packaide/packing.hpp`**: The core engine. `pack_decreasing()` sorts polygons by bounding box area (largest first), then calls `pack_polygons_ordered_first_fit()` which tries to place each polygon on existing sheets before creating new ones. For each placement, it computes candidate points from the Inner Fit Polygon (IFP) minus the union of all NFPs with already-placed polygons, then selects the point minimizing an `IncrementalBoundingBoxHeuristic` (minimizes bounding box expansion).
- **`include/packaide/no_fit_polygon.hpp`**: Computes NFPs via CGAL `minkowski_sum_by_full_convolution_2()`. The IFP (inner-fit polygon for the rectangular sheet) is computed as a Minkowski sum of the negated polygon with the sheet rectangle. Supports rotation by computing NFPs at each discrete rotation angle.
- **`include/packaide/primitives.hpp`**: Defines core types (`Point`, `Polygon`, `PolygonWithHoles`, `Transform`, `Sheet`, `Placement`). The `CandidatePoints` class uses CGAL's `Polygon_set_2` to perform exact boolean operations (IFP minus NFP unions) and extract feasible placement vertices.
- **`include/packaide/persistence.hpp`**: Implements NFP caching with `NFPCacheKey` (hashed pair of polygon indices + rotation) and a `State` struct holding canonical polygon representations and the NFP cache. Persists across packing calls.
- **`python/packaide/packaide.py`**: Python-level interface. Parses SVGs using `svgelements`, discretizes all curves (arcs, cubic/quadratic Beziers, lines) into polylines at a configurable tolerance, applies Shapely `buffer()` for dilation (offset/spacing), then calls the C++ packing engine. Handles overapproximation by dilating/eroding to ensure packed pieces never overlap.

## Algorithm Details

### Packing Strategy: Decreasing-Area First-Fit

1. **Sort**: All polygons sorted by bounding box area, largest first. This is a classic bin-packing heuristic that improves space utilization.
2. **First-Fit Sheet Assignment**: For each polygon, try to place it on each existing sheet. If no sheet can accommodate it, create a new sheet.
3. **Candidate Point Generation**: For a given sheet with already-placed polygons:
   - Compute the Inner Fit Polygon (IFP) -- the locus of valid reference-point positions keeping the polygon inside the sheet.
   - Compute NFPs between the new polygon and each placed polygon.
   - Subtract the union of all NFPs from the IFP using CGAL exact polygon set operations.
   - The vertices of the resulting polygon set are the candidate placement points.
4. **Heuristic Selection**: Among candidate points, select the one minimizing the `IncrementalBoundingBoxHeuristic` -- the increase in the bounding box of all placed polygons. This encourages compact layouts.

### NFP Computation (Minkowski Sum)

NFPs are computed via CGAL's `minkowski_sum_by_full_convolution_2()`:
- For two polygons A and B, `NFP(A, B) = A (+) (-B)` where `(+)` is the Minkowski sum and `-B` is B reflected through the origin.
- The IFP for a rectangular sheet is computed similarly: `IFP(sheet, P) = sheet (+) (-P)`.
- Uses CGAL's exact kernel, guaranteeing no floating-point errors in intersection/containment tests.
- NFPs are cached by polygon pair and rotation angle in `persistence.hpp`.

### Rotation Handling

- The user specifies a number of discrete rotations (e.g., 4 means 0, 90, 180, 270 degrees).
- For each polygon, the packing engine tries all rotation variants and selects the one yielding the best placement.
- NFPs are pre-computed for each rotation combination.

### SVG Processing and Overapproximation

The Python frontend handles input geometry:
1. **SVG Parsing**: Uses `svgelements` to extract paths, converting Arc, CubicBezier, QuadraticBezier, and Line segments into polyline approximations at the specified tolerance.
2. **Dilation for Spacing**: Applies `shapely.buffer(offset)` to each polygon, creating the required spacing between parts.
3. **Overapproximation Guarantee**: When `overapproximate=True` (default), the discretized polygon is guaranteed to contain the original curved shape. This is achieved by dilating before discretization, ensuring no material collision even with approximation error.
4. **Polygon Validation**: Ensures correct winding order (CCW for outer boundary, CW for holes) and handles polygon simplification.

### Caching and Persistence

- **NFP Cache**: Stores computed NFPs keyed by `(polygon_index_A, polygon_index_B, rotation)`. Avoids redundant Minkowski sum computations across iterations.
- **Canonical Polygon Cache**: Maps input polygons to canonical (centered, normalized) forms for consistent NFP lookup.
- **State Object**: A `State` struct persists between `pack()` calls, enabling incremental workflows where the user adjusts parameters without recomputing all NFPs.

## Dependencies

- **C++17 compiler** (GCC 7+ or Clang 5+)
- **CGAL** -- exact geometry kernel, Minkowski sums, polygon set operations
- **Boost.Python** -- C++ to Python bindings
- **Python 3.6+**
- **svgelements** -- SVG file parsing
- **shapely** -- polygon buffering (dilation/erosion) for spacing
- **numpy** -- coordinate array handling

## Strengths

- **Exact geometry**: CGAL's exact kernel eliminates floating-point errors in NFP computation, boolean operations, and containment checks. This is a significant advantage over floating-point-based implementations that suffer from numerical degeneracies.
- **Fast C++ core**: All heavy computation (Minkowski sums, polygon set operations, candidate point generation) runs in compiled C++, orders of magnitude faster than pure Python.
- **NFP caching with persistence**: The `State` object preserves computed NFPs across calls, making iterative/interactive use efficient. Users can adjust parameters without full recomputation.
- **Overapproximation guarantees**: The dilation-based approach ensures that packed pieces with curved boundaries never actually overlap, a correctness guarantee most nesting libraries lack.
- **Clean architecture**: Header-only C++ with clear separation between geometry primitives, NFP computation, packing logic, and persistence. The Python layer handles I/O and preprocessing cleanly.
- **SVG I/O**: Accepts industry-standard SVG files and produces SVG output, making it interoperable with CAD tools.
- **Supports holes**: Both stock sheets and pieces can have holes (via `PolygonWithHoles`).

## Weaknesses

- **Simple packing heuristic**: The decreasing-area first-fit with bounding-box minimization is a basic greedy approach. No metaheuristic optimization (GA, SA, etc.) is used to explore different orderings or rotations globally.
- **No sequence optimization**: Polygons are always sorted by decreasing area. There is no mechanism to try different orderings, which can significantly affect packing efficiency.
- **Rectangular sheets only**: The IFP computation assumes rectangular sheets. Irregular stock shapes are not supported.
- **CGAL dependency**: CGAL is a large, complex library that can be difficult to build and install, especially on Windows. This limits portability.
- **Limited Python API**: The `pack()` function is the main entry point with limited control over the packing process. No callbacks, no incremental placement, no manual polygon ordering.
- **No strip packing mode**: Only supports fixed-size sheets (bin packing). Cannot minimize strip length for the strip packing variant.
- **Build complexity**: Requires CMake, CGAL, Boost.Python, and a C++17 compiler, making the build process non-trivial compared to pure Python alternatives.

## Key Code Paths

1. **Full Packing Pipeline**: `python/packaide/packaide.py::pack()` -> parse SVGs via `svgelements` -> discretize curves -> dilate with `shapely.buffer()` -> convert to CGAL polygons via `bindings.cpp` -> `packing.hpp::pack_decreasing()` -> sort by area -> `pack_polygons_ordered_first_fit()` -> for each polygon and sheet: compute IFP, compute/cache NFPs, subtract NFPs from IFP via `CandidatePoints`, select best point by `IncrementalBoundingBoxHeuristic`.

2. **NFP Computation**: `no_fit_polygon.hpp::compute_nfp()` -> negate polygon B (reflect through origin) -> `CGAL::minkowski_sum_by_full_convolution_2(A, -B)` -> cache result in `State::nfp_cache`.

3. **Candidate Point Generation**: `primitives.hpp::CandidatePoints` constructor -> start with IFP as initial polygon set -> for each placed polygon's NFP: `polygon_set.difference(nfp)` -> extract boundary vertices of remaining feasible region -> return as candidate placement points.

4. **Sheet Placement**: `packing.hpp::pack_polygons_ordered_first_fit()` -> for each polygon: for each existing sheet: generate `CandidatePoints` -> if non-empty, evaluate heuristic at each point -> track best across all sheets and rotations -> place at best point, or create new sheet if no valid placement found.

5. **Persistence**: `persistence.hpp::State` -> `canonical_cache` maps raw polygon to index -> `nfp_cache` maps `NFPCacheKey{idx_a, idx_b, rotation}` to precomputed NFP -> reused across `pack()` calls when `persist=True`.
