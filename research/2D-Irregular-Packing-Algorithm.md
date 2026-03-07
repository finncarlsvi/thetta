# 2D-Irregular-Packing-Algorithm

## Overview

A comprehensive research-oriented repository implementing multiple algorithms for 2D irregular strip packing (nesting). Developed by Shan Yang and Zilu Wang at Tongji University, it serves as both a tutorial codebase and a testbed for various nesting strategies. The code targets the classic "irregular strip packing problem" -- placing irregular polygons onto a strip of fixed width while minimizing the used length.

## Architecture

The project is structured as a collection of standalone algorithm scripts sharing a common `tools/` library:

```
.
+-- bottom_left_fill.py       # BLF placement + TOPOS heuristic (also in TOPOS.py)
+-- genetic_algorithm.py       # GA + SA metaheuristics over BLF
+-- simulating_annealing.py    # SA-only version
+-- cuckoo_search.py           # Guided Cuckoo Search (GCS)
+-- fast_neighbor_search.py    # Fast Neighborhood Search (FNS)
+-- lp_search.py               # LP-based search with penetration depth
+-- compaction_separation.py   # LP-based Compaction & Separation
+-- nfp_test.py                # NFP generation testing
+-- tools/
|   +-- nfp.py                 # No-Fit Polygon computation (edge-sliding)
|   +-- packing.py             # Core: BLF, NFPAssistant, PackingUtil, PolyListProcessor, GeoFunc
|   +-- geofunc.py             # Geometry helper (re-exported in packing.py)
|   +-- data.py                # Dataset loader (CSV -> polygon lists)
|   +-- show.py                # Matplotlib visualization
|   +-- lp.py                  # LP solver wrapper
|   +-- lp_assistant.py        # LP geometry helpers
|   +-- assistant.py           # Output formatting
+-- data/                      # EURO benchmark datasets (CSV), pre-computed NFPs
+-- C++ Nesting Problem/       # Partial C++ port (Xcode project)
```

### Key Files

- **`tools/packing.py`** (~900 lines): The workhorse. Contains `BottomLeftFill`, `NFPAssistant` (NFP caching/lookup), `PackingUtil` (inner-fit rectangle), `PolyListProcessor` (sequence manipulation), `GeoFunc` (geometry primitives), and `RatotionPoly`.
- **`tools/nfp.py`** (~290 lines): Implements the orbiting/edge-sliding NFP algorithm from Burke et al. 2007. Computes the No-Fit Polygon by sliding one polygon around another and tracing the locus of a reference point.
- **`lp_search.py`** (~625 lines): The most sophisticated algorithm -- a custom approach using penetration depth (PD) and guided search. Iteratively minimizes overlap via LP-like optimization with NFP-based PD computation, shrinking/extending the strip boundary.
- **`genetic_algorithm.py`**: GA with ordered crossover and swap/rotate mutations, using BLF as the evaluation function.
- **`compaction_separation.py`**: Linear programming formulations for Compaction (minimize strip length) and Separation (eliminate overlap), referencing the SHAH hybrid approach.

## Algorithm Details

### Placement Heuristics
- **Bottom Left Fill (BLF)**: Places polygons sequentially. For each new polygon, computes the Inner Fit Rectangle (IFR) for the strip, then subtracts all NFPs with already-placed polygons to find feasible placement points. Selects the bottom-left-most feasible point.
- **TOPOS**: Similar sequential placement but minimizes the change in bounding rectangle width rather than using bottom-left priority. Based on Bennell's TOPOS Revised.

### Metaheuristic Sequence Optimization
- **Genetic Algorithm (GA)**: Evolves the placement order. Uses elite selection, ordered crossover (OX), and swap/rotate mutations. Fitness = strip length from BLF evaluation. Population size 20, 10 generations default.
- **Simulated Annealing (SA)**: Perturbs placement order via random swaps. Accepts worse solutions probabilistically per Metropolis criterion. Cooling schedule: multiplicative (rate 0.7).

### Layout-Based Optimization
- **Guided Cuckoo Search (GCS)**: Operates on polygon positions directly (not sequence). Uses Levy flights for exploration, minimizes penetration depth with adaptive penalty weights. Iteratively shrinks/expands the strip height.
- **Fast Neighborhood Search (FNS)**: Greedy approach from 2004 paper. For the polygon with maximum overlap, tests horizontal and vertical slides to find positions minimizing total intersection area. Uses break-point analysis for quadratic overlap functions.
- **LP Search**: Custom algorithm by the authors. Shrinks the strip boundary, then uses guided search over NFP intersection points and IFR vertices to find minimum-PD placements. Uses a `miu` penalty weight matrix updated per iteration. Caches PD computations at grid/digital precision levels.

### Linear Programming
- **Compaction**: LP formulation to minimize strip length while maintaining non-overlap constraints derived from NFP edge distances.
- **Separation**: LP formulation to eliminate overlap by moving polygons, minimizing total slack variables representing overlap.

### NFP Computation
The NFP is computed via edge-sliding (orbiting): polygon B slides around the boundary of fixed polygon A, and the trace of B's reference point forms the NFP. Touching edges are detected, potential translation vectors computed, and feasibility checked via cross-product tests. Vectors are trimmed at intersection points.

## Dependencies

- **Python 3.7+**
- **shapely** -- polygon operations (intersection, difference, union, containment)
- **numpy** -- numerical operations
- **pandas** -- CSV data loading
- **matplotlib** -- visualization
- **pyclipper** -- polygon clipping (imported in packing.py)
- **interval** -- interval arithmetic (used in FNS)
- LP solver (referenced in `tools/lp.py`)

## Strengths

- **Breadth of algorithms**: Implements 7+ distinct approaches from the nesting literature, making it valuable for comparative research.
- **Complete NFP implementation**: Edge-sliding NFP from scratch, not relying on external nesting libraries.
- **NFP caching**: `NFPAssistant` stores and loads pre-computed NFPs, significantly accelerating repeated evaluations.
- **Benchmark datasets**: Includes processed EURO benchmark instances (albano, blaz, dagli, fu, jakobs, etc.) ready for testing.
- **LP Search**: The authors' custom algorithm (`lp_search.py`) is the most developed, with multi-level PD caching (grid/digital precision), symmetry detection, and best-known-result thresholds for various datasets.

## Weaknesses

- **Code quality**: Significant amount of Chinese comments and variable names mixed with English, hardcoded file paths (e.g., `/Users/sean/Documents/Projects/Data/`), and duplicated code across files (BLF class appears in both `bottom_left_fill.py` and `tools/packing.py`).
- **Known bugs**: TOPOS has a typo (`self.border_heigt` instead of `self.border_height`), FNS and Cuckoo Search are noted as incomplete/buggy in the README.
- **No packaging**: No `setup.py`, `requirements.txt`, or proper module structure. Import paths are fragile.
- **LP Search coupling**: `lp_search.py` requires pre-computed NFP CSV files and an `lp_initial.csv` configuration file with specific column formats, making it hard to run on new datasets.
- **Performance**: Pure Python NFP computation is slow for large instances. The C++ port is incomplete.
- **Only strip packing**: Does not support bin packing (multiple sheets), only the single-strip variant with fixed width.

## Key Code Paths

1. **BLF Evaluation**: `genetic_algorithm.py::packingLength()` -> `BottomLeftFill.__init__()` -> for each polygon: `placePoly()` -> compute IFR via `PackingUtil.getInnerFitRectangle()`, subtract NFPs via `NFPAssistant.getDirectNFP()`, select bottom-left point.

2. **NFP Computation**: `tools/nfp.py::NFP.main()` -> `detectTouching()` (find edge-edge contacts) -> `potentialVector()` (candidate translation vectors) -> `feasibleVector()` (filter by cross-product tests) -> `trimVector()` (shorten at intersections) -> slide polygon and record locus point.

3. **LP Search**: `lp_search.py::LPSearch.main()` -> `shrinkBorder()` -> `minimizeOverlap()` -> for each polygon in random permutation: `lpSearch()` -> compute IFR, get all current NFPs via `getCurNFPs()`, find NFP-IFR intersection points and NFP-NFP intersection points, evaluate PD at each candidate point via `getPolyPtPD()`, select minimum.

4. **GA Optimization**: `genetic_algorithm.py::GA.geneticAlgorithm()` -> initialize random population -> for each generation: `getLengthRanked()` (evaluate all via BLF) -> `getNextGeneration()` (elite selection + crossover + mutation).
