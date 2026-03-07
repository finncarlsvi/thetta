# Deepnest-Next Analysis

## 1. Overview

Deepnest-next is an actively maintained fork of Deepnest, the open-source 2D nesting tool originally based on SVGNest. It is a desktop Electron application targeting plotter, laser cutter, and CNC tool users. The project is at version 1.5.6 and is maintained by the deepnest-next community (primary contributor: Josef Frohle). It uses a genetic algorithm to optimize part ordering and rotation, with NFP-based placement evaluation in a background worker process.

Fork lineage: SVGNest -> Deepnest (Jack000) -> Dogthemachine -> cmidgley -> deepnest-io -> deepnest-next.

## 2. Architecture

Electron desktop application with JavaScript/TypeScript frontend:

```
main.js               - Electron main process entry point
main/
  deepnest.js         - Core nesting engine (DeepNest class): SVG import, polygon processing, GA orchestration
  background.js       - Background worker: NFP computation, placement algorithm, fitness evaluation
  svgparser.js        - SVG file parsing
  nfpDb.ts            - NFP caching database
  ui/
    index.ts           - UI entry point
    components/
      navigation.ts    - Tab navigation
      nest-view.ts     - Nesting visualization
      parts-view.ts    - Parts list management
      sheet-dialog.ts  - Sheet configuration
    services/
      config.service.ts    - Configuration management
      export.service.ts    - SVG/DXF export
      import.service.ts    - File import
      nesting.service.ts   - Nesting orchestration
      preset.service.ts    - Settings presets
    utils/
      conversion.ts    - Unit conversion
      dom-utils.ts     - DOM helpers
      ui-helpers.ts    - UI utilities
  util/
    geometryutil.js    - Geometry utility functions (polygon area, intersection, etc.)
    clipper.js         - Clipper library for polygon boolean operations
    parallel.js        - Worker thread management
    simplify.js        - Polygon simplification (RDP)
    HullPolygon.ts     - Convex hull computation
    point.ts / vector.ts / matrix.ts  - Math primitives
```

**Key files:**
- `main/deepnest.js` - Main nesting engine (DeepNest class)
- `main/background.js` - Background placement worker
- `main/util/geometryutil.js` - Core geometry utilities

## 3. Algorithm Details

### Overall Architecture
The nesting uses a two-layer approach:
1. **Genetic Algorithm (main process)** - Evolves populations of part orderings and rotations
2. **Placement Worker (background process)** - Evaluates each individual via greedy NFP-based placement

### Genetic Algorithm
Configuration: population size 10, mutation rate 10%, configurable rotation steps (default 4: 0/90/180/270).

Each individual is a chromosome containing:
- `placement`: array of part references (order)
- `rotation`: array of rotation angles

### NFP-Based Placement (background.js)
The placement engine:
1. **Pre-compute NFP pairs** - For each part pair at given rotations, compute outer NFPs (collision boundaries) and inner NFPs (valid region inside sheet)
2. **NFP computation** - Delegated to `@deepnest/calculate-nfp` native Node.js addon (C-based, speed-critical)
3. **Greedy placement** - Place parts one at a time using NFP intersection to find valid positions
4. **Placement strategies**:
   - **Gravity** - Minimize Y-position (bottom placement)
   - **Bounding box** - Minimize bounding box area
   - **Convex hull** - Minimize convex hull area
5. **Hole fitting** - Smaller parts can be placed inside holes of larger parts
6. **Fitness scoring** - Based on material utilization; lower fitness = better (for GA minimization)

### NFP Caching
`NfpCache` (nfpDb.ts) caches computed NFPs keyed by (source_A, source_B, rotation_A, rotation_B). This avoids recomputing expensive NFPs across GA generations.

### Polygon Processing Pipeline
1. SVG/DXF import with scaling
2. Polygon simplification (RDP algorithm with tolerance)
3. Polygon offsetting (for part spacing) via Clipper library
4. Convex hull computation
5. Polygon cleaning (remove self-intersections, coincident points)

## 4. Dependencies

**Runtime:**
- `@deepnest/calculate-nfp` 202503.13 - Native C addon for NFP computation (speed-critical)
- `@deepnest/svg-preprocessor` 0.2.0 - SVG preprocessing and simplification
- `@electron/remote` 2.1.3 - Electron remote module
- `axios` 1.13 - HTTP client
- `graceful-fs` 4.2 - File system operations
- `marked` 17.0 - Markdown rendering

**Development:**
- `electron` 40.0 - Desktop framework
- `typescript` 5.8 - TypeScript compilation
- `@playwright/test` 1.57 - E2E testing
- `@electron/packager` 18.3 - Distribution packaging
- `eslint` 9.26 + `prettier` 3.5 - Code quality

**Vendored:**
- `clipper.js` - Clipper library (polygon boolean operations) bundled in util/
- `geometryutil.js` - Custom geometry utilities
- `simplify.js` - RDP simplification

## 5. Strengths

- **Actively maintained** - Regular updates, Electron 40, modern tooling (ESLint 9, TypeScript 5.8, Playwright tests)
- **End-user application** - Full GUI with SVG/DXF import, visual nesting results, export capabilities
- **Native NFP module** - Speed-critical NFP computation in C via `@deepnest/calculate-nfp`
- **Practical features** - Common line merging for laser cuts, part spacing, DXF support, presets
- **Established algorithm** - Well-tested SVGNest-derived approach with years of community validation
- **Hole fitting** - Can place parts inside holes of other parts
- **Multiple placement strategies** - Gravity, bounding box, convex hull minimization
- **Community** - Patreon funding, active issue tracking, contributor guidelines

## 6. Weaknesses

- **JavaScript core** - Main nesting logic in JS; only NFP computation is native. Significant performance ceiling.
- **Small GA population** - Default population size 10 is very small for metaheuristic optimization
- **Single placement strategy per run** - No adaptive strategy selection
- **Electron overhead** - Desktop-only, heavy runtime (~200MB+ for Electron)
- **Legacy code** - Significant amount of inherited SVGNest code with debugging artifacts and commented-out code
- **No 3D support** - 2D only
- **Limited rotation options** - Default 4 rotations (90-degree increments only)
- **Integer-based Clipper** - Clipper library requires integer coordinates, so all coordinates are scaled by 10,000,000
- **No headless/API mode** - Requires Electron GUI; no library or CLI mode for batch processing
- **Planned Rust migration** - README mentions "upcoming" Rust rewrite but not yet implemented

## 7. Key Code Paths

**Nesting flow:**
1. `DeepNest.importsvg()` in `main/deepnest.js` -> parse SVG, extract parts
2. User starts nesting -> GA initialization with part orderings + rotations
3. Each GA individual sent to background worker via IPC (`background-start` event)
4. `background.js`: precompute NFP pairs -> call `window.addon` (native NFP) -> greedy placement -> compute fitness
5. Results returned to main process -> GA selection/crossover/mutation -> next generation
6. Best result displayed in nest-view UI component

**NFP pair computation:**
`background.js` line ~86+: iterate all part pairs -> check NFP cache -> if miss, call `window.addon.calculateNFP()` -> cache result

**Placement:**
Greedy loop over parts in GA-determined order -> for each part, compute intersection of inner-NFP (sheet boundary) minus all outer-NFPs (placed parts) -> select position minimizing fitness criterion (gravity/bbox/hull)
