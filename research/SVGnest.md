# SVGnest - Detailed Analysis

## 1. Overview

SVGnest is a browser-based 2D nesting tool for irregular shapes, created by Jack Qiao. It takes a set of 2D shapes (provided as an SVG file) and attempts to pack them as tightly as possible into one or more rectangular (or arbitrary polygon) bins. The primary use case is optimizing material usage for CNC machines, laser cutters, and plasma cutters.

The tool runs entirely in the browser with no server component. It uses Web Workers for parallel NFP (No-Fit Polygon) computation and a genetic algorithm to search for good placement orderings and rotations.

**Repository:** https://github.com/nicklasb/SVGnest (fork of Jack Qiao's original)
**License:** MIT
**Language:** Vanilla JavaScript (ES5/ES6 mix), no build system or bundler.

---

## 2. Architecture

### File Structure

| File | Lines | Role |
|------|-------|------|
| `svgnest.js` | 988 | **Main orchestrator.** Contains `SvgNest` (top-level API), `GeneticAlgorithm`, configuration, worker launching, placement application. |
| `svgparser.js` | 792 | **SVG parsing.** Loads SVG strings, applies transforms, flattens groups, converts all shape types to polygons. |
| `util/geometryutil.js` | 1891 | **Core geometry engine.** NFP computation (orbiting algorithm), polygon area, point-in-polygon, intersection tests, Bezier/Arc linearization, polygon slide distance. |
| `util/placementworker.js` | 297 | **Placement evaluation.** Runs inside a Web Worker. Given an ordering of parts and pre-computed NFPs, determines where each part goes and computes fitness. |
| `util/parallel.js` | 386 | **Web Worker abstraction.** Provides `Parallel.map()` for distributing NFP computations across workers. |
| `util/clipper.js` | 243 | **Clipper library** (minified). Polygon boolean operations, Minkowski sum, offsetting. |
| `util/matrix.js` | 156 | **2D affine matrix** operations (translate, rotate, scale, skew). Used by SVG transform handling. |
| `util/eval.js` | 11 | **Worker bootstrap.** Receives and `eval()`s code strings posted to Web Workers. |
| `util/filesaver.js` | - | File download utility (FileSaver.js). |
| `util/pathsegpolyfill.js` | - | Polyfill for the deprecated `SVGPathSeg` API. |
| `util/domparser.js` | - | DOMParser polyfill. |
| `util/json.js` | - | JSON parse/stringify for worker contexts. |
| `index.html` | ~800 | UI: file upload, bin selection, settings panel, SVG display, start/stop controls. |

### Data Flow

```
SVG File (string)
    |
    v
SvgParser.load() --> DOM tree
    |
    v
SvgParser.clean() --> flattened, transform-free SVG elements
    |
    v
SvgNest.getParts() --> polygon arrays [{x,y},...] with parent/child (hole) tree
    |
    v
SvgNest.start() --> offset polygons by spacing, normalize winding, prepare bin
    |
    v
GeneticAlgorithm --> generates candidate orderings + rotations
    |
    v
launchWorkers() --> compute NFP pairs via Parallel.map() in Web Workers
    |
    v
PlacementWorker.placePaths() --> place parts sequentially using NFPs, return fitness
    |
    v
displayCallback() --> render best placement as SVG
```

---

## 3. Algorithm Details

### 3.1 NFP (No-Fit Polygon) Computation

The NFP defines the locus of positions where a reference point of polygon B can be placed such that B touches but does not overlap polygon A. SVGnest computes two types:

**Inner NFP** (part inside bin): The region where a part's reference point can move while staying entirely inside the bin. Computed at `svgnest.js:366-386`.

- If the bin is a rectangle, uses the fast `noFitPolygonRectangle()` shortcut (`geometryutil.js:1359-1412`), which simply computes the bounding box difference.
- Otherwise, uses the full orbiting algorithm `noFitPolygon(A, B, true, searchEdges)`.

**Outer NFP** (part outside part): The region that a part's reference point must avoid to not overlap an already-placed part. Computed at `svgnest.js:388-414`.

- If `exploreConcave` is off (default), uses **Clipper's Minkowski sum** (`svgnest.js:491-519`): negate B's coordinates, compute `MinkowskiSum(A, -B)`, take the largest resulting polygon. This is fast but only produces the outer NFP contour (no concave pockets).
- If `exploreConcave` is on, uses the full orbiting algorithm which can find multiple NFP loops for concave shapes.

**The Orbiting Algorithm** (`geometryutil.js:1417-1707`):

1. **Start position:** For outer NFPs, place B so its bottom-most vertex touches A's top-most vertex (guaranteed no overlap). For inner NFPs, calls `searchStartPoint()` to find a valid non-overlapping interior position.

2. **Main loop** (line 1483): At each step:
   - Find all **touching points** between A and B (vertex-vertex, vertex-on-edge, edge-on-vertex). Three touch types are identified (lines 1486-1500).
   - Generate **candidate translation vectors** from each touching point -- edges of A and inverted edges of B at touch points (lines 1504-1596).
   - For each candidate vector, compute `polygonSlideDistance()` to find how far B can slide along that vector before penetrating A.
   - Choose the vector that allows the **maximum** slide distance.
   - Translate B by that vector, record the new position as an NFP vertex.
   - Repeat until the NFP path loops back to the start.

3. **Multiple NFP loops:** If `searchEdges` is true, after finding one NFP loop, `searchStartPoint()` looks for additional start positions on unmarked edges of A, producing additional NFP contours (for concave pockets).

**Key supporting functions:**
- `polygonSlideDistance()` (line 1102): For every pair of edges from A and B, computes `segmentDistance()` in the slide direction. Returns the minimum positive distance before collision.
- `segmentDistance()` (line 938): Projects two line segments onto each other along a direction vector, accounting for collinear and parallel cases.
- `pointDistance()` (line 907): Projects a point onto a line segment along a normal direction.
- `searchStartPoint()` (line 1225): Tries placing B at each vertex of A, checks if B is inside/outside as required, checks for intersections, slides along edges to find valid positions.

### 3.2 Genetic Algorithm

Defined in `svgnest.js:820-986`. The GA optimizes two things: the **order** in which parts are placed and the **rotation** of each part.

**Encoding** (line 831):
- Each individual is `{placement: [polygon_refs...], rotation: [angle_values...]}`.
- `placement` is a permutation of the polygon objects (determines insertion order).
- `rotation` is an array of angles, one per part, chosen from `{0, 360/n, 2*360/n, ...}` where `n = config.rotations` (default 4, giving 0/90/180/270).

**Initialization** (line 265-275):
- The "adam" individual sorts parts by **decreasing area** (largest first).
- The rest of the initial population is generated by mutating adam.

**Mutation** (`mutate()`, line 872-894):
- For each part position, with probability `mutationRate/100`:
  - Swap the part with its neighbor (adjacent swap).
- Independently, with the same probability:
  - Assign a new random rotation angle.

**Crossover** (`mate()`, line 897-932):
- **Single-point order crossover (OX1 variant):** Pick a random cut point (between 10% and 90%). Take the first segment from parent 1, fill remaining positions with parts from parent 2 in their original order (skipping duplicates). Vice versa for child 2.

**Selection** (`randomWeightedIndividual()`, line 963-986):
- Population sorted by fitness (ascending, lower is better).
- Uses a **linearly weighted** random selection: earlier (fitter) individuals have higher probability. The weight decreases as `2 * weight * (remaining/total)`.

**Generation** (`generation()`, line 934-960):
- **Elitism:** Best individual is preserved unchanged.
- Remaining slots filled by mating two selected parents, then mutating both children.

**Population size:** Default 10 (configurable). Very small by GA standards.

### 3.3 Placement/Evaluation

Implemented in `PlacementWorker.placePaths()` (`placementworker.js:59-289`). This is the core evaluation function that determines fitness.

**Algorithm:**
1. Rotate all parts by their assigned angles (line 70-79).
2. **Outer loop** -- open new bins as needed (line 86):
   - For each new bin, `fitness += 1` (penalty per bin used).
3. **Inner loop** -- for each unplaced part (line 92):
   - Look up the inner NFP (part-in-bin) from cache.
   - Look up all outer NFPs (part-vs-each-placed-part) from cache.
   - If any required NFP is missing, skip this part.
   - **First part in bin** (line 122-141): Place at the leftmost point of the inner NFP.
   - **Subsequent parts** (line 143-263):
     a. Convert and union all outer NFPs (translated to placed positions) using Clipper boolean union.
     b. Subtract the unioned NFPs from the inner NFP using Clipper difference. The result (`finalNfp`) represents valid placement positions.
     c. For each vertex of `finalNfp`, trial-place the part there and compute the **bounding box** of all placed parts.
     d. Score: `width * 2 + height` (width weighted 2x to compress horizontally -- "gravity" direction).
     e. Choose the position with the minimum score. Ties broken by minimum x.
4. After all parts placed in current bin, add `minwidth / binarea` to fitness (line 266-268).
5. Unplaced parts get a heavy penalty: `fitness += 2 * unplaced_count` (line 286).

**Fitness** (lower is better):
```
fitness = num_bins_used + sum(minwidth_per_bin / binarea) + 2 * num_unplaced
```

### 3.4 Rotation Handling

- `config.rotations` (default 4) determines the set of allowed angles: `{0, 90, 180, 270}` for rotations=4.
- `randomAngle()` (line 840-869) picks a random angle from the allowed set, but validates it by checking the rotated bounding box fits within the bin. Falls back to 0 if nothing fits.
- Rotation is applied by simple 2D rotation matrix in `rotatePolygon()` (`placementworker.js:27-47`).
- Rotations are part of the GA genome and get mutated alongside placement order.
- NFPs are cached with rotation as part of the key: `{A: id, B: id, inside: bool, Arotation: deg, Brotation: deg}`.

---

## 4. Input/Output

### Input
- **Format:** SVG file (string), loaded via file upload or pasted.
- **Supported SVG elements:** `svg`, `circle`, `ellipse`, `path`, `polygon`, `polyline`, `rect`, `line` (see `svgparser.js:17`).
- **Transforms:** All SVG transforms (translate, rotate, scale, skew, matrix) are baked into coordinates during parsing. Groups (`<g>`) are flattened.
- **Compound paths:** Split into individual paths at M/m commands (`splitPath()`, `svgparser.js:485-562`).
- **Curves:** Bezier curves (quadratic/cubic) and arcs are linearized to polylines with configurable tolerance (`curveTolerance`, default 0.3 SVG units).

### Internal Representation
- Polygons are arrays of `{x: number, y: number}` objects.
- Additional properties attached to arrays: `.id`, `.source`, `.rotation`, `.children` (for holes), `.offsetx`, `.offsety`.
- The bin polygon gets `id = -1`, `.width`, `.height`.
- A tree structure (`children` arrays) represents part-in-part (hole) relationships.

### Output
- SVG elements with transforms applied. Each bin gets its own `<svg>` element containing the bin outline and placed parts wrapped in `<g>` elements with `translate(x y) rotate(deg)` transforms.
- Can be exported/downloaded via FileSaver.js.

### Configuration Parameters
| Parameter | Default | Description |
|-----------|---------|-------------|
| `clipperScale` | 10,000,000 | Integer scaling factor for Clipper library |
| `curveTolerance` | 0.3 | Max deviation for Bezier/Arc linearization (SVG units) |
| `spacing` | 0 | Gap between parts (applied as polygon offset) |
| `rotations` | 4 | Number of allowed rotation angles (360/n increments) |
| `populationSize` | 10 | GA population size |
| `mutationRate` | 10 | Mutation probability percentage |
| `useHoles` | false | Enable nesting parts inside holes of other parts |
| `exploreConcave` | false | Use orbiting NFP instead of Minkowski for concave pockets |

---

## 5. Strengths

1. **Runs entirely in the browser** -- no server, no installation. Immediate accessibility.

2. **Web Worker parallelism** for NFP computation. NFP pairs are distributed across `navigator.hardwareConcurrency` workers, keeping the UI responsive.

3. **NFP caching** -- NFPs are keyed by part IDs and rotations, so repeated evaluations in the GA reuse previously computed NFPs. Cache is pruned each generation to avoid memory bloat.

4. **Clipper library integration** -- polygon offsetting (for spacing), boolean operations (for combining NFPs during placement), and Minkowski sum (fast outer NFP for convex/simple cases) are all handled by the well-tested Clipper library.

5. **Handles diverse SVG input** -- circles, ellipses, arcs, Bezier curves, rects, polygons, paths with transforms are all supported and converted to polygons.

6. **Hole detection** -- parts containing holes can be identified (tree structure via point-in-polygon tests), and other parts can optionally be nested inside those holes (`useHoles` flag).

7. **Concave shape support** -- the orbiting NFP algorithm (`exploreConcave` flag) can find concave pockets that the Minkowski approach misses.

8. **Incremental improvement** -- the GA runs continuously, displaying the best result found so far. Users can stop at any time.

9. **Rectangle bin optimization** -- fast-path NFP computation for rectangular bins avoids the expensive orbiting algorithm.

---

## 6. Weaknesses

1. **Very small GA population** (default 10). This severely limits exploration of the search space. Combined with adjacent-swap-only mutation, diversity is low and convergence to local optima is likely.

2. **Greedy bottom-left placement** -- within a single evaluation, parts are placed one at a time using a greedy heuristic (minimize `2*width + height`). There is no backtracking. A bad early placement can doom the entire layout.

3. **O(n^2) NFP pairs** -- for n parts, there are O(n^2) NFP pairs to compute. Each NFP computation itself is O(v_A * v_B) per orbiting step, with multiple steps. This scales poorly with part count.

4. **Orbiting algorithm fragility** -- the NFP orbiting algorithm (`noFitPolygon`) has known robustness issues with degenerate geometries (collinear edges, very small edges, near-tangent touching). The code includes a `counter < 10*(A.length + B.length)` safety check (line 1483) and silently returns null on failure, causing parts to be skipped.

5. **No multi-bin optimization** -- when parts overflow one bin, a new bin is opened. But the algorithm does not optimize across bins (e.g., rebalancing parts between bins).

6. **Deprecated SVG APIs** -- relies on `SVGPathSeg` API (deprecated, requires polyfill) and `pathSegList` manipulation. Modern browsers have removed this API.

7. **No support for non-polygonal bins** -- while the bin can be any polygon, there is no support for sheets with defects, irregular edges from remnant material, etc.

8. **Spacing implementation is approximate** -- polygon offsetting via Clipper can produce artifacts with very small or complex polygons. Parts are offset outward and the bin is offset inward by half the spacing, which does not guarantee exact spacing for concave shapes.

9. **Gravity direction hardcoded** -- placement heuristic always compresses leftward (`width * 2 + height`). There is a TODO comment about generalizing gravity direction (`placementworker.js:215`) but it was never implemented.

10. **No rotation optimization within placement** -- the GA pre-assigns rotations, but the placement evaluation does not try alternative rotations for individual parts that fail to fit.

11. **Memory management** -- NFP cache is stored as stringified JSON keys in a plain object. For large part counts with many rotations, this becomes a memory and performance bottleneck due to repeated `JSON.stringify()` calls.

12. **Worker communication overhead** -- entire polygon data and NFP caches are serialized/deserialized via `postMessage` for each worker invocation. The `eval.js` approach (posting code as strings to be `eval()`d) is fragile and a security concern.

13. **No TypeScript, no tests, no build system** -- the codebase has no automated tests. The geometry code is complex and has known edge cases (multiple TODO comments and "may need work!" warnings in the code).

14. **Single-threaded placement** -- while NFP computation is parallelized, the actual placement evaluation (`placePaths`) runs in a single worker. For large part counts, this is the bottleneck.

15. **Mutation only swaps adjacent parts** -- the GA mutation can only swap part `i` with part `i+1`. This means large reorderings require many generations, limiting the algorithm's ability to escape local optima.

---

## 7. Key Code Paths

### Main Entry Point
- **`SvgNest.start()`** -- `svgnest.js:131-241`. Sets up the bin polygon, offsets parts for spacing, starts the worker timer loop.

### Worker Launch & NFP Generation
- **`SvgNest.launchWorkers()`** -- `svgnest.js:243-591`. Initializes GA (first call), selects next individual to evaluate, computes required NFP pairs, dispatches to workers via `Parallel.map()`, chains placement evaluation.

### NFP Computation (in Web Worker)
- **NFP map function** -- `svgnest.js:354-521` (anonymous function passed to `p.map()`). For each pair, rotates polygons, calls either `noFitPolygonRectangle()`, `noFitPolygon()`, or `minkowskiDifference()`.
- **`GeometryUtil.noFitPolygon()`** -- `geometryutil.js:1417-1707`. The orbiting NFP algorithm.
- **`GeometryUtil.polygonSlideDistance()`** -- `geometryutil.js:1102-1165`. Computes how far B can slide along a vector before hitting A.
- **`GeometryUtil.segmentDistance()`** -- `geometryutil.js:938-1100`. Core segment-to-segment distance computation.
- **`GeometryUtil.searchStartPoint()`** -- `geometryutil.js:1225-1340`. Finds valid starting positions for NFP orbiting.

### Placement Evaluation (in Web Worker)
- **`PlacementWorker.placePaths()`** -- `placementworker.js:59-289`. Sequential greedy placement of all parts using pre-computed NFPs. Returns fitness score.
- **Placement position scoring** -- `placementworker.js:216-258`. For each candidate position, computes bounding box of all placed parts, scores as `width*2 + height`.

### Genetic Algorithm
- **`GeneticAlgorithm` constructor** -- `svgnest.js:820-837`. Creates initial population from area-sorted seed.
- **`GeneticAlgorithm.prototype.mutate()`** -- `svgnest.js:872-894`. Adjacent swap + random rotation mutations.
- **`GeneticAlgorithm.prototype.mate()`** -- `svgnest.js:897-932`. Single-point order crossover.
- **`GeneticAlgorithm.prototype.generation()`** -- `svgnest.js:934-960`. Elitist selection, mating, and mutation.
- **`GeneticAlgorithm.prototype.randomWeightedIndividual()`** -- `svgnest.js:963-986`. Linearly-weighted fitness-proportionate selection.

### SVG Parsing
- **`SvgParser.load()`** -- `svgparser.js:29-59`. Parses SVG string to DOM.
- **`SvgParser.cleanInput()`** -- `svgparser.js:62-78`. Applies transforms, flattens groups, filters elements, splits compound paths.
- **`SvgParser.applyTransform()`** -- `svgparser.js:223-450`. Recursively bakes SVG transforms into element coordinates. Handles path, polygon, circle, ellipse, rect, line.
- **`SvgParser.polygonify()`** -- `svgparser.js:576-780`. Converts any SVG element to a polygon (array of {x,y} points). Linearizes curves and arcs.

### Polygon Processing
- **`SvgNest.getParts()`** -- `svgnest.js:595-668`. Converts SVG elements to polygons, builds parent-child tree for hole detection.
- **`SvgNest.cleanPolygon()`** -- `svgnest.js:695-722`. Uses Clipper to remove self-intersections and clean up small artifacts.
- **`SvgNest.polygonOffset()`** -- `svgnest.js:672-692`. Uses Clipper to expand/contract polygons for spacing.

### Geometry Utilities
- **`GeometryUtil.polygonArea()`** -- `geometryutil.js:560-567`. Shoelace formula.
- **`GeometryUtil.pointInPolygon()`** -- `geometryutil.js:524-556`. Ray-casting algorithm.
- **`GeometryUtil.isRectangle()`** -- `geometryutil.js:1342-1356`. Checks if all vertices lie on bounding box edges.
- **`GeometryUtil.rotatePolygon()`** -- `geometryutil.js:1870-1889`. 2D rotation about origin.
- **`GeometryUtil.noFitPolygonRectangle()`** -- `geometryutil.js:1359-1412`. Fast inner NFP for rectangular bin.
- **Bezier linearization** -- `geometryutil.js:162-307`. Quadratic and cubic Bezier subdivision via de Casteljau.
- **Arc linearization** -- `geometryutil.js:309-487`. SVG arc to polyline conversion.
