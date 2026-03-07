# 2D-Irregular-Cutting-Stock-Problem

## Overview

A Python implementation of two metaheuristic approaches -- Particle Swarm Optimization (PSO) and Differential Evolution with Global and Local mutations (DEGL) -- for the 2D irregular cutting stock problem. Unlike strip packing, this formulation uses multiple heterogeneous bins (irregular stock pieces) and processes multiple orders sequentially. Developed as a research project, it uses Shapely for all geometric operations and includes a two-stage evaluation: a metaheuristic optimizes placement parameters, then a Bottom-Left Fill (BLF) fine-tuning step adjusts positions to reduce waste.

## Architecture

```
.
+-- runPSO.py              # Main PSO execution script (~595 lines)
+-- runDEGL.py             # Main DEGL execution script (~579 lines)
+-- DynNeighborPSO.py      # PSO with dynamic random neighborhood (~368 lines)
+-- DEGL.py                # Differential Evolution (DEGL) optimizer (~392 lines)
+-- WoodProblemDefinition.py # Problem definition: 8 stock pieces + 3 orders
+-- demo_shapely.py        # Shapely/descartes visualization helper (~197 lines)
```

### Key Files

- **`WoodProblemDefinition.py`**: Defines the problem instance. Contains 8 irregular stock pieces (`Stock`) as Shapely Polygons (some with holes) and 3 orders (`Order1`, `Order2`, `Order3`) each containing multiple polygons to be placed. Stock pieces include complex shapes like L-shapes and polygons with rectangular cutouts.
- **`runPSO.py`**: The primary execution script. Implements the objective function, BLF fine-tuning, smoothness metric, and orchestrates the PSO optimization. Each particle encodes (x, y, rotation, bin_index) for every polygon in the order. Processes orders sequentially, updating bin availability after each.
- **`runDEGL.py`**: Nearly identical to `runPSO.py` but uses DEGL instead of PSO. Shares the same objective function formulation and BLF logic.
- **`DynNeighborPSO.py`**: PSO variant with dynamic random neighborhood topology. Uses adaptive inertia weight that increases when the swarm stalls. Supports boundary handling via reflection.
- **`DEGL.py`**: Differential Evolution with combined global and local mutation strategies. The mutation weight `W` blends local neighborhood-best and global-best difference vectors.

## Algorithm Details

### Encoding

Each solution is a vector of length `4 * N` where N is the number of polygons in the current order. For polygon `i`:
- `x[4*i]`: x-position (continuous, within bin bounds)
- `x[4*i+1]`: y-position (continuous, within bin bounds)
- `x[4*i+2]`: rotation angle in degrees (continuous, 0-360)
- `x[4*i+3]`: bin index (continuous, rounded to integer 0-7 for bin assignment)

### Objective Function

The fitness function (`ObjectiveFcn`) computes a weighted sum of three terms:

```
fitness = 0.300 * overlap + 0.695 * penalty + 0.005 * waste
```

- **Overlap** (weight 0.300): Sum of intersection areas between all pairs of placed polygons within the same bin, normalized by total polygon area.
- **Penalty** (weight 0.695): Sum of areas where polygons extend outside their assigned bin boundaries, normalized by total polygon area. Computed as `polygon.area - polygon.intersection(bin).area`.
- **Waste** (weight 0.005): Fraction of used bin area that is wasted. Only bins containing at least one polygon contribute. Computed as `1 - (total_polygon_area_in_bin / bin_area)`.

The heavy penalty weight ensures containment is prioritized over overlap elimination, which in turn is prioritized over waste minimization.

### BLF Fine-Tuning

After the metaheuristic finds a solution, a Bottom-Left Fill refinement pass adjusts each polygon:
1. For each polygon (sorted by placement order), attempt to slide it left and down.
2. Use small step sizes to incrementally move polygons toward the bottom-left of their bin.
3. Check containment and non-overlap at each step using Shapely's `within()` and `intersects()`.
4. A "smoothness" metric measures how well polygons nestle against bin edges and each other.

### PSO (DynNeighborPSO)

- **Dynamic Random Neighborhood**: Each particle's neighborhood is randomly reassigned periodically, preventing premature convergence.
- **Adaptive Inertia**: Inertia weight `w` starts at `wdamp * w` each iteration but increases when the global best stagnates (stall counter), encouraging exploration.
- **Velocity Clamping**: Velocities are clamped to `[-VelMax, VelMax]` where `VelMax = 0.1 * (UB - LB)`.
- **Boundary Handling**: Positions that exceed bounds are reflected back: `x = 2*bound - x`.
- **Parameters**: Population 100, 600 iterations, `c1=c2=1.4962`, `w=0.7298`.

### DEGL (Differential Evolution)

- **Combined Mutation**: For each individual, computes both a local mutation (using neighborhood best) and a global mutation (using population best), then blends them: `y = W * y_global + (1-W) * y_local`.
- **Local Mutation**: `x_i + alpha * (x_nbest - x_i) + beta * (x_r1 - x_r2)` where `x_nbest` is the best in the neighborhood ring.
- **Global Mutation**: `x_i + alpha * (x_gbest - x_i) + beta * (x_r1 - x_r2)` with globally random `r1, r2`.
- **Crossover**: Binomial crossover with rate `CR`.
- **Self-Adaptive W**: Weight `W` evolves per individual using a logistic-normal update scheme.
- **Parameters**: Population 100, 600 iterations, `CR=0.9`, `alpha=beta=0.5`, neighborhood radius `k=3`.

### Sequential Order Processing

Orders are processed one at a time. After placing all polygons in Order 1, the bins used (those with polygons) are marked, and subsequent orders can only use remaining capacity. This models a real cutting stock workflow where material is consumed across jobs.

## Dependencies

- **Python 3.6+**
- **shapely** -- all geometric operations (intersection, union, within, area computation)
- **descartes** -- converting Shapely polygons to matplotlib patches for visualization
- **numpy** -- numerical operations, random number generation
- **matplotlib** -- visualization of placements

## Strengths

- **Realistic problem formulation**: Models heterogeneous irregular bins (not just rectangular strips), polygons with holes, and sequential multi-order processing -- closer to real-world cutting stock than most academic implementations.
- **Two metaheuristic implementations**: PSO and DEGL provide comparative baselines with nearly identical evaluation pipelines.
- **Weighted multi-objective**: The three-term fitness function (overlap + penalty + waste) with carefully chosen weights provides a smooth optimization landscape that prioritizes feasibility.
- **BLF post-processing**: The fine-tuning step improves solutions found by the metaheuristic, combining global search with local refinement.
- **Self-contained**: All problem data is defined inline in `WoodProblemDefinition.py`, no external data files needed.

## Weaknesses

- **Hardcoded problem instance**: Only one problem instance (8 stocks, 3 orders) is defined. No data loader or parameterization for other instances.
- **Massive code duplication**: `runPSO.py` and `runDEGL.py` are ~95% identical (~580 lines each), differing only in which optimizer is called. The objective function, BLF, and visualization code are copy-pasted.
- **Slow evaluation**: Every fitness evaluation performs multiple Shapely polygon intersections (O(N^2) pair checks), making large instances prohibitively slow. No NFP computation or caching.
- **No NFP usage**: Unlike most nesting implementations, this code does not compute No-Fit Polygons. All overlap/containment checks use raw Shapely boolean operations, which is significantly slower and less precise for optimization.
- **Continuous bin index encoding**: The bin assignment dimension is continuous (0.0-7.0) but rounded to integer, creating a non-smooth search landscape for the metaheuristic.
- **Limited rotation handling**: Rotation is encoded as a continuous 0-360 degree value, but the metaheuristic has no special operator to exploit rotational symmetry or discrete rotation sets.
- **No packaging**: No `requirements.txt`, `setup.py`, or proper module structure.

## Key Code Paths

1. **PSO Optimization**: `runPSO.py` main block -> `DynNeighborPSO.Optimize()` -> for each iteration: evaluate `ObjectiveFcn(x)` for each particle -> update personal best -> update neighborhood best -> update velocity and position with adaptive inertia.

2. **Objective Evaluation**: `runPSO.py::ObjectiveFcn(x)` -> decode (x, y, rotation, bin) for each polygon -> rotate polygon by angle, translate to (x, y) -> compute pairwise `intersection().area` for overlap term -> compute `polygon.area - polygon.intersection(bin).area` for penalty term -> compute waste as unused bin fraction -> return weighted sum.

3. **BLF Fine-Tuning**: `runPSO.py` post-optimization block -> for each polygon: try shifting left (decrement x by step), check `within(bin)` and `not intersects(others)` -> try shifting down similarly -> repeat until no improvement.

4. **DEGL Optimization**: `runDEGL.py` main block -> `DEGL.Optimize()` -> for each generation: compute local and global mutations for each individual -> combine with weight W -> binomial crossover -> greedy selection -> update W via self-adaptive scheme.

5. **Problem Setup**: `WoodProblemDefinition.py` -> creates `Stock` list of 8 Shapely Polygons (some via `Polygon(...).difference(hole)` for holes) -> creates `Order1/2/3` lists of demand polygons -> imported by `runPSO.py`/`runDEGL.py`.
