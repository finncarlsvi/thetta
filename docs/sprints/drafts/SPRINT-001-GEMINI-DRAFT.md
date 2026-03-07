# Sprint 001: Thetta MVP — Gemini Draft (written by Claude as alternate perspective)

## Overview

This sprint delivers a **vertical slice** of Thetta: a working nesting application that takes polygons as input and produces optimized sheet layouts. The key insight is to **decouple STEP import from the nesting core** — STEP is a file format concern, not an algorithmic one.

**Sprint 1 scope:** Test polygon input (JSON) + jagua-rs nesting + egui viewport.
**Sprint 2 scope:** STEP import/export via OpenCASCADE FFI.

This de-risks the hardest unknown (OCCT FFI on ARM64) while delivering a functional, testable nesting tool in Sprint 1.

## Use Cases

### UC1: Load Test Polygons
- App ships with sample polygon sets (JSON files)
- User loads a JSON file containing polygon definitions
- Polygons appear in the viewport

### UC2: Configure and Nest
- User sets sheet size (default 2440x1220mm), separation (default 2mm)
- Nesting runs automatically on load, updates in real-time
- Stats: utilization %, iteration count, time, sheets used

### UC3: Interactive Control
- Stop/resume optimization
- Delete individual pieces
- Reset to reload all pieces

### UC4: Viewport Navigation
- Zoom via scroll wheel
- Pan via middle-click drag or Ctrl+drag
- Click to select pieces
- Sheet tabs for multi-sheet navigation

## Architecture

### Why Defer STEP to Sprint 2

1. **OCCT FFI is the highest-risk item** — compiling opencascade-sys on ARM64 macOS is untested territory. Isolating this de-risks Sprint 1.
2. **JSON polygon input is trivial** and lets us test the nesting engine thoroughly.
3. **The nesting algorithm is the core value** — get it right first.
4. **STEP import is a preprocessing step** that doesn't affect nesting architecture.

### Crate Structure

```
thetta/
├── crates/
│   ├── thetta-geometry/     # 2D types, polygon ops, serialization
│   ├── thetta-nesting/      # GLS optimizer + jagua-rs integration
│   └── thetta-app/          # egui GUI, viewport, file I/O
```

Note: `thetta-step` exists as a stub but is not implemented in Sprint 1.

### jagua-rs Integration Strategy

jagua-rs has a `probs/bpp/` module (Bin Packing Problem) that is directly applicable:

```rust
// jagua-rs provides:
// - BPInstance: problem definition (items + bins)
// - BPProblem: mutable problem state with layouts
// - Layout: a bin with placed items and CDE
// - CDEngine: collision detection per layout

// Our integration:
// 1. Convert Vec<Piece> → jagua-rs Items (SimplePolygon + allowed rotations)
// 2. Create a rectangular Container as our sheet
// 3. Build BPInstance
// 4. Use Layout + CDEngine for collision queries during GLS
```

Key jagua-rs types from `research/repos/jagua-rs/jagua-rs/src/`:
- `entities/item.rs` — Item definition with shape + allowed transformations
- `entities/layout.rs` — Layout with placed items + CDE
- `entities/container.rs` — Container shape (our rectangular sheet)
- `collision_detection/cd_engine.rs` — The collision detection engine
- `probs/bpp/` — Bin packing problem types
- `geometry/primitives/simple_polygon.rs` — Polygon representation

### Threading Model

```
┌─────────────────────┐
│   Main Thread        │
│   (egui render loop) │
│                      │       mpsc::channel<NestingUpdate>
│   - Renders viewport │ ←──────────────────────────────────
│   - Handles input    │                                    │
│   - Updates state    │       Arc<AtomicBool> cancel       │
│                      │ ──────────────────────────────────→ │
└─────────────────────┘                                    │
                                              ┌────────────┴──┐
                                              │ Worker Thread  │
                                              │ (GLS Optimizer)│
                                              │                │
                                              │ Owns:          │
                                              │ - jagua-rs CDE │
                                              │ - Piece data   │
                                              │ - GLS state    │
                                              └────────────────┘
```

The optimizer owns all jagua-rs state (no sharing across threads). It clones the result and sends it via channel when an improvement is found.

## Implementation

### Phase 1: Geometry Foundation (Days 1-2)

#### Task 1.1: Polygon operations
File: `crates/thetta-geometry/src/polygon_ops.rs`
- `area(polygon) -> f64` (shoelace formula)
- `bounding_box(polygon) -> Rect`
- `translate(polygon, dx, dy) -> Polygon`
- `rotate(polygon, angle, center) -> Polygon`
- `offset(polygon, distance) -> Polygon` (for separation — use Clipper or manual offset)

#### Task 1.2: Serialization
- Add `serde::{Serialize, Deserialize}` to all geometry types
- Create sample JSON files with test polygons:
  - `test_data/rectangles.json` — 20 mixed rectangles
  - `test_data/irregular.json` — L-shapes, T-shapes, triangles
  - `test_data/mixed.json` — realistic mix of shapes

#### Task 1.3: jagua-rs type conversion
File: `crates/thetta-nesting/src/conversion.rs`
- `piece_to_jagua_item(piece: &Piece) -> jagua_rs::Item`
- `sheet_to_jagua_container(config: &SheetConfig) -> jagua_rs::Container`
- `jagua_placement_to_ours(placed: &PlacedItem) -> Placement`

### Phase 2: jagua-rs Integration + LBF Baseline (Days 3-5)

#### Task 2.1: NestingEngine struct
File: `crates/thetta-nesting/src/engine.rs`
```rust
pub struct NestingEngine {
    instance: BPInstance,
    problem: BPProblem,
}

impl NestingEngine {
    pub fn new(pieces: Vec<Piece>, sheet: SheetConfig) -> Self;
    pub fn place_item(&mut self, item_id: usize, transformation: DTransformation) -> bool;
    pub fn remove_item(&mut self, item_id: usize);
    pub fn check_collision(&self, item_id: usize, transformation: DTransformation) -> f64;
    pub fn current_placements(&self) -> Vec<Placement>;
    pub fn utilization(&self) -> f64;
}
```

#### Task 2.2: Left-Bottom-Fill heuristic
File: `crates/thetta-nesting/src/lbf.rs`

Reference: `research/repos/jagua-rs/lbf/src/opt/lbf_bpp.rs`

This is the baseline placement strategy:
1. Sort pieces by area (descending)
2. For each piece, sample positions in a grid pattern
3. Pick the bottom-left-most valid position (no collision)
4. Place it

This gives us a working nesting result before implementing GLS.

#### Task 2.3: Unit tests
- Test: place a single rectangle → verify placement
- Test: place two overlapping pieces → verify collision detected
- Test: LBF on 10 rectangles → verify all placed, no overlaps
- Test: LBF on pieces that don't fit one sheet → verify overflow

### Phase 3: GLS Optimizer (Days 6-10)

#### Task 3.1: GLS exploration phase
File: `crates/thetta-nesting/src/gls.rs`

Reference: `research/repos/sparrow/src/optimizer/explore.rs`

```rust
pub struct GLSOptimizer {
    engine: NestingEngine,
    penalty_weights: Vec<f64>,
    config: OptimizerConfig,
    best_result: Option<NestingResult>,
    rng: StdRng,
}

impl GLSOptimizer {
    pub fn run(
        &mut self,
        tx: mpsc::Sender<NestingUpdate>,
        cancel: Arc<AtomicBool>,
    );
}
```

Core loop:
1. Initialize: place all pieces at random positions (collisions allowed via jagua-rs's `place_item` with collision tolerance)
2. Compute collision cost per piece
3. Select piece to move: weighted random by `cost[i] * weight[i]`
4. Sample N candidate positions (uniform random x, y, θ)
5. Evaluate each via `check_collision()`
6. Pick best, apply if it reduces total cost
7. Update penalty weights for still-colliding pieces
8. If total_cost == 0 → feasible! Send update, enter compression.

#### Task 3.2: Candidate sampling and refinement
File: `crates/thetta-nesting/src/sampler.rs`

Reference: `research/repos/sparrow/src/sample/`

- `UniformSampler`: random (x, y, θ) within sheet bounds
- `CoordinateDescentRefiner`: from a good sample, optimize x→y→θ independently
- Sample 1000 random, take top 5, refine each, pick overall best

#### Task 3.3: Compression phase
File: `crates/thetta-nesting/src/compression.rs`

Reference: `research/repos/sparrow/src/optimizer/compress.rs`

When feasible solution found:
1. Record current "used width" (rightmost piece edge)
2. Set container width = used_width - step
3. Re-run exploration
4. If feasible: reduce further. If not: restore best, reduce step.
5. Binary search converges on tightest feasible packing.

#### Task 3.4: Multi-sheet sequential filling
File: `crates/thetta-nesting/src/multi_sheet.rs`
- Run GLS on all pieces for sheet 1
- After time limit or convergence, identify placed vs unplaced
- Create sheet 2 with unplaced pieces, repeat
- Send combined NestingResult (all sheets) via channel

### Phase 4: egui Application (Days 8-12, overlaps with Phase 3)

#### Task 4.1: Application state
File: `crates/thetta-app/src/state.rs`
```rust
pub struct AppState {
    pub pieces: Vec<Piece>,
    pub sheet_config: SheetConfig,
    pub current_result: Option<NestingResult>,
    pub selected_piece: Option<PieceId>,
    pub active_sheet: usize,
    pub optimizer_running: bool,
    pub stats: NestingStats,
    // Channel endpoints
    pub update_rx: Option<mpsc::Receiver<NestingUpdate>>,
    pub cancel_token: Option<Arc<AtomicBool>>,
}
```

#### Task 4.2: Viewport widget
File: `crates/thetta-app/src/viewport.rs`

Key implementation:
```rust
fn draw_viewport(ui: &mut egui::Ui, state: &AppState) {
    let (response, painter) = ui.allocate_painter(
        ui.available_size(),
        egui::Sense::click_and_drag(),
    );

    // Zoom/pan transform
    let transform = RectTransform::from_to(
        world_rect,   // sheet coordinates in mm
        response.rect, // screen pixels
    );

    // Draw sheet boundary
    painter.rect_stroke(transform.transform_rect(sheet_rect), ...);

    // Draw each placed piece
    for placement in &sheet.placements {
        let polygon = get_transformed_polygon(placement);
        let points: Vec<Pos2> = polygon.outer.iter()
            .map(|p| transform.transform_pos(pos2(p.x as f32, p.y as f32)))
            .collect();
        painter.add(Shape::convex_polygon(points, fill_color, stroke));
    }
}
```

Zoom: handle `ui.input().scroll_delta.y` to adjust world_rect scale.
Pan: handle drag on middle mouse button to offset world_rect.

#### Task 4.3: Side panel + controls
File: `crates/thetta-app/src/panels.rs`
- Load file button (JSON for Sprint 1)
- Sheet size inputs (width, height in mm)
- Separation input (mm)
- Start/Stop nesting button
- Piece list with delete buttons
- Stats display

#### Task 4.4: Wire everything together
File: `crates/thetta-app/src/main.rs`
- Load JSON → create pieces → start optimizer → update viewport
- Handle user interactions (delete, stop, configure)
- Poll channel each frame for optimizer updates

## Files Summary

### New Files (Sprint 1)

| File | LOC Est. | Purpose |
|------|----------|---------|
| `thetta-geometry/src/polygon_ops.rs` | 150 | Geometry operations |
| `thetta-geometry/src/test_shapes.rs` | 80 | Test polygon generators |
| `thetta-nesting/src/conversion.rs` | 100 | Type conversion to/from jagua-rs |
| `thetta-nesting/src/engine.rs` | 200 | NestingEngine wrapper |
| `thetta-nesting/src/lbf.rs` | 150 | Left-bottom-fill baseline |
| `thetta-nesting/src/gls.rs` | 400 | GLS optimizer core |
| `thetta-nesting/src/sampler.rs` | 150 | Position sampling + refinement |
| `thetta-nesting/src/compression.rs` | 100 | Strip compression phase |
| `thetta-nesting/src/multi_sheet.rs` | 80 | Sequential sheet filling |
| `thetta-app/src/state.rs` | 100 | Application state |
| `thetta-app/src/viewport.rs` | 250 | 2D viewport with zoom/pan |
| `thetta-app/src/panels.rs` | 200 | UI panels and controls |
| `test_data/*.json` | — | Sample polygon sets |
| **Total** | **~1960** | |

### Modified Files

| File | Changes |
|------|---------|
| `Cargo.toml` | Workspace deps (serde, jagua-rs) |
| `thetta-nesting/Cargo.toml` | Add jagua-rs, rand, serde |
| `thetta-app/Cargo.toml` | Add rfd, serde_json |
| `thetta-geometry/src/types.rs` | Serde derives, Display impls |
| `thetta-geometry/src/lib.rs` | New module declarations |
| `thetta-nesting/src/lib.rs` | New module declarations |
| `thetta-app/src/main.rs` | Complete rewrite |

## Definition of Done

### P0 — Must ship
- [ ] `cargo build --release` succeeds on macOS ARM64
- [ ] App launches and shows empty viewport
- [ ] Can load JSON polygon file → pieces appear in viewport
- [ ] LBF baseline places all pieces without overlap
- [ ] GLS optimizer runs in background thread
- [ ] Viewport updates as optimizer improves
- [ ] Zoom (scroll) and pan (drag) work in viewport
- [ ] Sheet boundary visible as rectangle outline
- [ ] Multi-sheet: overflow pieces go to sheet 2+
- [ ] Stop/resume button works
- [ ] Delete piece works
- [ ] No panics or crashes on normal usage

### P1 — Should ship
- [ ] Separation distance enforced between pieces
- [ ] Compression phase improves packing after feasibility
- [ ] Stats display (utilization %, time, pieces, sheets)
- [ ] Sheet tab navigation
- [ ] Piece selection with visual highlight

### P2 — Stretch
- [ ] Drag-and-drop JSON loading
- [ ] Configurable optimizer parameters in UI
- [ ] SVG export of nesting layout (for debugging)

## Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| jagua-rs API incompatibility with our use case | High | Study lbf reference impl thoroughly; we have full source to fork if needed |
| GLS convergence speed is too slow for interactive use | Medium | Start with LBF as fast baseline; GLS runs in background for improvement |
| Polygon offset for separation is complex | Medium | Use simple bounding-box inflation first; proper Minkowski offset later |
| egui immediate mode makes complex state management hard | Low | Use a clean AppState struct; egui is well-suited for this scale of app |

## Security

- No network access
- File I/O limited to user-selected files via native dialog
- No eval/exec of user-provided content
- JSON parsing via serde (safe)

## Dependencies

| Crate | Purpose |
|-------|---------|
| `jagua-rs` | Collision detection engine (git dep) |
| `eframe`/`egui` | GUI framework |
| `rfd` | Native file dialogs |
| `serde`/`serde_json` | JSON serialization |
| `rand` | RNG for optimizer |

No system dependencies required for Sprint 1 (OCCT deferred to Sprint 2).

## Open Questions

1. **jagua-rs BPP vs SPP**: Should we use Bin Packing Problem mode (fixed bins) or Strip Packing Problem mode (minimize strip length) per sheet? BPP matches our problem better but SPP might work with a height cutoff.
2. **Polygon offset strategy**: For separation, should we inflate polygons before passing to jagua-rs, or use jagua-rs's quality zones? Need to check jagua-rs API.
3. **Continuous rotation representation**: jagua-rs uses `DTransformation(x, y, rotation)`. Confirm we can pass arbitrary f64 angles, not just discrete steps.
4. **Performance target**: What's "good enough" for interactive use? 1 update/sec? 10 updates/sec?
