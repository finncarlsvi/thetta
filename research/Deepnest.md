# Deepnest - Detailed Codebase Analysis

## 1. Overview

Deepnest is a desktop nesting application for CNC/laser cutting, built by Jack Qiao. It takes irregular 2D shapes (parts) and packs them efficiently onto rectangular or non-rectangular sheets of material, minimizing material waste. It is the desktop (Electron) evolution of [SVGnest](http://svgnest.com), an earlier browser-based nesting tool by the same author.

The core algorithm uses a **No-Fit Polygon (NFP)** approach for placement combined with a **Genetic Algorithm (GA)** for global optimization of insertion order and part rotation. The NFP computation is accelerated with a **C++ native addon** using Boost.Polygon for Minkowski sum calculations.

- **Author:** Jack Qiao
- **License:** GPLv3 (deepnest.js), MIT (SVGnest/geometry utilities)
- **Tech Stack:** Electron 1.4, Node.js, C++ (Boost.Polygon via node-gyp/NAN), ClipperLib (JS)

---

## 2. Architecture

### High-Level Structure

Deepnest is an Electron app with a **multi-window architecture**:

1. **Main Window** (`main/index.html`) -- the UI. Loads parts, displays nesting results, handles import/export.
2. **Background Window** (`main/background.html` + `main/background.js`) -- a hidden Electron BrowserWindow that performs the heavy computation (NFP generation + placement). Communication with the main window is via Electron IPC (`ipcRenderer`/`ipcMain`).
3. **Main Process** (`main.js`) -- the Electron main process. Routes IPC messages between the main window and background window(s). Manages the NFP cache directory.

### Key Files and Their Roles

| File | Role |
|------|------|
| `main.js` | Electron main process. Creates windows, routes IPC messages between UI and background workers. |
| `main/index.html` | Main UI -- part import, sheet setup, configuration, nest display, export. ~1300 lines of inline JS. |
| `main/deepnest.js` | Core orchestration. The `DeepNest` object manages parts, config, and the GA. Contains the `GeneticAlgorithm` class (Deepnest version). Sends work to background via IPC. |
| `main/svgnest.js` | The original SVGnest in-browser nesting engine. Contains its own `GeneticAlgorithm` and `PlacementWorker` orchestration using web workers (Parallel.js). Not the primary code path in Deepnest but shares the same logic. |
| `main/background.js` | Runs in the hidden background window. Receives nesting jobs via IPC, computes NFPs (using both ClipperLib Minkowski and the C++ addon), runs placement (`placeParts()`), and returns results. This is where the actual nesting work happens. |
| `main/background single.js` | Alternate version of background.js (slightly different caching strategy, uses filesystem-based NFP cache). |
| `main/svgparser.js` | SVG parsing and geometry conversion. Loads SVG strings, flattens transforms, converts circles/ellipses/rects/paths to polygon point lists via linearization of Bezier curves. |
| `main/util/geometryutil.js` | Core geometry library (~1900 lines). Contains: polygon area, bounds, point-in-polygon, polygon intersection, **NFP computation** (orbital approach), segment distance calculations, polygon slide distance, rotation. |
| `main/util/placementworker.js` | Placement logic for the SVGnest code path. Contains `PlacementWorker` which evaluates placement positions using NFP caches, Clipper boolean operations, and a gravity-based heuristic. |
| `main/util/clipper.js` | ClipperLib -- Angus Johnson's polygon clipping library (JS port). Used for boolean operations (union, difference), polygon offsetting, Minkowski sum, and polygon cleaning. |
| `main/util/parallel.js` | Parallel.js -- web worker parallelism library. Used in svgnest.js code path for parallel NFP computation. |
| `main/util/simplify.js` | Ramer-Douglas-Peucker line simplification. |
| `main/util/d3-polygon.js` | d3-polygon -- convex hull, polygon area, centroid. |
| `main/util/hull.js` | Convex hull (Graham scan). |
| `addon.cc` | Node.js native addon entry point. Exposes `calculateNFP` to JavaScript. |
| `minkowski.cc` | C++ NFP computation via Boost.Polygon Minkowski sum. Single-pair version. |
| `minkowski thread.cc` | Threaded version of the C++ NFP computation (uses boost::asio thread pool). Also contains `calculateNFPBatch` for batch processing. Not actively compiled (binding.gyp uses `minkowski.cc`). |
| `minkowski.h` | Header declaring `calculateNFP`. |
| `binding.gyp` | node-gyp build config for the C++ addon. Links against Boost (hardcoded path). |

### Data Flow

```
User imports SVG/DXF
    |
    v
SvgParser.load() -> SvgParser.clean() -> polygonify()
    |
    v
DeepNest.getParts() -- builds polygon tree (parent/child for holes)
    |
    v
DeepNest.start() -- applies spacing offsets, simplification, inits GA
    |
    v
setInterval(launchWorkers, 100ms)
    |
    v
For each GA individual (placement order + rotations):
    Send via IPC to background window
    |
    v
background.js receives job:
    1. Compute outer NFPs for all part pairs (parallel web workers using ClipperLib.MinkowskiSum)
    2. Compute inner NFPs for holes (C++ addon via addon.calculateNFP using Boost.Polygon)
    3. Cache NFPs in window.nfpcache (keyed by source ID + rotation)
    4. Run placeParts() -- greedy placement using NFP intersection via Clipper
    5. Return fitness + placements via IPC
    |
    v
DeepNest receives result, updates GA population fitness
When all individuals evaluated -> GA.generation() (selection, crossover, mutation)
Repeat
```

---

## 3. Algorithm Details

### 3.1 NFP Computation

There are **three different NFP implementations** in the codebase:

#### A. JavaScript Orbital NFP (`geometryutil.js`, line ~1437)
- `GeometryUtil.noFitPolygon(A, B, inside, searchEdges)`
- **Used by:** svgnest.js code path (web workers)
- **Method:** Orbiting/sliding approach per [Lopez-Camacho et al. 2013] and [E.K. Burke et al. 2006]
- **Steps:**
  1. Find a valid starting position (top-most of A, bottom-most of B for outer NFP; `searchStartPoint()` for inner)
  2. Place B at start position with offset
  3. **Orbit loop** (max iterations = 10 * (|A| + |B|)):
     - Find all touching points/edges between A and B (vertex-vertex, vertex-edge, edge-vertex)
     - Generate candidate translation vectors from touching features
     - For each candidate vector, compute `polygonSlideDistance()` -- the maximum distance B can slide along that vector before intersection
     - Pick the vector with the maximum valid slide distance
     - Translate B, record the new reference point as an NFP vertex
     - Continue until the loop closes (returns to start point)
  4. If `searchEdges` is true, search for additional NFP components (for concave shapes) via `searchStartPoint()`
  5. Return array of NFP polygons

- **Special case:** `noFitPolygonRectangle(A, B)` for when A is a rectangle (line 1379) -- simply computes the inner fit rectangle directly from bounds.

#### B. ClipperLib Minkowski Sum (`background.js`, line ~173 and ~650)
- Used for **outer NFPs between parts** (the common case)
- **Method:** Calls `ClipperLib.Clipper.MinkowskiSum(Ac, Bc_negated, true)` which computes the Minkowski sum of A and the reflection of B
- Steps:
  1. Convert polygons to Clipper integer coordinates (scale up by 10,000,000)
  2. Negate B's coordinates (reflection for NFP = Minkowski sum of A and -B)
  3. Call MinkowskiSum
  4. Select the largest-area result polygon
  5. Shift by B[0] offset (NFPs are referenced to B's first point)
- **Much faster** than the orbital approach but doesn't handle holes

#### C. C++ Boost.Polygon NFP (`minkowski.cc`, line ~106)
- Used for **inner NFPs** and cases with holes in parts
- **Method:** Minkowski sum via Boost.Polygon's `convolve` operations
- Steps:
  1. Receive polygon A (with holes as `children`) and polygon B from JavaScript via NAN bindings
  2. Compute dynamic scaling factor: `inputscale = 0.1 * INT_MAX / max_coordinate` to map floating-point coordinates to integer space while preserving precision
  3. Build Boost polygon_set for A (including subtraction of holes: `a -= hole_poly`)
  4. Build Boost polygon_set for B with **negated coordinates** (Minkowski sum with negated B = NFP)
  5. Record B[0] offset (`xshift`, `yshift`) since JS NFPs reference B's first point
  6. Call `convolve_two_polygon_sets(c, a, b)`:
     - For each polygon pair, convolve their edge sequences
     - `convolve_two_segments()` creates a quadrilateral from two edges
     - `convolve_two_point_sequences()` iterates all edge pairs and unions results
     - Handles polygon holes by convolving hole boundaries too
  7. Convert result back to floating-point, add xshift/yshift, return to JavaScript
- **Key advantage:** Handles polygons with holes correctly (unlike ClipperLib MinkowskiSum)

### 3.2 Genetic Algorithm

The GA is implemented in `deepnest.js` (line 1329) and `svgnest.js` (line 745). Both implementations are nearly identical.

**Gene representation:** Each individual is `{ placement: [part1, part2, ...], rotation: [angle1, angle2, ...] }`
- `placement` array defines the **insertion order** of parts
- `rotation` array defines the rotation angle for each part

**Initialization (line 1062-1088):**
1. Create "adam" -- the initial individual with parts sorted by **decreasing area** (first-fit-decreasing heuristic)
2. Assign random rotations from allowed angles (`360/config.rotations` increments, default 4 = 0, 90, 180, 270)
3. Generate `populationSize` individuals (default 10) by mutating adam

**Mutation (`mutate()`, line 1349):**
- For each part in the placement:
  - With probability `mutationRate/100` (default 10%): **swap** with the next part in order
  - With probability `mutationRate/100`: assign a **random rotation** from allowed angles

**Crossover (`mate()`, line 1374):**
- Single-point crossover at a random cut point (between 10% and 90% of gene length)
- Gene 1: first half from male, remaining parts from female (preserving female's order)
- Gene 2: first half from female, remaining parts from male
- Ensures no duplicate parts via ID checking

**Selection (`randomWeightedIndividual()`, line 1440):**
- Population is sorted by fitness (ascending -- lower is better)
- Selection probability is weighted toward the front (fitter individuals)
- Weight function: linearly decreasing probability `2 * weight * (remaining/total)`

**Generation cycle (`generation()`, line 1411):**
1. Sort population by fitness
2. **Elitism:** preserve the best individual
3. Fill remaining slots with offspring of weighted random parents
4. Each child is also mutated

### 3.3 Placement / Evaluation

The placement algorithm is in `background.js`, function `placeParts()` (line 804). Also in `placementworker.js` for the svgnest code path.

**For each sheet (bin):**
1. Open a new sheet, add its area to fitness
2. For each part (in GA-specified order):
   a. Compute **inner NFP** of the sheet and the part (`getInnerNfp()`) -- this defines where the part can be placed within the sheet. Tries all rotation increments if needed.
   b. If this is the first part: place at the **top-left corner** of the inner NFP (minimum x, then minimum y)
   c. For subsequent parts:
      - Compute **outer NFP** of each already-placed part and the current part (`getOuterNfp()`)
      - Shift each outer NFP by the placed part's position
      - **Union** all outer NFPs using ClipperLib
      - **Subtract** the union from the inner NFP using ClipperLib difference
      - The resulting regions are the **feasible placement positions**
      - **Evaluate each vertex** of the feasible region as a candidate position
      - Select the position that minimizes the objective:
        - **Gravity mode** (default): `2 * bounding_width + bounding_height` (compresses horizontally)
        - **Box mode:** `bounding_width * bounding_height`
        - **Hull mode:** area of convex hull of all placed parts
      - If `mergeLines` is enabled, subtract bonus for overlapping collinear edges (for laser cutting efficiency)
3. Parts that couldn't be placed remain for additional sheets
4. **Fitness** = sum of (sheet_area + minwidth/sheetarea + minarea) per sheet + massive penalty for unplaced parts

### 3.4 Rotation Handling

- Allowed rotations are discrete: `0, 360/N, 2*360/N, ...` where N = `config.rotations` (default 4)
- Each part gets a rotation angle stored in the GA's `rotation` array
- Before NFP computation: `rotatePolygon(polygon, degrees)` applies a standard 2D rotation matrix to all points (and recursively to children/holes)
- NFPs are cached with rotation as part of the key: `{A: sourceA, B: sourceB, Arotation: rotA, Brotation: rotB}`
- In svgnest.js, `randomAngle()` also checks if the rotated part fits within the bin dimensions, preferring valid rotations

### 3.5 C++ vs JavaScript Division

| Component | Language | Why |
|-----------|----------|-----|
| Outer NFP (no holes) | JavaScript (ClipperLib MinkowskiSum) | Runs in web workers for parallelism; fast for simple polygons |
| Inner NFP / NFP with holes | C++ (Boost.Polygon) | ClipperLib MinkowskiSum doesn't support holes; Boost handles polygon-with-holes correctly |
| Placement (NFP boolean ops) | JavaScript (ClipperLib) | Union/difference of NFPs, vertex evaluation |
| GA | JavaScript | Lightweight; no need for native code |
| SVG Parsing | JavaScript (DOMParser) | Browser-native SVG parsing |
| Polygon cleaning/offset | JavaScript (ClipperLib) | Robust polygon operations |

---

## 4. Input / Output

### Input Formats
- **SVG** (.svg) -- native format, parsed directly by `SvgParser`. Supports: paths, polygons, polylines, circles, ellipses, rects, images (embedded), lines. Bezier curves are linearized within `curveTolerance`.
- **DXF** (.dxf) and other formats -- converted to SVG via an external conversion server (`config.conversionServer`). The conversion is done by uploading the file to a server endpoint which returns SVG.
- Supported SVG elements for polygonification: `circle`, `ellipse`, `path`, `polygon`, `polyline`, `rect`

### Output Formats
- **SVG** -- direct export of nested layout with transforms
- **DXF** -- converted from SVG via the same external conversion server
- **GCode** -- code exists but is commented out; would use the conversion server

### Internal Polygon Format
- Arrays of `{x, y}` objects
- Holes stored as `.children` array on the polygon array
- Metadata stored as properties on arrays: `.id`, `.source`, `.rotation`, `.offsetx`, `.offsety`

---

## 5. Strengths

1. **Handles irregular/concave shapes:** Unlike rectangular bin packing, Deepnest works with arbitrary polygon shapes including concave containers.
2. **Part-in-part nesting:** Can place parts inside the holes of other parts, maximizing material usage.
3. **NFP-based placement:** Uses the mathematically rigorous No-Fit Polygon approach rather than heuristic bounding box methods, giving tight fits.
4. **Multi-method NFP computation:** Falls back gracefully between three NFP methods (C++ Boost for holes, ClipperLib for simple cases, orbital JS for edge cases).
5. **Aggressive caching:** NFPs are cached by part source and rotation, so re-evaluation across GA generations is fast.
6. **Line merge optimization:** Detects and rewards collinear shared edges between parts -- useful for laser/CNC cutting where shared cut lines save time.
7. **Polygon simplification:** RDP simplification with selective offset for processing complex curves efficiently.
8. **Iterative improvement:** GA continuously refines placement -- users can stop at any time and use the best result found so far.
9. **Desktop app:** Unlike the SVGnest web version, the Electron app can use native C++ addons for faster computation.

---

## 6. Weaknesses

1. **Outdated Electron version:** Uses Electron 1.4 (from 2016), with deprecated APIs throughout (`makeSingleInstance`, old V8 API in C++ code, etc.).
2. **Hardcoded Boost path:** `binding.gyp` contains `/Users/jackqiao/boost_1_62_0/` -- requires manual editing to build.
3. **Single background worker:** Despite having infrastructure for multiple background windows, the code limits to 1 (`winCount < 1`), serializing nesting evaluations.
4. **No true multi-threading in JS:** The background window runs placement synchronously. Parallel.js web workers are used for NFP computation but the placement itself is single-threaded.
5. **External conversion server dependency:** DXF import/export requires a running conversion server (not included in the repo). This is a hard dependency for non-SVG formats.
6. **Memory management issues:** NFP cache is stored in `window.nfpcache` (in-memory) with a simple heap check (`totalJSHeapSize < 0.8 * jsHeapSizeLimit`). Large nesting jobs can consume excessive memory.
7. **Floating-point precision issues:** The C++ code acknowledges precision uncertainty (`"why 0.1? dunno. it doesn't screw up with 0.1"` at `minkowski.cc` line 163). The integer scaling approach can lose precision for very small or very large coordinates.
8. **No concavity handling by default:** The `exploreConcave` flag is off by default in svgnest.js and the orbital NFP is not used in the primary Deepnest code path (which uses ClipperLib/C++ Minkowski). The Minkowski sum approach doesn't find inner NFP pockets from concavities.
9. **Duplicated code:** The nesting logic exists in two parallel implementations (svgnest.js + placementworker.js for the web worker path, and deepnest.js + background.js for the Electron IPC path), with subtle differences between them.
10. **No rotation optimization for rectangular bins:** Rotation choices are purely random within GA; no heuristic to prefer rotations that align flat edges with bin boundaries.
11. **Placement heuristic limitations:** The gravity-direction fitness (`2*width + height`) is fixed and not configurable. Other placement strategies (e.g., bottom-left with angle bisector) are not implemented.
12. **No support for curved bins:** All geometry is polygonized before nesting.
13. **IPC serialization overhead:** Sending polygon data between Electron windows via IPC (JSON serialization) for every GA individual is slow. Array metadata (`.children`, `.id`, `.source`) must be manually separated and reassembled because IPC strips non-index properties from arrays.

---

## 7. Key Code Paths

### NFP Computation

| Location | Function | Lines | Description |
|----------|----------|-------|-------------|
| `main/util/geometryutil.js` | `noFitPolygon(A, B, inside, searchEdges)` | 1437-1727 | JavaScript orbital NFP algorithm |
| `main/util/geometryutil.js` | `noFitPolygonRectangle(A, B)` | 1379-1432 | Fast inner NFP for rectangular A |
| `main/util/geometryutil.js` | `searchStartPoint(A, B, inside, NFP)` | 1245-1360 | Find valid start position for NFP orbit |
| `main/util/geometryutil.js` | `polygonSlideDistance(A, B, direction)` | 1122-1185 | Maximum slide distance along direction without intersection |
| `main/util/geometryutil.js` | `segmentDistance(A, B, E, F, direction)` | 958-1120 | Distance between two line segments in a given direction |
| `minkowski.cc` | `calculateNFP()` | 106-269 | C++ Minkowski sum NFP via Boost.Polygon |
| `minkowski.cc` | `convolve_two_polygon_sets()` | 70-92 | Core Boost convolution of two polygon sets |
| `minkowski.cc` | `convolve_two_point_sequences()` | 33-56 | Convolve two edge sequences |
| `main/background.js` | `getOuterNfp(A, B, inside)` | 620-710 | Dispatches to C++ addon or ClipperLib MinkowskiSum |
| `main/background.js` | `getInnerNfp(A, B, config)` | 734-801 | Computes inner NFP using a frame trick + outer NFP |
| `main/background.js` | `process(pair)` (within IPC handler) | 173-249 | Parallel web worker function for ClipperLib Minkowski NFP |

### Genetic Algorithm

| Location | Function | Lines | Description |
|----------|----------|-------|-------------|
| `main/deepnest.js` | `GeneticAlgorithm(adam, config)` | 1329-1346 | Constructor: creates initial population from adam |
| `main/deepnest.js` | `GeneticAlgorithm.prototype.mutate()` | 1349-1371 | Swap-adjacent + random-rotation mutation |
| `main/deepnest.js` | `GeneticAlgorithm.prototype.mate()` | 1374-1409 | Single-point crossover producing two children |
| `main/deepnest.js` | `GeneticAlgorithm.prototype.generation()` | 1411-1437 | New generation: elitism + crossover + mutation |
| `main/deepnest.js` | `GeneticAlgorithm.prototype.randomWeightedIndividual()` | 1440-1463 | Weighted selection (front-biased) |

### Placement

| Location | Function | Lines | Description |
|----------|----------|-------|-------------|
| `main/background.js` | `placeParts(sheets, parts, config, nestindex)` | 804-1173 | Main placement loop for Deepnest -- greedy placement using NFP + Clipper booleans |
| `main/util/placementworker.js` | `PlacementWorker.placePaths()` | 59-289 | Placement for svgnest.js code path (web worker) |
| `main/background.js` | `mergedLength(parts, p, minlength, tolerance)` | 352-487 | Compute length of collinear shared edges (for merge line bonus) |

### SVG Parsing & Part Extraction

| Location | Function | Lines | Description |
|----------|----------|-------|-------------|
| `main/svgparser.js` | `SvgParser.prototype.load()` | 38-130 | Parse SVG string, handle scaling/units |
| `main/svgparser.js` | `SvgParser.prototype.clean()` | ~131+ | Flatten transforms, remove invisible elements |
| `main/svgparser.js` | `SvgParser.prototype.polygonify()` | ~300+ | Convert SVG element to polygon point array |
| `main/deepnest.js` | `DeepNest.getParts()` | 642-922 | Extract parts from SVG children, build parent-child tree |
| `main/deepnest.js` | `DeepNest.simplifyPolygon()` | 145-475 | RDP simplification with selective offset reversal |
| `main/deepnest.js` | `DeepNest.cleanPolygon()` | 1180-1216 | Remove self-intersections via ClipperLib.SimplifyPolygon |

### Orchestration

| Location | Function | Lines | Description |
|----------|----------|-------|-------------|
| `main/deepnest.js` | `DeepNest.start()` | 943-1016 | Entry point: apply offsets, start worker timer |
| `main/deepnest.js` | `DeepNest.launchWorkers()` | 1040-1153 | Check GA state, dispatch individuals to background via IPC |
| `main/deepnest.js` | IPC handler `background-response` | 1018-1038 | Receive placement results, update nests |
| `main/background.js` | IPC handler `background-start` | 112-347 | Receive job, compute NFPs, run placement, return results |
| `main.js` | IPC routing | 174-217 | Route messages between main window and background windows |
