# 2DIBPP-MergeDJD Analysis

## 1. Overview

MergeDJD is a Java-based implementation of a constructive heuristic algorithm for the Two-Dimensional Irregular Bin Packing Problem (2DIBPP). It improves upon the DJD (Djang and Finch) heuristic by incorporating piece merging. The project is packaged as a Spring Boot web application with a browser-based visualization interface. It handles convex and non-convex polygonal pieces (no curves) packed into rectangular bins.

The project appears to be an academic research implementation accompanying a paper on the MergeDJD algorithm.

## 2. Architecture

Java Spring Boot web application (JDK 1.8):

```
src/main/java/com/example/demo/
  DemoApplication.java          - Spring Boot entry point
  controller/
    PolygonController.java      - REST API for nesting operations
    TestController.java         - Test endpoints
  share/                        - Core algorithm implementation
    Piece.java                  - Polygon piece representation (int coords, rotation, area)
    Sheet.java                  - Bin/sheet representation
    Instance.java               - Problem instance (pieces + sheets)
    Heuristics.java             - Main heuristic algorithms (DJD, FFD, BFD, Filler)
    HeuristicsPlacement.java    - Placement sub-heuristics (BL, EC, EC2, MA, MABR, MALBR)
    ControlHeuristics.java      - Heuristic orchestration and fitness calculation
    RunHeuristics.java          - Batch execution runner
    RePlacement.java            - Re-placement/improvement procedures
    ResultVisual.java           - Result visualization
    MyPiece.java / PieceVirtual.java  - Extended piece types for merging
    RWfiles.java                - File I/O
    ArrayOperations.java        - Array utilities
  nest/                         - NFP-related data structures
    data/
      NestPath.java             - Polygon path representation
      Segment.java              - Line segment
      Bound.java                - Bounding box
      Vector.java               - 2D vector
    util/
      GeometryUtil.java         - Geometry utilities
      CommonUtil.java           - Common utilities
      Config.java               - Configuration
      coor/                     - Coordinate conversion (Clipper <-> Nest)
  de/lighti/clipper/            - Bundled Java Clipper library (polygon booleans)
  utils/
    InputData.java              - Input data parsing
    Polygon.java / Points.java / Container.java  - Data model for web API
```

**Key files:**
- `share/Heuristics.java` - Core algorithm implementations (DJD, FFD, BFD, Filler)
- `share/HeuristicsPlacement.java` - Six placement heuristics
- `share/ControlHeuristics.java` - Strategy selection and fitness calculation
- `share/Piece.java` - Polygon representation with integer coordinates

## 3. Algorithm Details

### DJD (Djang and Finch) Heuristic
The core algorithm `Djang_and_Finch_2D()` in `Heuristics.java`:

1. **Sort pieces** by area (descending)
2. **Capacity threshold** - Start with an initial capacity threshold (e.g., 33% of sheet area)
3. **Greedy packing** - Try to place single pieces, then pairs, then triples that together fill the remaining capacity
4. **Piece merging (DJD++)** - When single pieces don't fit, try combining 2 or 3 pieces to find a combination that fills the target capacity. Uses "unfit" lookup tables (`pieceUnfit`, `pieceUnfit2`, `pieceUnfit3`) to avoid retrying known failures.
5. **Capacity increment** - If no pieces fit at current threshold, increase threshold by 1/20 of sheet area
6. **New bin** - Open a new bin when current one reaches capacity

Two variants controlled by `type` parameter:
- Type 0: Standard DJD
- Type 1: DJD with merging enhancements

### Placement Heuristics
Six placement strategies in `HeuristicsPlacement.java`:

1. **BL (Bottom-Left)** - Move piece to top-right corner, then slide toward bottom-left until valid position found
2. **EC (Edge Contact)** - Try all 4 rotations, for each: try placing at existing piece contact points, select position with minimum X (then Y as tiebreaker)
3. **EC2** - Like EC but optimizes for minimum bounding area (X*Y) rather than minimum X
4. **MA (Maximum Area)** - Placement maximizing area utilization
5. **MABR** - MA variant with bottom-right bias
6. **MALBR** - MA variant with left-bottom-right bias (default for DJD)

### Placement Validation
`posicionValida()` checks:
- Piece within sheet bounds (coordinates >= 0 and <= sheet max)
- No overlap with already-placed pieces (polygon intersection test)

### Fitness Calculation
`ControlHeuristics.calcularAptitud()`: Sum of (utilization ratio)^2 per bin, averaged over all bins. This rewards high utilization per bin.

The K-value metric (`calculateK()`) computes fractional bin count based on how much of the last bin is used.

### Coordinate System
All coordinates are **integers** -- the README notes that input data should be multiplied by a scaling factor for accuracy.

## 4. Dependencies

- **Spring Boot** 2.6.13 - Web framework
- **jxl** 2.6.12 - Excel file reading (JExcelAPI)
- **Lombok** 1.16.10 - Java boilerplate reduction
- **Clipper** (bundled) - Java port of the Clipper polygon clipping library (de.lighti.clipper package)
- **JDK 1.8** - Java 8

Maven-based build system.

## 5. Strengths

- **Complete DJD implementation** - Full implementation of the Djang and Finch heuristic with piece merging enhancement
- **Multiple placement strategies** - Six different placement heuristics (BL, EC, EC2, MA, MABR, MALBR)
- **Piece merging** - The key innovation: trying combinations of 2-3 pieces to fill capacity gaps
- **Unfit tables** - Memoization of failed placement attempts avoids redundant computation
- **Web visualization** - Browser-based UI for interactive demonstration
- **Batch processing** - `RunHeuristics` supports batch execution over multiple problem instances
- **Multiple input formats** - JSON, TXT, and Excel input support
- **Fitness metrics** - Multiple evaluation metrics (aptitude, K-value, utilization)
- **Handles non-convex polygons** - Via Clipper library for polygon operations

## 6. Weaknesses

- **Integer coordinates only** - All piece coordinates are integers, limiting precision. Requires pre-scaling.
- **No metaheuristic optimization** - Purely constructive heuristic; no GA/SA/ALNS wrapper for solution improvement
- **Java 8** - Outdated language version; no modern Java features
- **Hardcoded limits** - `objectsMaximum = 30`, `piecesMaximum = 200` as fixed array dimensions
- **O(n^3) complexity** - The triple-piece merging uses three nested loops with unfit table lookups
- **Simple overlap detection** - Uses coordinate-based intersection rather than robust NFP-based collision checking
- **No rotation granularity** - Only 4 rotations (0/90/180/270 degrees) tested
- **Mixed language code** - Variable names and comments in Spanish/English mix reduces readability
- **Web framework overhead** - Spring Boot is heavy for what is essentially a computation engine
- **No curved shapes** - Polygons only, no arc/curve support

## 7. Key Code Paths

**Main execution flow:**
1. `PolygonController` receives REST request with polygon data
2. `RunHeuristics.run_input_data()` creates problem instance
3. `Instance.execute()` or `Instance.ejecutaAccion()` calls `ControlHeuristics.executeHeuristic()`
4. `ControlHeuristics.executeHeuristic()` dispatches to selected heuristic (default: case 1 = DJD with MALBR)

**DJD algorithm:**
`Heuristics.Djang_and_Finch()` -> `Djang_and_Finch_2D()`:
1. Sort pieces by area descending (`OrderPieces`)
2. For each piece, try placement on current bin
3. If single piece fails, try pairs (`searchPair2`) then triples (`searchPair3`)
4. Increment capacity threshold if all fail
5. Open new bin when needed

**Placement:**
`HeuristicsPlacement.HAcomodo()` dispatches to selected placement strategy -> e.g., `BLHeuristic()`:
1. Move piece to sheet corner
2. Call `runIntoBottomLeftPlacement()` to slide toward bottom-left
3. Validate position with `posicionValida()`

**Fitness:**
`ControlHeuristics.calcularAptitud()`: sum of (piece_area / bin_area)^2 per bin, averaged
