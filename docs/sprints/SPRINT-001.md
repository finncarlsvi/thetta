# Sprint 001: Thetta — Nesting Engine + Interactive GUI

## Overview

Build the first working version of Thetta: a macOS desktop application that loads 2D polygon definitions (JSON), nests them into rectangular sheets using a GLS optimizer backed by jagua-rs, and displays the result in an interactive egui viewport with real-time progressive optimization.

**Scope:** JSON polygon input → jagua-rs nesting (LBF baseline + GLS optimizer) → egui GUI with polished panels.
**Deferred to Sprint 2:** STEP file import/export (OpenCASCADE FFI), multi-sheet support.

## Use Cases

### UC1: Load Polygons
- User clicks "Load" → native file dialog → select JSON polygon file
- Polygons appear in the viewport immediately
- Can load additional files (pieces accumulate)
- Pieces colored by source file

### UC2: Configure Sheet
- Side panel inputs for sheet width (default 2440mm), height (default 1220mm), separation (default 2mm)
- Changes trigger automatic re-nesting

### UC3: Run Nesting
- Nesting starts automatically on polygon load
- LBF runs first for instant baseline result
- GLS optimizer continues in background, progressively improving
- Viewport updates in real-time as better solutions are found
- Stats panel: utilization %, pieces placed/total, elapsed time

### UC4: Control Optimization
- Start/Stop button pauses/resumes the GLS optimizer
- Default 60-second time limit, then auto-pauses
- User can resume for continued optimization

### UC5: Manage Pieces
- Click a piece in viewport to select it (highlighted outline)
- Delete key or button removes selected piece from nest
- "Reload" button per file resets its pieces
- Deletion triggers re-nesting

### UC6: Navigate Viewport
- Scroll wheel to zoom (centered on cursor)
- Middle-click drag (or Ctrl+drag) to pan
- Sheet boundary drawn as rectangle outline

## Architecture

### Crate Structure

```
thetta/
├── Cargo.toml                    (workspace)
├── crates/
│   ├── thetta-geometry/          (2D types, polygon ops, serde)
│   ├── thetta-nesting/           (LBF + GLS optimizer, jagua-rs integration)
│   ├── thetta-app/               (egui GUI, viewport, panels, state)
│   └── thetta-step/              (STUB — Sprint 2)
├── test_data/                    (JSON polygon files)
```

### Dependency Graph

```
thetta-app
├── thetta-nesting
│   ├── thetta-geometry
│   └── jagua-rs (git dependency)
├── thetta-geometry
├── eframe / egui
├── rfd (file dialogs)
└── serde_json
```

### Data Flow

```
JSON File → [serde] → Vec<Piece>
                          │
                          ▼
                   ┌──────────────┐     ┌──────────────┐
                   │ NestingEngine │────▶│   jagua-rs   │
                   │ (LBF + GLS)  │◀────│    CDE       │
                   └──────┬───────┘     └──────────────┘
                          │
                    mpsc::channel
                          │
                          ▼
                   ┌──────────────┐
                   │   egui App   │
                   │  (viewport + │
                   │   panels)    │
                   └──────────────┘
```

### Threading Model

```
Main Thread (egui)                    Background Thread
┌─────────────────┐                  ┌───────────────────┐
│ - Renders UI     │ ◀── channel ── │ - Owns jagua-rs    │
│ - Handles input  │                 │   CDE + Layout     │
│ - Polls updates  │ ── cancel ──▶  │ - Runs LBF then    │
│   each frame     │   AtomicBool   │   GLS optimizer    │
└─────────────────┘                  └───────────────────┘
```

- Optimizer thread **owns** all jagua-rs state (no sharing)
- Sends `NestingUpdate` messages via `mpsc::channel` on each improvement
- UI calls `rx.try_recv()` each frame — non-blocking
- `Arc<AtomicBool>` cancel token for stop/resume

## Implementation

### Phase 1: Geometry Foundation

#### Task 1.1: Polygon operations
**File:** `crates/thetta-geometry/src/polygon_ops.rs`
- `fn area(poly: &Polygon) -> f64` — shoelace formula
- `fn bounding_box(poly: &Polygon) -> BoundingBox`
- `fn translate(poly: &Polygon, dx: f64, dy: f64) -> Polygon`
- `fn rotate(poly: &Polygon, angle: f64, center: Point2D) -> Polygon`
- `fn offset(poly: &Polygon, distance: f64) -> Polygon` — for separation

#### Task 1.2: Serialization and test data
- Add `serde::{Serialize, Deserialize}` to all geometry types
- **File:** `crates/thetta-geometry/src/test_shapes.rs` — generators for rectangles, L-shapes, T-shapes, triangles, circles
- **Files:** `test_data/rectangles.json`, `test_data/irregular.json`, `test_data/mixed.json`

#### Task 1.3: Unit tests
- Area calculation for known shapes
- Bounding box correctness
- Translate/rotate preserve area
- Serialization round-trip

### Phase 2: jagua-rs Integration + LBF Baseline

#### Task 2.1: Type conversion layer
**File:** `crates/thetta-nesting/src/conversion.rs`
- `fn piece_to_jagua_item(piece: &Piece, config: &SheetConfig) -> jagua_rs Item`
- `fn sheet_to_jagua_container(config: &SheetConfig) -> jagua_rs Container`
- `fn extract_placements(layout: &Layout) -> Vec<Placement>`

Must handle:
- Our `Polygon` → jagua-rs `SimplePolygon` (coordinate conversion, winding order)
- Continuous rotation range `[0, 2π)` for allowed transformations
- Polygon inflation by `separation/2` for spacing enforcement

#### Task 2.2: NestingEngine wrapper
**File:** `crates/thetta-nesting/src/engine.rs`
```rust
pub struct NestingEngine {
    // Wraps jagua-rs BPProblem or SPProblem
    // Provides high-level API for our optimizer
}

impl NestingEngine {
    pub fn new(pieces: &[Piece], sheet: &SheetConfig) -> Self;
    pub fn place_item(&mut self, idx: usize, x: f64, y: f64, rotation: f64) -> bool;
    pub fn remove_item(&mut self, idx: usize);
    pub fn collision_cost(&self, idx: usize, x: f64, y: f64, rotation: f64) -> f64;
    pub fn current_result(&self) -> NestingResult;
    pub fn utilization(&self) -> f64;
}
```

#### Task 2.3: Left-Bottom-Fill heuristic
**File:** `crates/thetta-nesting/src/lbf.rs`

Reference: `research/repos/jagua-rs/lbf/src/opt/lbf_bpp.rs`

1. Sort pieces by area (descending)
2. For each piece:
   - Sample N positions in a grid across the sheet
   - For each position, try M rotation angles
   - Use `collision_cost()` to find valid placements
   - Pick the bottom-left-most valid position
3. Return result

This gives an instant baseline (sub-second for typical inputs).

#### Task 2.4: Integration tests
- Place a single rectangle → verify valid placement
- Place overlapping pieces → verify collision detected
- LBF on 10 rectangles → all placed, no overlaps, within bounds
- LBF on too many pieces → some reported as unplaced

### Phase 3: GLS Optimizer

#### Task 3.1: GLS core loop
**File:** `crates/thetta-nesting/src/gls.rs`

Reference: `research/repos/sparrow/src/optimizer/explore.rs`

```rust
pub struct GLSOptimizer {
    engine: NestingEngine,
    penalty_weights: Vec<f64>,
    config: OptimizerConfig,
    best_feasible: Option<NestingResult>,
    rng: StdRng,
}
```

Algorithm:
1. **Initialize:** Place all pieces at random positions (overlaps allowed)
2. **Compute** collision cost per piece via jagua-rs
3. **Loop** until cancelled or time limit:
   a. **Select** piece to move — weighted by `cost[i] * weight[i]`
   b. **Sample** 1000 random candidate positions `(x, y, θ)`
   c. **Refine** top 5 candidates via coordinate descent
   d. **Evaluate** each via `collision_cost()`
   e. **Apply** best if it reduces total overlap
   f. **Update** penalty weights: `weight[i] += 0.1` for still-colliding pieces
   g. **If** total overlap == 0 → feasible! Send update via channel.

#### Task 3.2: Position sampling
**File:** `crates/thetta-nesting/src/sampler.rs`

Reference: `research/repos/sparrow/src/sample/`

- `UniformSampler` — random `(x, y)` within sheet, `θ` in `[0, 2π)`
- `CoordinateDescentRefiner` — optimize x, y, θ independently from a seed point
- Top-K selection from random samples → refine → pick best

#### Task 3.3: Compression phase
**File:** `crates/thetta-nesting/src/compression.rs`

Reference: `research/repos/sparrow/src/optimizer/compress.rs`

When feasible solution found:
1. Compute used width (rightmost piece edge)
2. Tighten sheet width by `compression_step` (default 5mm)
3. Re-run GLS exploration with tighter bounds
4. Binary search for minimum feasible width
5. Send improved result each time

#### Task 3.4: Background thread integration
**File:** `crates/thetta-nesting/src/runner.rs`

```rust
pub fn run_nesting(
    pieces: Vec<Piece>,
    sheet: SheetConfig,
    config: OptimizerConfig,
    tx: mpsc::Sender<NestingUpdate>,
    cancel: Arc<AtomicBool>,
) {
    // 1. Run LBF → send initial result
    // 2. Run GLS → send improvements
    // 3. Run compression → send tighter results
    // 4. Send Finished when done/cancelled
}
```

### Phase 4: egui Application

#### Task 4.1: Application state
**File:** `crates/thetta-app/src/state.rs`

```rust
pub struct AppState {
    pub loaded_files: Vec<LoadedFile>,
    pub pieces: Vec<Piece>,
    pub sheet_config: SheetConfig,
    pub optimizer_config: OptimizerConfig,
    pub current_result: Option<NestingResult>,
    pub selected_piece: Option<PieceId>,
    pub optimizer_running: bool,
    pub stats: NestingStats,
    pub update_rx: Option<Receiver<NestingUpdate>>,
    pub cancel_token: Option<Arc<AtomicBool>>,
}

pub struct LoadedFile {
    pub path: PathBuf,
    pub piece_count: usize,
}

pub struct NestingStats {
    pub utilization_pct: f64,
    pub pieces_placed: usize,
    pub pieces_total: usize,
    pub elapsed: Duration,
    pub iterations: usize,
}
```

#### Task 4.2: Viewport widget
**File:** `crates/thetta-app/src/viewport.rs`

- Custom egui widget using `ui.allocate_painter()`
- `RectTransform` maps world coordinates (mm) → screen pixels
- **Zoom:** scroll wheel adjusts transform scale, centered on cursor
- **Pan:** middle-click drag (or Ctrl+left drag) offsets transform
- **Render:** sheet boundary, placed pieces (filled + outlined), selected piece highlight
- **Hit testing:** click → find piece under cursor → select

#### Task 4.3: Side panel
**File:** `crates/thetta-app/src/panels.rs`

Left side panel:
- **Files section:** list of loaded files with piece counts, "Reload" button each, "Load" button
- **Sheet section:** width/height/separation inputs (drag values or type)
- **Pieces section:** scrollable list, click to select, delete button
- **Nesting section:** Start/Stop button, time limit input
- **Stats section:** utilization %, pieces placed/total, elapsed time

#### Task 4.4: Main app wiring
**File:** `crates/thetta-app/src/main.rs`

- File loading: `rfd::FileDialog` → read JSON → deserialize → add to pieces
- On pieces change: spawn optimizer thread, pass pieces + config
- Each frame: poll `rx.try_recv()` for updates
- Handle keyboard: Delete key removes selected piece
- Handle stop/resume: toggle cancel token

#### Task 4.5: Piece colors
- Assign a color per source file (cycle through a palette)
- Selected piece: bright outline + slight transparency change
- Unplaced pieces: shown in a separate "unplaced" area or grayed out

## Files Summary

### New Files

| File | Purpose | Est. LOC |
|------|---------|----------|
| `thetta-geometry/src/polygon_ops.rs` | Geometry operations | 150 |
| `thetta-geometry/src/test_shapes.rs` | Test polygon generators | 80 |
| `thetta-nesting/src/conversion.rs` | Polygon ↔ jagua-rs conversion | 120 |
| `thetta-nesting/src/engine.rs` | NestingEngine wrapper | 200 |
| `thetta-nesting/src/lbf.rs` | Left-Bottom-Fill heuristic | 150 |
| `thetta-nesting/src/gls.rs` | GLS optimizer core | 400 |
| `thetta-nesting/src/sampler.rs` | Position sampling + refinement | 150 |
| `thetta-nesting/src/compression.rs` | Strip compression phase | 100 |
| `thetta-nesting/src/runner.rs` | Background thread orchestration | 80 |
| `thetta-app/src/state.rs` | Application state | 100 |
| `thetta-app/src/viewport.rs` | 2D viewport with zoom/pan | 300 |
| `thetta-app/src/panels.rs` | Side panel, controls, stats | 250 |
| `test_data/rectangles.json` | Test rectangles | — |
| `test_data/irregular.json` | Test L/T/triangle shapes | — |
| `test_data/mixed.json` | Realistic shape mix | — |
| **Total** | | **~2080** |

### Modified Files

| File | Changes |
|------|---------|
| `Cargo.toml` | Workspace deps (serde, rand) |
| `thetta-geometry/Cargo.toml` | Add serde |
| `thetta-geometry/src/types.rs` | Serde derives, BoundingBox type |
| `thetta-geometry/src/lib.rs` | New module declarations |
| `thetta-nesting/Cargo.toml` | Add jagua-rs, rand, serde |
| `thetta-nesting/src/lib.rs` | New module declarations |
| `thetta-nesting/src/optimizer.rs` | Replace stub with real module structure |
| `thetta-app/Cargo.toml` | Add rfd, serde_json |
| `thetta-app/src/main.rs` | Complete rewrite |

## Definition of Done

### P0 — Must Ship
- [ ] `cargo build --release` succeeds on macOS ARM64
- [ ] App launches with empty viewport and side panel
- [ ] Load JSON polygon file via file dialog → pieces appear in viewport
- [ ] LBF places all pieces without overlap (verified by jagua-rs)
- [ ] GLS optimizer runs in background, sends progressive updates
- [ ] Viewport renders placed pieces with correct positions and rotations
- [ ] Zoom (scroll wheel) and pan (middle-click drag) work
- [ ] Separation distance enforced between all pieces
- [ ] Start/Stop button controls optimizer
- [ ] Click to select piece, Delete to remove it
- [ ] Stats display: utilization %, pieces placed, elapsed time
- [ ] No panics or crashes on normal usage

### P1 — Should Ship
- [ ] Compression phase tightens packing after feasibility
- [ ] Multiple file loading (pieces accumulate)
- [ ] Per-file "Reload" button
- [ ] Piece colors by source file
- [ ] Selected piece visual highlight
- [ ] Scrollable piece list in side panel

### P2 — Stretch
- [ ] Drag-and-drop JSON loading
- [ ] SVG export of nesting layout
- [ ] Configurable optimizer parameters in UI

### Cut if Behind Schedule
- Multi-sheet support (deferred to Sprint 2 along with STEP)

## Risks

| Risk | Impact | Likelihood | Mitigation |
|------|--------|-----------|------------|
| jagua-rs API doesn't match our needs (BPP vs SPP) | High | Medium | Study `lbf/src/opt/lbf_bpp.rs` reference; we have full source to fork |
| GLS produces poor results without tuning | Medium | Medium | LBF baseline ensures usable output; tune GLS against test data |
| Polygon offset for separation is geometrically complex | Medium | Medium | Start with bounding-box inflation; proper Minkowski later |
| egui zoom/pan jank | Low | Low | Well-documented pattern; allocate time for polish |
| jagua-rs compile time bloats incremental builds | Low | Medium | Feature-gate optional jagua-rs modules |

## Security

- No network access. Fully local application.
- File I/O via native OS dialog only.
- JSON parsed via serde (safe, no arbitrary code execution).
- No user-provided code, scripts, or macros.

## Dependencies

### Rust Crates

| Crate | Purpose |
|-------|---------|
| `jagua-rs` | Collision detection engine (git dependency) |
| `eframe` / `egui` | GUI framework |
| `rfd` | Native macOS file dialogs |
| `serde` / `serde_json` | JSON serialization |
| `rand` | RNG for optimizer sampling |

### System Dependencies

None for Sprint 1 (OCCT deferred to Sprint 2).

## Open Questions

1. **jagua-rs BPP vs SPP:** Use Bin Packing (fixed bins) or Strip Packing (minimize length) mode? Need to evaluate both against our use case. BPP is semantically correct but SPP with height cutoff may be simpler to integrate.
2. **Separation enforcement:** Inflate polygons before passing to jagua-rs, or use jagua-rs quality zones? Needs API investigation.
3. **Continuous rotation in jagua-rs:** Confirm `DTransformation` accepts arbitrary f64 angles. Check if jagua-rs pre-discretizes or truly supports continuous.
4. **Test STEP files for Sprint 2:** Start collecting representative sheet-metal STEP files during Sprint 1 development.
