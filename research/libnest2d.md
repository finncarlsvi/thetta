# libnest2d -- Detailed Analysis

## 1. Overview

libnest2d is a C++11 header-only library and framework for the **2D bin packing problem** (also called nesting or irregular strip packing). It was originally developed as part of PrusaSlicer (formerly Slic3r PE) and is used in production for the "auto-arrange" feature that places 3D-printed parts efficiently on the print bed.

**Where it is used:**
- **PrusaSlicer** (primary): The NFP placer + first-fit selection strategy powers the arrangement feature. The library was extracted from PrusaSlicer's codebase and lives at `src/libnest2d` within PrusaSlicer.
- **Cura**: Ultimaker's Cura slicer uses a fork/adaptation of libnest2d for its part arrangement.
- This standalone repository is a continuation/fork that accepts backported stable changes and experimental features.

**Key references cited by the project:**
- SVGNest (JavaScript library -- the original inspiration)
- "An effective heuristic for the two-dimensional irregular bin packing problem" (Lopez-Camacho et al. 2013)
- "Complete and robust no-fit polygon generation for the irregular stock cutting problem"
- "A comprehensive and robust procedure for obtaining the nofit polygon using Minkowski sums"

## 2. Architecture

### Header Structure

The library is organized as a set of header files under `include/libnest2d/`:

```
include/libnest2d/
  libnest2d.hpp              -- Top-level entry point; typedefs, nest() convenience function
  common.hpp                 -- Compiler compat macros, debug output, Radians type
  nester.hpp                 -- _Item, _Nester, PlacementStrategyLike, SelectionStrategyLike
  geometry_traits.hpp        -- Geometry abstraction layer (tags, traits, shapelike/pointlike)
  geometry_traits_nfp.hpp    -- NFP computation (convex-only Cuninghame-Green, NfpLevel enum)
  optimizer.hpp              -- Generic optimizer abstraction (Bound, StopCriteria, Result)
  parallel.hpp               -- Parallel enumeration helper

  placers/
    placer_boilerplate.hpp   -- CRTP base for placement strategies (PackResult, pack/trypack)
    nfpplacer.hpp            -- NFP-based placer (the main algorithm)
    bottomleftplacer.hpp     -- Bottom-left heuristic placer

  selections/
    selection_boilerplate.hpp -- Base for selection strategies
    firstfit.hpp             -- First-fit decreasing selection
    filler.hpp               -- Simple filler (one bin at a time)
    djd_heuristic.hpp        -- DJD heuristic (pairs/triplets)

  backends/clipper/
    clipper_polygon.hpp      -- ClipperLib::Polygon wrapper
    geometries.hpp           -- Clipper backend: template specializations for all geometry ops

  optimizers/
    nlopt/                   -- NLopt-based optimizers (subplex, genetic, simplex)
    optimlib/                -- OptimLib-based particle swarm (alternative)

  utils/
    boost_alg.hpp            -- Boost.Geometry integration for remaining operations
    rotfinder.hpp            -- Rotation finder utilities
    rotcalipers.hpp          -- Rotating calipers algorithm
```

### Template Design & Backend Abstraction

The core design principle is **policy-based design via templates**:

1. **Geometry Backend Abstraction**: All geometric operations go through a traits/tag-dispatch system defined in `geometry_traits.hpp`. The library defines abstract operations (`shapelike::rotate`, `shapelike::translate`, `shapelike::intersects`, `shapelike::isInside`, `nfp::merge`, etc.) that must be specialized for the chosen geometry type. Tags (`PointTag`, `PathTag`, `PolygonTag`, `BoxTag`, `CircleTag`) enable tag-dispatching.

2. **Key Traits**: `TPoint<S>`, `TCoord<S>`, `TContour<S>`, `TCompute<S>` (double-precision computation type), `OrientationType<S>` (clockwise/counter-clockwise).

3. **Clipper Backend** (`backends/clipper/geometries.hpp`): The default backend specializes all traits for `ClipperLib::IntPoint` / `ClipperLib::Polygon`. It uses integer coordinates (`ClipperLib::cInt`) with 1,000,000 units per mm. Boolean operations (merge/union for NFP) use Clipper's `ClipperOffset` and `Clipper` classes. Remaining operations (intersection tests, `isInside`, serialization) fall through to Boost.Geometry via `boost_alg.hpp`.

4. **Placer and Selector are template parameters** to `_Nester`. The `nest()` convenience function defaults to `NfpPlacer` + `FirstFitSelection`.

### Bin Types

Bins can be:
- `Box` (rectangular) -- the most common
- `_Circle` (circular)
- Arbitrary `RawShape` polygon

The `_NofitPolyPlacer` is templated on `TBin` so it supports all bin shapes. The `overfit()` family of functions handles boundary checking for each bin type.

## 3. Algorithm Details

### 3.1 NFP Placer (`nfpplacer.hpp`)

This is the primary placement engine. The algorithm for placing each item:

**Step 1 -- Configuration:**
- `NfpPConfig` holds: rotation angles to try (default: 0, 90, 180, 270 degrees), alignment mode, starting point, accuracy (0-1 slider), custom objective function, `before_packing` callback, parallel flag.

**Step 2 -- First item placement:**
- The first item is placed at the configured starting point (center, corner, etc.) of the bin.
- All configured rotations are tried; the one with smallest "overfit" is chosen.

**Step 3 -- Subsequent items (NFP-based):**
For each rotation angle:
1. Place item outside the bin to ensure disjunction.
2. **Compute NFP** of the new item against each already-placed item. This calls `calcnfp()` which dispatches to `nfp::noFitPolygon<NfpLevel::CONVEX_ONLY>()` (the Cuninghame-Green algorithm). NFPs are computed in parallel using `__parallel::enumerate`.
3. **Position correction**: `correctNfpPosition()` translates the raw NFP to its proper position relative to the stationary item and orbiter.
4. The individual NFPs are **merged** via `nfp::merge()` (Clipper boolean union) to get the combined no-fit region.
5. An **EdgeCache** is built for each merged NFP polygon. This data structure maps a normalized parameter `[0,1]` to points on the NFP boundary in O(log n) time via binary search on cumulative edge lengths.
6. **Local optimization**: For each NFP polygon and each sampled "corner" point on its boundary, a **SUBPLEX optimizer** (NLopt) searches for the position along the NFP boundary that minimizes the objective function. The objective function (customizable) by default minimizes `distance(item_center, bin_center) / norm + overfit_penalty`.
7. A **boundary check** ensures the convex hull of the merged pile fits inside the bin.
8. The best position across all rotations and all NFP contours (including holes, if `explore_holes` is enabled) is selected.

**Step 4 -- Accept result:**
- The item's translation and rotation are set.
- The merged pile is updated with `nfp::merge(merged_pile_, item.transformedShape())`.

**Key detail -- EdgeCache:**
The `EdgeCache` class pre-computes cumulative distances along the NFP boundary so that the optimizer can quickly map a single parameter `t in [0,1]` to an (x,y) coordinate on the boundary. The `accuracy` setting controls the stride (number of corner sample points), trading speed for quality.

### 3.2 NFP Computation (`geometry_traits_nfp.hpp`)

**Convex-only (Cuninghame-Green):**
The `nfpConvexOnly()` function implements the classic edge-sorting approach:
1. Collect all edges from the stationary polygon (same direction).
2. Collect all edges from the orbiter polygon (**reversed** direction -- this implements the Minkowski difference).
3. Sort all edges by their angle to the X axis. The comparison uses a quadrant-based scheme with dot products to avoid atan2 calls and maintain precision.
4. Build the NFP polygon by concatenating edges in sorted order.

The result is exact for two convex polygons. Complexity: O(n log n) where n = total edge count.

**Concave (nfpSimpleSimple -- incomplete):**
An implementation based on the algorithm from "CORMSIS-05-05" is present but appears incomplete/experimental. It has debug `std::cout` statements and TODO comments. It handles edge lists with turning points and the Mink(Q,R,direction) sub-procedure, but algorithms 2 and 3 (breaking into track line trips and finding boundaries) are not fully implemented.

**NfpLevel enum:**
```
CONVEX_ONLY           -- Both polygons must be convex (the only fully implemented level)
ONE_CONVEX            -- One polygon can be concave
BOTH_CONCAVE          -- Both can be concave
ONE_CONVEX_WITH_HOLES -- Concave with holes (one side)
BOTH_CONCAVE_WITH_HOLES -- Full generality
```

`MaxNfpLevel` defaults to `CONVEX_ONLY`. Higher levels require specializing `NfpImpl` (e.g., using libnfporb as a backend via `tools/nfp_svgnest_glue.hpp` or `tools/libnfpglue.hpp`).

### 3.3 Bottom-Left Placer (`bottomleftplacer.hpp`)

A simpler alternative placer. Algorithm:
1. Place item at top-right corner of the bin.
2. Alternately move down and left as far as possible without overlapping placed items.
3. Uses `availableSpaceDown()` and `availableSpaceLeft()` which construct "wall polygons" (shadow projections) and find minimum clearance by checking vertex-to-segment distances.
4. Only supports `Box` bins. Can optionally try 90-degree rotation.

Based on Lopez-Camacho et al. 2013 (Algorithm 8).

### 3.4 Selection Strategies

**FirstFitSelection (`firstfit.hpp`):**
1. Separate fixed items (pre-placed) from free items.
2. Sort free items by priority (descending), then by area (descending).
3. Remove items that cannot fit in an empty bin.
4. For each item, try to pack it into existing bins (in order). If none work, open a new bin.
5. Reports progress via callback.

**FillerSelection (`filler.hpp`):**
- Simpler variant: sorts by area descending, fills one bin at a time. When an item fails, starts a new bin.

**DJDHeuristic (`djd_heuristic.hpp`):**
- Based on Lopez-Camacho et al. 2013.
- More sophisticated: tries packing singles, then pairs, then triplets.
- Has configurable waste increment threshold and initial fill proportion (default 1/3 of bin area).
- Supports parallel bin filling.
- Much slower for large item counts (O(n^3) with triplets enabled).

### 3.5 Rotation Handling

Rotation is handled at two levels:
- **Placer level**: `NfpPConfig::rotations` lists angles to try (default: 0, pi/2, pi, 3*pi/2). For each angle, the full NFP computation and optimization is repeated.
- **Item level**: `_Item` stores a `rotation_` field and lazily computes `transformedShape()` by applying rotation then translation. The transformation is cached and invalidated on change.
- **Bottom-left placer**: Only tries 0 and 90 degrees (configurable via `allow_rotations`).

### 3.6 Geometry Backend

**Clipper operations:**
- `offset()` -- uses `ClipperOffset` with square joins for inflating/deflating shapes (used for min object distance).
- `merge()` / union -- uses `Clipper::Execute(ctUnion)` with reverse solution and negative poly fill type. Returns a `TMultiShape` (vector of polygons) handling holes via `PolyTree` traversal.
- `rotate()` -- manual cos/sin transformation on all points.
- `translate()` -- offset all points.

**Boost.Geometry fallback:**
- `intersects()`, `isInside()`, `touches()`, `boundingBox()`, serialization -- delegated to Boost.Geometry via `boost_alg.hpp`.

### 3.7 Optimizer

Two optimizer backends:
- **NLopt** (default): Subplex (local, derivative-free) for NFP placement optimization and genetic (global) for minimum bounding box.
- **OptimLib**: Particle swarm optimization (alternative).

The optimizer wraps a generic `TOptimizer<Method>` interface with `optimize_min()` / `optimize_max()` taking initial values and bounds.

## 4. API

### Simple usage:

```cpp
#include <libnest2d/libnest2d.hpp>
using namespace libnest2d;

std::vector<Item> items = { /* polygon vertices */ };
size_t bins = nest(items, Box(mm(250), mm(210)));

for (Item& item : items) {
    auto shape = item.transformedShape();  // final positioned polygon
    int bin = item.binId();                // which bin it ended up in
}
```

### Advanced usage:

```cpp
NestConfig<NfpPlacer, FirstFitSelection> cfg;
cfg.placer_config.rotations = {0, Pi/4, Pi/2};
cfg.placer_config.accuracy = 0.8f;
cfg.placer_config.alignment = NfpPConfig<PolygonImpl>::Alignment::BOTTOM_LEFT;

NestControl ctl;
ctl.progressfn = [](unsigned remaining) { /* ... */ };
ctl.stopcond = []{ return /* should_stop */; };

size_t bins = nest<NfpPlacer, FirstFitSelection>(items, Box(mm(250), mm(210)), mm(2), cfg, ctl);
```

### Key types:
- `Item` -- wraps a polygon with translation/rotation/inflation state and caching
- `Box`, `Circle` -- bin shape types
- `_Nester<Placer, Selector>` -- the main orchestrator
- `NestConfig<Placer, Selector>` -- configuration holder
- `NestControl` -- progress callback and stop condition
- `mm(val)` -- converts mm to coordinate units (1mm = 1,000,000 ClipperLib units)

### Item API:
- `item.translation()`, `item.rotation()`, `item.binId()` -- placement results
- `item.transformedShape()` -- get the final polygon
- `item.markAsFixedInBin(binId)` -- pre-place items
- `item.priority(p)` -- set priority (higher = placed first)
- `item.inflate(distance)` -- offset the shape (for min spacing)
- `item.isContourConvex()` -- convexity test (cached)
- `item.boundingBox()` -- bounding box (cached)
- `item.area()` -- signed area (cached)

## 5. Strengths

1. **Production-proven**: Used in PrusaSlicer for millions of users. The NFP placer produces good-quality results for real 3D printing scenarios.
2. **Clean, extensible architecture**: The template/trait system allows plugging in different geometry backends, optimizers, placers, and selectors without modifying core code.
3. **Header-only option**: Easy to integrate -- just include headers and link dependencies.
4. **Configurable objective function**: The `object_function` callback allows complete customization of placement scoring.
5. **Multi-bin support**: Automatically opens new bins when items don't fit, with configurable selection strategies.
6. **Parallelism**: NFP computation and optimization can run in parallel (configurable).
7. **Caching**: `_Item` aggressively caches transformations, area, bounding box, and convexity checks.
8. **Local optimization**: The SUBPLEX optimizer refines placement along NFP boundaries rather than just using vertex positions, finding better placements in the continuous space.

## 6. Weaknesses

1. **Convex-only NFP by default**: The built-in NFP implementation only handles convex polygons correctly. For concave shapes, it computes NFP of convex hulls, losing placement opportunities in concavities. The `nfpSimpleSimple` concave implementation is incomplete.
2. **Concave/hole handling requires external libraries**: To handle concave shapes properly, you must integrate libnfporb or another NFP library as a backend (via `NfpImpl` specialization). The library README explicitly states: "At this stage it works well for rectangles and convex closed polygons without considering holes and concavities."
3. **Optimizer overhead**: The SUBPLEX optimization for each rotation + each NFP contour can be slow for many items or complex shapes. The accuracy slider helps but is a compromise.
4. **No built-in concave NFP**: Unlike SVGNest which uses Minkowski sum decomposition, libnest2d relies on external implementations.
5. **Integer coordinate precision**: The Clipper backend uses 64-bit integers. While this avoids floating-point issues, it requires careful coordinate scaling and can lose precision for very small features.
6. **Limited selection heuristics**: DJD heuristic with triplets is O(n^3) and the simpler strategies may not find optimal bin counts.
7. **Bin types**: While multiple bin shapes are supported, some operations (like the bottom-left placer) only work with boxes.
8. **`nfpSimpleSimple` has debug output**: The concave NFP attempt has `std::cout` debug statements left in production code.

## 7. Key Code Paths

### Nesting execution flow:
```
nest() [libnest2d.hpp]
  -> _Nester::execute() [nester.hpp]
       -> inflate items by min_obj_distance/2
       -> selector_.packItems<PlacementStrategy>() [firstfit.hpp]
            -> sort items by priority then area (descending)
            -> remove_unpackable_items() -- test each item in empty bin
            -> for each item:
                 -> try placers[j].pack(item) for existing bins
                 -> if none work, open new bin
                      -> placer.trypack(item) [nfpplacer.hpp]
                           -> if first item: setInitialPosition, try rotations
                           -> else: for each rotation:
                                -> calcnfp() -- parallel NFP vs all placed items
                                     -> nfpConvexOnly() [geometry_traits_nfp.hpp]
                                     -> correctNfpPosition()
                                -> nfp::merge() -- Clipper union
                                -> build EdgeCache for each NFP
                                -> for each NFP contour + corners:
                                     -> SUBPLEX optimize along boundary
                                     -> boundaryCheck (convex hull in bin?)
                                -> pick best (rotation, position)
                           -> accept: update merged_pile_
       -> deflate items back
```

### Key files for understanding the system:
- `/Users/finn.carlsvi/thetta/research/repos/libnest2d/include/libnest2d/libnest2d.hpp` -- Entry point, type aliases, `nest()` function
- `/Users/finn.carlsvi/thetta/research/repos/libnest2d/include/libnest2d/nester.hpp` -- `_Item`, `_Nester`, strategy wrappers
- `/Users/finn.carlsvi/thetta/research/repos/libnest2d/include/libnest2d/placers/nfpplacer.hpp` -- NFP placer (main algorithm)
- `/Users/finn.carlsvi/thetta/research/repos/libnest2d/include/libnest2d/geometry_traits_nfp.hpp` -- NFP computation, NfpLevel
- `/Users/finn.carlsvi/thetta/research/repos/libnest2d/include/libnest2d/geometry_traits.hpp` -- Geometry abstraction layer
- `/Users/finn.carlsvi/thetta/research/repos/libnest2d/include/libnest2d/selections/firstfit.hpp` -- First-fit selection
- `/Users/finn.carlsvi/thetta/research/repos/libnest2d/include/libnest2d/backends/clipper/geometries.hpp` -- Clipper backend
- `/Users/finn.carlsvi/thetta/research/repos/libnest2d/examples/main.cpp` -- Usage example
