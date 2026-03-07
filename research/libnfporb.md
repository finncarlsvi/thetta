# libnfporb -- Detailed Analysis

## 1. Overview

libnfporb is a C++ header-only library that generates **No-Fit Polygons (NFPs)** using the **orbiting** (sliding) approach. Unlike simple Minkowski-sum methods that only work for convex shapes, the orbiting approach can handle **concave polygons and polygons with holes** -- making it suitable for general irregular nesting problems.

The library is built on top of **Boost.Geometry** and uses a custom `LongDouble` wrapper type for extended precision arithmetic (or optionally GMP rational arithmetic for exact computation). It is licensed under GPLv3.

libnfporb is also bundled inside libnest2d (at `tools/libnfporb/`) where it can serve as the NFP backend for concave shapes, upgrading libnest2d beyond its default convex-only capability.

**What it does:** Given two polygons A (stationary) and B (orbiting), `generate_nfp()` returns the NFP -- a set of rings that define where the reference point of B can be placed such that B touches but does not overlap A. The NFP may consist of multiple loops: an outer boundary plus inner loops for interlocking positions and holes.

## 2. Architecture

### Code Structure

```
src/
  libnfporb.hpp            -- Main entry point: generate_nfp(), cleanup utilities
  geometry.hpp             -- Core geometry types: LongDouble, point_t, segment_t, polygon_t,
                              helper functions (alignment, angles, min/max finders)
  translation_vector.hpp   -- TranslationVector struct (vector + source edge + metadata)
  history.hpp              -- History tracking for translation vectors (find, count)
  svg.hpp                  -- SVG debug visualization output
  wkt.hpp                  -- WKT file I/O for polygons

  algo/
    touching_point.hpp     -- Find all touching points between two rings
    find_feasible.hpp      -- Derive feasible translation vectors from touching points
    trim_vector.hpp        -- Trim translation vectors to prevent overlap
    select_next.hpp        -- Select the next translation vector (heuristic)
    search_start.hpp       -- Search for new start positions for additional NFP loops
    slide.hpp              -- The main slide/orbit loop

examples/
  nfp.cpp                  -- Simple CLI: read two WKT polygons, generate NFP, write SVG
  writeGolden.cpp          -- Golden test file generator
  fuzz.cpp                 -- Fuzzing harness
  isvalid.cpp              -- Polygon validity checker

test/
  test_main.cpp            -- Catch2-based test suite
```

### Type System

- **`LongDouble`**: A wrapper around `long double` with custom arithmetic operators and Boost.Geometry integration (custom `numeric_limits`, `square_root`, `abs`, `equals`, `smaller` specializations). The epsilon-based comparison (`NFP_EPSILON = 1e-8`) is critical for numerical stability in the orbiting algorithm.
- **`coord_t`**: Either `LongDouble` (default) or `bm::number<bm::gmp_rational>` if `LIBNFP_USE_RATIONAL` is defined (exact arithmetic, much slower).
- **`point_t`**: 2D point with `x_` and `y_` fields of type `coord_t`. Has a `marked_` flag used during start position search.
- **`polygon_t`**: `bg::model::polygon<point_t, false, true>` -- counter-clockwise outer ring, clockwise holes, closed.
- **`segment_t`**: `bg::model::segment<point_t>`.
- **`nfp_t`**: `std::vector<polygon_t::ring_type>` -- the NFP result is a collection of rings (outer loop + inner loops for interlocks/holes).

### Dependency on Boost.Geometry

The library heavily uses Boost.Geometry for:
- Polygon validity checking (`bg::is_valid`)
- Intersection detection (`bg::intersects`, `bg::overlaps`)
- Containment tests (`bg::within`, `bg::covered_by`, `bg::touches`)
- Geometric transformations (`trans::translate_transformer`)
- Line-polygon intersection (`bg::intersection`)
- Length computation (`bg::length`)

## 3. Algorithm Details -- How Orbital NFP Generation Works

The orbiting approach generates the NFP by **sliding polygon B around the boundary of polygon A**, recording the path of B's reference point. The algorithm handles concavities by detecting when B must enter or exit concave regions.

### 3.1 High-Level Flow (`generate_nfp()`)

1. **Preprocessing:**
   - Remove co-linear points from both polygons (simplify without changing shape).
   - Optionally validate both polygons with Boost.Geometry.
   - Translate both polygons to ensure all coordinates are positive (numerical stability).
   - Find extremal vertices: minimum/maximum X and Y for both polygons.

2. **Initial Positioning:**
   - Find the bottom-most vertex of A (minimum Y, rightmost if tied).
   - Find the top-most vertex of B (maximum Y, leftmost if tied).
   - Compute initial translation to place B touching A from below: `transB = pAstart - pBstart`.

3. **Outer NFP Loop:**
   - Call `slide()` to orbit B around the outside of A's outer ring.
   - B slides along A's boundary, always maintaining contact without overlap.
   - The slide loop records B's reference point at each step, forming the outer NFP ring.
   - Must complete a full loop back to the starting position (`LOOP` result).

4. **Interlock Detection:**
   - After the outer loop, repeatedly call `search_start_translation()` to find new start positions where B can be placed touching A in a concavity.
   - For each found position, run `slide()` again to trace the interlock loop.
   - Handle "jigsaw" cases (perfect fit where B sits in a concavity with no slide possible).
   - Handle "FIT" cases (B matches A exactly at some position).

5. **Hole Processing:**
   - For each hole (inner ring) of polygon A, search for start positions where B can fit inside the hole.
   - Run `slide()` for each hole to generate hole NFP loops.

6. **Post-processing:**
   - Remove consecutive repeating point patterns (oscillations/loops) from each NFP ring.
   - Apply `bg::correct()` to ensure valid ring orientation.
   - Translate all NFP rings back by reversing the initial positive-coordinate translation.

### 3.2 The Slide Function (`slide.hpp`)

The core orbiting loop:

```
slide(pA, rA, rB, nfp, startTrans, inside):
  1. Translate rB by startTrans
  2. Record B's reference point (rB.front())
  3. Find all touching points between rA and rB
  4. From touching points, derive feasible translation vectors
  5. Select the best next translation vector
  6. Trim the vector to prevent intersection
  7. Translate rB by the trimmed vector
  8. Check for overlap (error if found)
  9. Repeat until B returns to start position or touches the initial NFP ring
```

**Loop termination:** The slide stops when B's reference point equals the starting reference point (outer loop completed) or, for inner loops, when B's reference point touches the already-computed outer NFP ring.

### 3.3 Finding Touching Points (`touching_point.hpp`)

For each vertex of rA against each vertex/edge of rB (and vice versa):
- **VERTEX**: Two vertices coincide (within epsilon).
- **B_ON_A**: A vertex of B lies on an edge of A.
- **A_ON_B**: A vertex of A lies on an edge of B.

Uses `bg::intersects(segment, point)` for edge-point containment tests.

### 3.4 Finding Feasible Translation Vectors (`find_feasible.hpp`)

From each touching point, candidate translation vectors are derived:

- **VERTEX touches**: Four edge pairs are formed (two from each polygon meeting at the touching vertex). For each pair, the alignment (LEFT/RIGHT/ON) of the far endpoint determines whether the translation follows edge A or edge B. LEFT alignment means B should slide along its own edge (away from A); RIGHT means B should follow A's edge.

- **B_ON_A touches**: The translation vector is simply `vertexA - vertexB` (slide along A's edge to the next vertex).

- **A_ON_B touches**: The translation vector is `vertexA - vertexB` along B's edge.

**Filtering:** Each potential vector is checked against all touching edge pairs. Vectors that would cause immediate intersection (based on inner angle comparisons between the translation direction and the touching edges) are discarded. This is the most geometrically subtle part of the algorithm -- it determines which slides are valid without causing overlap.

### 3.5 Selecting the Next Translation Vector (`select_next.hpp`)

When multiple feasible vectors exist:

1. **If only one vector:** Use it.
2. **If history exists (not the first step):**
   - Sort candidates by history count (least used first).
   - For each candidate (least used first), trim it and check if the resulting position is NOT already in the NFP. Return the first such candidate.
   - If all lead to already-visited NFP points: sort by length, pick the longest unused.
   - Fallback: pick the least-used overall.
3. **If no history (first step):** Pick the longest vector.

This heuristic avoids infinite loops and oscillations by preferring unexplored directions.

### 3.6 Trimming Translation Vectors (`trim_vector.hpp`)

A translation vector may be too long (would cause B to pass through A or vice versa). The trim function:

1. For each vertex of rA: project it backward along the translation vector and intersect with rB. If the intersection is shorter than the current vector, shorten it.
2. For each vertex of rB: project it forward along the translation vector and intersect with rA. Same shortening logic.

This ensures B slides only until the next contact point, preventing pass-through.

### 3.7 Searching for Start Positions (`search_start.hpp`)

After the outer NFP loop is complete, this function searches for additional start positions for interlock/hole loops:

For each vertex of rA (not yet marked as visited):
  For each vertex of rB:
  1. Compute test translation = ptA - ptB.
  2. Translate rB by this amount.
  3. Check if translated B is identical to A (FIT case -- perfect match).
  4. Check if translated B is properly inside/outside A (depending on `inside` flag) without overlap.
  5. If the position is valid and not already in the NFP, return FOUND.
  6. Also try sliding along A's edges to the next vertex (one-step lookahead).

Vertices are marked after being tried to avoid repeated searches.

### 3.8 History Tracking (`history.hpp`)

The `History` type is a `std::vector<TranslationVector>`. Functions:
- `find(history, tv)` -- find a translation vector in history by segment+vector equality.
- `count(history, tv)` -- count occurrences (used for loop avoidance).

### 3.9 Post-processing

`delete_consecutive_repeating_point_patterns()` removes oscillations from NFP rings where the algorithm may have bounced back and forth. It detects repeating subsequences of points and collapses them.

## 4. Strengths and Weaknesses

### Strengths

1. **Handles concave polygons**: Unlike Minkowski-sum approaches, the orbiting method correctly generates NFPs for arbitrary concave shapes.
2. **Handles holes**: The algorithm explicitly processes inner rings (holes) of polygon A, generating separate NFP loops for positions where B fits inside holes.
3. **Handles interlocking**: After the outer loop, the algorithm searches for positions where B can fit into concavities of A, generating additional NFP loops.
4. **Header-only**: Easy to integrate -- single `#include` with Boost.Geometry as the only dependency (plus optionally GMP for rational arithmetic).
5. **Numerical precision**: The `LongDouble` wrapper with epsilon-based comparisons provides better stability than raw floating point. Optional GMP rational arithmetic mode provides exact results.
6. **Debug visualization**: Built-in SVG output for step-by-step algorithm visualization (enabled with `NFP_DEBUG`).
7. **Validation**: Input polygons are validated with Boost.Geometry before processing.
8. **Used in production**: Bundled with libnest2d and used as the concave NFP backend in PrusaSlicer.

### Weaknesses

1. **Performance**: The orbiting algorithm is inherently slower than the convex Minkowski-sum approach. Each step requires O(n*m) touching point detection (n,m = vertex counts). The overall complexity can be O(n^2 * m^2) or worse for complex shapes.
2. **Numerical fragility**: Despite the epsilon-based comparisons, floating-point edge cases can cause:
   - Infinite loops (mitigated by history tracking but not eliminated).
   - Missing interlock positions.
   - The algorithm throwing `runtime_error` for "unable to complete outer nfp loop".
3. **Header-only function definitions**: All functions are defined in headers without `inline` in some cases, which can cause multiple-definition linker errors if included from multiple translation units. This is a common issue with header-only C++ libraries that define non-template functions.
4. **Debug output in production code**: Some `std::cerr` output (e.g., in `find_maximum_x`) is not guarded by `NFP_DEBUG`.
5. **GPLv3 license**: The viral GPL license limits use in proprietary software.
6. **Limited API**: The library only provides NFP generation -- it does not include a placer or nesting solver. You must use it as a component within a larger system (like libnest2d).
7. **No multi-threading**: The algorithm is single-threaded.
8. **Coordinate translation requirement**: The algorithm translates all coordinates to be positive before processing, which adds overhead and potential precision loss.
9. **Heuristic-based direction selection**: The `select_next_translation_vector` function uses heuristics (longest unused, least-used) rather than a mathematically proven approach, which means it can occasionally choose suboptimal paths.

## 5. Key Code Paths

### NFP Generation Flow:
```
generate_nfp(pA, pB) [libnfporb.hpp]
  -> remove_co_linear(pA), remove_co_linear(pB)
  -> bg::is_valid() checks
  -> translate both to positive coordinates
  -> find extremal vertices (min/max X/Y)
  -> compute initial translation: transB = pAstart - pBstart
  -> slide(pA, pA.outer(), pB.outer(), nfp, transB, false)  // outer loop
       -> [slide.hpp] loop:
            find_touching_points(rA, rB)                     // touching_point.hpp
            find_feasible_translation_vectors(rA, rB, touchers) // find_feasible.hpp
            select_next_translation_vector(nfp, pA, rA, rB, feasible, history) // select_next.hpp
            trim_vector(rA, rB, next)                        // trim_vector.hpp
            translate rB by trimmed vector
            check for return to start
  -> loop: search_start_translation(pA.outer(), pB.outer(), nfp, false, startTrans)
       -> if FOUND: slide() for interlock loop
       -> if FIT: record single point
  -> for each hole in pA:
       -> search_start_translation(hole, pB.outer(), nfp, true, startTrans)
       -> if FOUND: slide() for hole loop
  -> post-process: delete_consecutive_repeating_point_patterns()
  -> translate back to original coordinates
```

### Key files:
- `/Users/finn.carlsvi/thetta/research/repos/libnfporb/src/libnfporb.hpp` -- Main entry point, `generate_nfp()`, post-processing
- `/Users/finn.carlsvi/thetta/research/repos/libnfporb/src/geometry.hpp` -- All geometry types, `LongDouble`, comparison functions, polygon utilities
- `/Users/finn.carlsvi/thetta/research/repos/libnfporb/src/algo/slide.hpp` -- Core orbiting loop
- `/Users/finn.carlsvi/thetta/research/repos/libnfporb/src/algo/find_feasible.hpp` -- Feasible translation vector derivation (most complex algorithmic logic)
- `/Users/finn.carlsvi/thetta/research/repos/libnfporb/src/algo/touching_point.hpp` -- Contact detection between rings
- `/Users/finn.carlsvi/thetta/research/repos/libnfporb/src/algo/select_next.hpp` -- Heuristic direction selection
- `/Users/finn.carlsvi/thetta/research/repos/libnfporb/src/algo/trim_vector.hpp` -- Vector trimming to prevent overlap
- `/Users/finn.carlsvi/thetta/research/repos/libnfporb/src/algo/search_start.hpp` -- Start position search for inner loops
- `/Users/finn.carlsvi/thetta/research/repos/libnfporb/src/translation_vector.hpp` -- TranslationVector data structure
- `/Users/finn.carlsvi/thetta/research/repos/libnfporb/src/history.hpp` -- History tracking for loop avoidance
- `/Users/finn.carlsvi/thetta/research/repos/libnfporb/examples/nfp.cpp` -- Usage example
