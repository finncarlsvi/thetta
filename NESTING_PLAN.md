# Thetta Nesting Engine — Implementation Plan

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Rotation | Continuous (any angle) | Best packing quality. No grain direction constraint. |
| Collision detection | jagua-rs (direct dependency) | MIT-licensed, Rust, SOTA speed (millions of queries/sec), no NFP needed |
| Optimization | Guided Local Search (sparrow-style) | Current academic SOTA. Collision-resolution approach. |
| Part-in-part | No | Simpler. Not needed for current use case. |
| Time control | Default time limit + user can stop/continue | Shows progressive improvement in viewport. |
| Multi-sheet | Sequential filling | Fill sheet 1 first, overflow to sheet 2, etc. |

---

## Architecture Overview

```
┌─────────────────────────────────────────────────┐
│                   Thetta App                     │
│                                                  │
│  ┌──────────┐  ┌──────────────┐  ┌───────────┐  │
│  │ STEP I/O │  │ Nesting Core │  │  egui UI  │  │
│  │ module   │──│              │──│  + viewport│  │
│  └──────────┘  └──────────────┘  └───────────┘  │
│                       │                          │
│              ┌────────┴────────┐                 │
│              │    jagua-rs     │                 │
│              │ (collision det.)│                 │
│              └─────────────────┘                 │
└─────────────────────────────────────────────────┘
```

### Crate Structure

```
thetta/
├── Cargo.toml              (workspace)
├── crates/
│   ├── thetta-app/         (GUI, egui, main binary)
│   ├── thetta-nesting/     (optimizer, uses jagua-rs)
│   ├── thetta-step/        (STEP import/export via opencascade-sys)
│   └── thetta-geometry/    (2D polygon types, shared between crates)
```

---

## Phase 1: Foundation (Geometry + STEP Import)

### 1.1 Polygon Representation

Define the core 2D shape types that flow through the entire system.

```rust
/// A 2D polygon with optional holes
struct Polygon {
    outer: Vec<Point2D>,       // CCW winding
    holes: Vec<Vec<Point2D>>,  // CW winding
}

/// A piece to be nested, with metadata
struct Piece {
    id: PieceId,
    polygon: Polygon,
    source_file: PathBuf,
    original_solid: SolidRef,  // reference back to 3D solid for STEP export
    quantity: u32,
}

/// A placed piece on a sheet
struct Placement {
    piece_id: PieceId,
    translation: Vec2D,
    rotation: f64,  // radians, continuous
}
```

### 1.2 STEP Import Pipeline

Using OpenCASCADE via `opencascade-sys`:

1. `STEPControl_Reader::ReadFile()` — load STEP
2. `TransferRoots()` — parse into shapes
3. `TopExp_Explorer(TopAbs_SOLID)` — iterate all solids
4. For each solid, compute bounding box via `Bnd_Box` + `BRepBndLib::Add()`
5. Determine thickness = smallest bounding box dimension (Z typically for flat parts)
6. Group solids by thickness (round to 0.1mm tolerance)
7. For the selected thickness, extract the largest face via `TopExp_Explorer(TopAbs_FACE)` + `BRepGProp::SurfaceProperties()` to compute area
8. Extract the outer wire of that face via `BRepTools::OuterWire()`
9. Discretize wire edges into polyline points via `GCPnts_UniformAbscissa` or `BRepMesh_IncrementalMesh`
10. Output: `Vec<Piece>` with 2D polygons

**Thickness guessing:** After step 5, count solids per thickness bucket. The bucket with the most solids is the default.

### 1.3 Deliverable

- Load a STEP file → extract 2D polygons → display them in an egui viewport
- Thickness selector showing all detected thicknesses with counts
- No nesting yet, just display the raw shapes

---

## Phase 2: jagua-rs Integration

### 2.1 Problem Setup

Convert our `Piece` data into jagua-rs's input format:

```rust
// jagua-rs expects:
// - Items: SimplePolygons (outer boundary + holes)
// - Container: a rectangular strip or bin
// - Allowed rotations: continuous range [0, 2π]

// Build a SPProblem (Strip Packing Problem) or BPProblem (Bin Packing Problem)
// We use strip packing per-sheet: fixed width (sheet_x), minimize length
// If length exceeds sheet_y, overflow to next sheet
```

jagua-rs key types we'll use:
- `SimplePolygon` — our polygons converted to jagua-rs format
- `CDEngine` (Collision Detection Engine) — the core query engine
- `Layout` — represents a sheet with placed items
- `PlaceItem` / `RemoveItem` — operations on the layout

### 2.2 Collision Queries

jagua-rs's CDE answers: "Can piece P be placed at position (x, y, θ) without overlapping anything?"

The CDE uses:
- **Surrogates** (poles + piers) for fast fail rejection
- **Quadtree** spatial index for edge-level checks
- **Hazard proximity grid** for fine-grained queries

We don't compute NFPs at all — jagua-rs handles everything via trigonometric queries.

### 2.3 Deliverable

- Place pieces one at a time using a simple left-bottom-fill heuristic
- Validate no overlaps via jagua-rs
- Display placed pieces in viewport

---

## Phase 3: Guided Local Search Optimizer

This is the core of the nesting engine, inspired by sparrow's approach.

### 3.1 Algorithm Overview

The GLS optimizer works in two phases:

**Phase A — Exploration:**
Find a feasible layout (no overlaps) from an initially overlapping state.

**Phase B — Compression:**
Try to use less strip length (tighter packing) while maintaining feasibility.

### 3.2 Initialization

1. Place all pieces at random positions within the sheet bounds (overlaps allowed)
2. Compute initial collision cost for each piece via jagua-rs

### 3.3 The GLS Loop

```
repeat until time_limit or user_stop:
    1. Select a piece to move (weighted by collision cost + penalty)
    2. Sample N candidate positions for that piece:
       - Random (x, y, θ) within sheet bounds
       - Perturbations of current position
       - Positions near edges of other pieces
    3. Evaluate each candidate via jagua-rs collision queries
    4. Pick the best candidate (least total overlap)
    5. If the piece reduces total overlap, place it there
    6. Update dynamic penalty weights:
       - Pieces that remain in collision get increasing penalty
       - This forces the search to prioritize stuck pieces
    7. If total overlap == 0:
       - Feasible solution found! Record it.
       - Enter Phase B: reduce strip length and try again
```

### 3.4 Candidate Position Sampling

For each piece being moved, generate candidate positions:

1. **Random sampling** — uniform random (x, y) within sheet, random θ ∈ [0, 2π)
2. **Coordinate descent** — from best random sample, refine x, then y, then θ independently
3. **Edge-guided** — sample positions along edges of already-placed pieces (using jagua-rs's hazard proximity data)

The number of samples per move controls quality vs speed. Start with ~1000 samples, refine the best 5 via coordinate descent.

### 3.5 Dynamic Penalty Weights (GLS Core)

Each piece has a penalty weight `w[i]`, initially 1.0.

After each iteration:
- For every piece still in collision: `w[i] += α` (augmentation)
- The piece selection probability is proportional to `collision_cost[i] * w[i]`

This prevents the optimizer from ignoring hard-to-place pieces. Over time, stuck pieces accumulate high weights and get prioritized.

### 3.6 Phase B — Compression

When a feasible solution is found:
1. Record the current strip length as the best known
2. Reduce the strip length by a small delta (e.g., 1% or 5mm)
3. Re-run Phase A with the tighter constraint
4. If feasible again, reduce further. If not, restore best and try smaller delta.
5. Binary search toward the minimum strip length

### 3.7 Multi-Sheet Sequential Filling

```
remaining_pieces = all_pieces
sheet_number = 1

while remaining_pieces is not empty:
    # Try to fit as many pieces as possible on this sheet
    # Start with all remaining pieces
    # Run GLS optimizer with sheet dimensions (x, y)
    # If some pieces can't fit (overlap remains after timeout):
    #   Remove the pieces with highest collision cost
    #   Re-run with fewer pieces
    # Once feasible:
    #   Record placements for this sheet
    #   Remove placed pieces from remaining_pieces
    #   sheet_number += 1
```

### 3.8 Threading Model

- The GLS optimizer runs on a **background thread** (not blocking the UI)
- After each improvement, it sends the new `Vec<Placement>` to the UI thread via a channel
- The UI renders the latest best solution in real-time
- User can click "Stop" to halt optimization and keep the current best

```rust
// Main thread (egui)
let (tx, rx) = std::sync::mpsc::channel::<NestingUpdate>();

// Spawn optimizer
std::thread::spawn(move || {
    optimizer.run(pieces, sheet, tx, cancel_token);
});

// In egui update loop
if let Ok(update) = rx.try_recv() {
    self.current_placements = update.placements;
    self.stats = update.stats; // utilization %, iteration count, etc.
}
```

### 3.9 Deliverable

- Full GLS optimizer running in background
- Real-time viewport updates showing improving solutions
- Stats display: utilization %, pieces placed, time elapsed
- Stop/resume controls

---

## Phase 4: Refinements

### 4.1 Separation Distance

jagua-rs supports item-level "quality zones" — inflate each polygon by `separation / 2` using Minkowski sum (or polygon offsetting via the `geo` crate). This guarantees minimum distance between pieces.

### 4.2 STEP Export

For each sheet:
1. For each placed piece, retrieve the original 3D solid reference
2. Apply the 2D transformation (translation + rotation) as a 3D transform on the XY plane
3. Write all transformed solids to a new STEP file via `STEPControl_Writer`

### 4.3 Optimizer Tuning Parameters

Expose as advanced settings (with sensible defaults):

| Parameter | Default | Description |
|-----------|---------|-------------|
| `samples_per_move` | 1000 | Random positions sampled per piece move |
| `refinement_depth` | 5 | Coordinate descent iterations on best samples |
| `penalty_increment` | 0.1 | GLS weight augmentation per stuck iteration |
| `compression_step` | 5.0mm | Strip length reduction per Phase B iteration |
| `default_time_limit` | 60s | Default optimization time before pausing |

---

## Phase Summary

| Phase | What | Key Risk |
|-------|------|----------|
| 1 | STEP import + polygon extraction + basic viewport | OpenCASCADE FFI complexity |
| 2 | jagua-rs integration + simple placement | Learning jagua-rs API |
| 3 | GLS optimizer + real-time updates + multi-sheet | Algorithm tuning for good results |
| 4 | Separation, STEP export, parameter tuning | Polish |

Each phase produces a working, testable artifact. Phase 1-2 can be developed in parallel (geometry/STEP vs jagua-rs integration) if desired.
