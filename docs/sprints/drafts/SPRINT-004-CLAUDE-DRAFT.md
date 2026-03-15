# Sprint 004: Multi-Sheet Bin Packing + Multiple STEP File Loading

## Overview

Implement multi-sheet bin packing (BPP) so pieces that exceed one sheet overflow onto additional fixed-size sheets, and support loading multiple STEP files with pieces accumulating across loads. This sprint adds a dual-mode nesting engine (SPP for benchmarks, BPP for sheet-based workflows), adapts LBF and GLS for multi-bin placement, renders all sheets in the viewport with side-by-side scrolling, and aggregates statistics across sheets.

**Scope:** Dual-mode `NestingEngine` (SPP/BPP enum), BPP instance construction via jagua-rs `BPInstance`/`BPProblem`, multi-bin LBF placement with overflow, per-sheet GLS optimization, multi-sheet viewport rendering with horizontal scrolling, per-sheet + aggregate stats, multiple STEP file accumulation, sheet navigation UI.
**Prerequisites:** Sprint 003 complete. The nesting engine wraps jagua-rs `SPProblem` and accepts `Vec<Piece>` with optional `Vec<RotationRange>` orientations. STEP import/export pipeline is operational. `NestingResult` already contains `Vec<Sheet>`. The app supports background optimizer with cancel token and STEP file loading.
**Deferred:** Cross-sheet GLS moves (moving pieces between sheets to balance utilization), sheet-count compression (removing last sheet and redistributing), per-sheet export selection UI, drag-and-drop piece reordering between sheets.

## Use Cases

### UC1: Automatic Multi-Sheet Overflow
- User loads a STEP file (or JSON polygon file) with many pieces
- App nests pieces into the first 2440x1220mm sheet
- When pieces no longer fit on the current sheet, a new sheet is automatically opened
- Process repeats until all pieces are placed or maximum sheet count is reached
- Viewport shows all sheets side-by-side; stats show per-sheet and aggregate utilization

### UC2: Load Multiple STEP Files with Piece Accumulation
- User clicks "Load STEP" and loads `parts_a.step` (10 solids)
- App imports, nests across N sheets
- User clicks "Load STEP" again and loads `parts_b.step` (8 solids)
- The 8 new pieces are added to the existing 10 (not replacing them)
- Nesting restarts with all 18 pieces, potentially using more sheets
- Each file's pieces retain their source color in the viewport
- Piece list shows all pieces with file attribution

### UC3: Sheet Navigation and Visualization
- After nesting produces multiple sheets, viewport renders them side-by-side horizontally
- Each sheet is outlined with its index label ("Sheet 1", "Sheet 2", etc.)
- Horizontal scrolling / pan navigates between sheets
- The side panel shows: "Sheets: 3" with per-sheet utilization breakdown
- Clicking a piece on any sheet selects it (existing selection behavior extended)

### UC4: Benchmark Mode Backward Compatibility
- User loads a sparrow JSON benchmark file (e.g., `jakobs1.json`)
- Engine uses SPP mode (single infinite strip), identical to Sprint 002 behavior
- No multi-sheet overhead: single layout, strip compression, GLS on one layout
- All existing benchmark results remain reproducible

### UC5: Multi-Sheet STEP Export
- User nests pieces across 3 sheets, clicks "Export STEP"
- App writes `output_sheet_001.step`, `output_sheet_002.step`, `output_sheet_003.step`
- Each file contains only the solids placed on that sheet
- Status shows "Exported 3 sheets"
- This already works if `NestingResult.sheets` is correctly populated (Sprint 003 export iterates sheets)

### UC6: Sheet Configuration Applies to All Sheets
- User changes sheet width/height in the Sheet panel
- All sheets use the new dimensions (single template replicated)
- Nesting restarts with the new sheet size; piece overflow recomputed
- Separation value applies identically within each sheet

## Architecture

### Crate Structure (Post-Sprint 004)

```
thetta/
+-- Cargo.toml                    (workspace: resolver = "2")
+-- crates/
|   +-- thetta-geometry/          (unchanged: Piece, Polygon, Point2D, Placement, Sheet, NestingResult)
|   +-- thetta-nesting/           (NestingEngine gains BPP mode, LBF/GLS adapted for multi-bin)
|   +-- thetta-app/               (viewport renders all sheets, stats aggregated, multi-file loading)
|   +-- thetta-step/              (unchanged from Sprint 003)
|   +-- jagua-rs/                 (unchanged: BPP support already exists, enable "bpp" feature)
+-- research/repos/sparrow/       (benchmark data)
+-- test_data/
```

### Dependency Graph (Sprint 004 Changes)

```
thetta-app
+-- thetta-nesting
|   +-- thetta-geometry
|   +-- jagua-rs (features = ["spp", "bpp"])    <-- ADD "bpp" feature
+-- thetta-step
|   +-- thetta-geometry
|   +-- opencascade-sys (feature-gated)
+-- thetta-geometry
+-- jagua-rs (features = ["spp", "bpp"])         <-- ADD "bpp" feature
+-- eframe 0.31 / egui 0.31
+-- serde_json 1
+-- rfd
```

### Data Flow

```
Vec<Piece> + SheetConfig
        |
        v
  +---------------------+
  | NestingMode::Bpp     |    pieces_to_bp_instance()
  | -> BPInstance with    |    Single Bin template (W x H), stock = 100
  |    one Bin type       |
  +----------+-----------+
             |
             v
  +---------------------+
  | BppNestingEngine     |    Wraps BPProblem
  | .place_item(idx,     |    Routes to correct Layout via LayKey
  |   sheet, x, y, r)    |
  +----------+-----------+
             |
             v
  +---------------------+
  | run_lbf_bpp()        |    For each piece (area-descending):
  |  Try all open sheets |      1. Try placement on each existing sheet
  |  Open new sheet if   |      2. If no fit, open new bin via BPLayoutType::Closed
  |  piece won't fit     |      3. LBF grid search within chosen sheet
  +----------+-----------+
             |
             v
  +---------------------+
  | run_gls_bpp()        |    For each sheet independently:
  |  Per-sheet GLS       |      Run GLS optimizer on that layout
  |  (same logic as SPP  |      Minimize overlap within the sheet
  |   but scoped to one  |
  |   layout)            |
  +----------+-----------+
             |
             v
  NestingResult {
      sheets: Vec<Sheet {
          placements: Vec<Placement>,
          utilization: f64,
          strip_length: f64,  // = sheet width for BPP (fixed)
      }>,
      unplaced: Vec<PieceId>,
  }
             |
             v
  +---------------------+
  | Viewport             |    Render sheets side-by-side
  | sheet_gap = 50mm     |    Horizontal offset per sheet
  | Labels: "Sheet 1"    |    Click-to-select works across sheets
  +---------------------+
```

### New Types

```rust
// crates/thetta-nesting/src/engine.rs

/// Dual-mode nesting engine: SPP (strip packing) or BPP (bin packing)
#[derive(Clone)]
pub enum NestingEngine {
    Spp(SppEngine),
    Bpp(BppEngine),
}

/// Existing SPP engine (renamed from current NestingEngine)
#[derive(Clone)]
pub struct SppEngine {
    pieces: Vec<Piece>,
    sheet: SheetConfig,
    pub instance: SPInstance,
    pub problem: SPProblem,
    pk_map: Vec<Option<PItemKey>>,
    orientations: Vec<RotationRange>,
}

/// New BPP engine wrapping jagua-rs BPProblem
#[derive(Clone)]
pub struct BppEngine {
    pieces: Vec<Piece>,
    sheet: SheetConfig,
    pub instance: BPInstance,
    pub problem: BPProblem,
    /// Maps (piece_index) -> Option<(LayKey, PItemKey)>
    pk_map: Vec<Option<(LayKey, PItemKey)>>,
    /// Tracks which LayKey corresponds to which sheet index in output
    layout_order: Vec<LayKey>,
    orientations: Vec<RotationRange>,
}
```

```rust
// crates/thetta-nesting/src/conversion.rs (new function)

/// Create a BPInstance from pieces and a fixed sheet size.
/// Single bin type replicated with high stock (100 sheets max).
pub fn pieces_to_bp_instance(
    pieces: &[Piece],
    sheet: &SheetConfig,
    orientations: Option<&[Vec<f64>]>,
) -> Result<BPInstance>
```

```rust
// crates/thetta-nesting/src/runner.rs (new enum variant)

/// Controls whether the pipeline uses SPP or BPP mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NestingMode {
    /// Strip packing: single infinite-width strip (benchmarks)
    Spp,
    /// Bin packing: fixed-size sheets, overflow to new sheets
    Bpp,
}
```

### Design Decisions

1. **Dual-mode engine via enum dispatch.** `NestingEngine` becomes an enum with `Spp(SppEngine)` and `Bpp(BppEngine)` variants. Both implement the same public API surface (`piece_count()`, `place_item()`, `collision_cost()`, `current_result()`, etc.) via methods on the enum that delegate to the inner engine. This keeps runner/LBF/GLS code unified -- they call `engine.place_item()` without knowing the mode. The alternative (trait object) was rejected because `NestingEngine` is cloned in GLS and stored by value.

2. **LBF for BPP: try existing sheets first, then open new bin.** When placing a piece, the LBF heuristic iterates all open layouts (sheets) looking for a valid placement. If no sheet has a collision-free position, it opens a new bin via `BPPlacement { layout_id: BPLayoutType::Closed { bin_id: 0 }, ... }`. This maximizes utilization of existing sheets before creating new ones. The order of sheet preference is first-opened-first-tried (FIFO), which tends to fill sheets sequentially.

3. **GLS for BPP: optimize each sheet independently.** Each open layout gets its own GLS optimization pass. This is simpler than cross-sheet moves and avoids complexity of tracking piece migrations between layouts. The trade-off is that we cannot rebalance utilization across sheets, but for the common case (sheets are sequentially filled), this is acceptable. Cross-sheet GLS is deferred to a future sprint.

4. **Viewport: side-by-side horizontal layout.** Sheets render left-to-right with a configurable gap (50mm world-space). The world coordinate system extends rightward: sheet N starts at `x = N * (sheet_width + gap)`. Existing zoom/pan works unchanged -- the user simply pans right to see more sheets. Sheet index labels render above each sheet outline. No tab/pagination UI needed.

5. **Multiple STEP files: accumulate pieces.** Loading a second STEP file appends its pieces to the existing piece list (same behavior as `load_json_file` already does for JSON). Each STEP file gets its own `StepModel` stored in a `Vec<Arc<Mutex<StepModel>>>` (replacing the current single `Option<Arc<Mutex<StepModel>>>`). Export iterates all models to find the correct solid for each piece. Thickness selection applies per-model.

6. **Backward compatibility: benchmarks stay SPP.** `load_benchmark_file()` sets `NestingMode::Spp`. STEP and JSON polygon workflows use `NestingMode::Bpp`. The mode is stored on `AppState` and passed through `run_optimizer()` -> `run_nesting()`.

7. **Container construction bypasses `Importer.import_container()`.** The jagua-rs `Importer` struct lacks an `import_container` method (the BPP import code references it but it doesn't exist). We construct `Container` directly via `Container::new_strip(id, width, height, cde_config, shape_modify_config)` which creates a rectangular container suitable for fixed-size sheets. The `Bin` wraps this container with `stock` and `cost`.

### Threading Model

No changes to the threading model. The optimizer thread already runs `run_nesting()` on a background thread with `mpsc::channel` and `Arc<AtomicBool>` cancel token. BPP mode runs within the same thread, just with a different engine variant. Multi-sheet LBF and per-sheet GLS are sequential (no per-sheet parallelism -- the collision detection engine is not thread-safe within a single problem instance).

## Implementation

### Phase 0: Enable BPP Feature + Container Construction

**Goal:** jagua-rs BPP types compile and a `BPInstance` can be constructed programmatically.

#### Task 0.1: Enable BPP feature flag
**File:** `crates/thetta-nesting/Cargo.toml` (MODIFY)
```toml
jagua-rs = { path = "../jagua-rs", features = ["spp", "bpp"] }
```

**File:** `crates/thetta-app/Cargo.toml` (MODIFY) -- if it has a direct jagua-rs dep
```toml
jagua-rs = { path = "../jagua-rs", features = ["spp", "bpp"] }
```

#### Task 0.2: Add `Importer.import_container()` to jagua-rs fork
**File:** `crates/jagua-rs/src/io/import.rs` (MODIFY)

The BPP import code calls `importer.import_container()` but this method does not exist on `Importer`. Add it:

```rust
impl Importer {
    pub fn import_container(&self, ext_container: &ExtContainer) -> Result<Container> {
        // ExtContainer doesn't exist either -- we construct Container directly
        // This is a shim for the BPP import path
        unimplemented!("use Container::new_strip directly")
    }
}
```

In practice, we bypass the import path entirely and construct `BPInstance` directly:

```rust
pub fn pieces_to_bp_instance(
    pieces: &[Piece],
    sheet: &SheetConfig,
    orientations: Option<&[Vec<f64>]>,
) -> Result<BPInstance> {
    // Build items identically to pieces_to_sp_instance
    let importer = Importer::new(DEFAULT_CDE_CONFIG, None,
        (sheet.separation > 0.0).then_some(sheet.separation as f32), None);

    let items: Vec<(Item, usize)> = pieces.iter().enumerate().map(|(idx, piece)| {
        let shape = polygon_to_ext_simple_polygon(&piece.polygon)?;
        let allowed_orientations = orientations
            .and_then(|ori| ori.get(idx))
            .map(|angles| angles.iter().map(|r| r.to_degrees() as f32).collect());
        let ext_item = ExtItem {
            id: idx as u64,
            allowed_orientations,
            shape: ExtShape::SimplePolygon(shape),
            min_quality: None,
        };
        let item = importer.import_item(&ext_item)?;
        Ok((item, 1usize)) // demand = 1 per piece
    }).collect::<Result<Vec<_>>>()?;

    // Build a single rectangular bin
    let shape_modify_config = ShapeModifyConfig {
        offset: (sheet.separation > 0.0).then_some(sheet.separation as f32 / 2.0),
        simplify_tolerance: None,
        narrow_concavity_cutoff: None,
    };
    let container = Container::new_strip(
        0, // bin id
        sheet.width as f32,
        sheet.height as f32,
        DEFAULT_CDE_CONFIG,
        shape_modify_config,
    )?;
    let bin = Bin::new(container, 100, 1); // stock=100, cost=1

    Ok(BPInstance::new(items, vec![bin]))
}
```

#### Task 0.3: Smoke test BPP instance construction
**File:** `crates/thetta-nesting/tests/bpp_smoke.rs` (NEW)

```rust
#[test]
fn create_bp_instance_and_problem() {
    let pieces = vec![rect_piece(0, 100.0, 50.0), rect_piece(1, 80.0, 40.0)];
    let sheet = SheetConfig { width: 200.0, height: 100.0, separation: 0.0 };
    let instance = pieces_to_bp_instance(&pieces, &sheet, None).unwrap();
    assert_eq!(instance.items.len(), 2);
    assert_eq!(instance.bins.len(), 1);
    assert_eq!(instance.bins[0].stock, 100);

    let problem = BPProblem::new(instance);
    assert_eq!(problem.layouts.len(), 0); // no layouts open yet
}
```

**Deliverable:** `cargo test -p thetta-nesting` passes with BPP instance construction.

### Phase 1: Dual-Mode NestingEngine

**Goal:** Refactor `NestingEngine` into an enum with SPP and BPP variants sharing a common API.

#### Task 1.1: Rename current engine to SppEngine
**File:** `crates/thetta-nesting/src/engine.rs` (MODIFY)

Rename the existing `NestingEngine` struct to `SppEngine`. All existing methods stay on `SppEngine`.

#### Task 1.2: Implement BppEngine
**File:** `crates/thetta-nesting/src/engine.rs` (MODIFY)

```rust
#[derive(Clone)]
pub struct BppEngine {
    pieces: Vec<Piece>,
    sheet: SheetConfig,
    pub instance: BPInstance,
    pub problem: BPProblem,
    /// piece_index -> Option<(layout_key, placed_item_key)>
    pk_map: Vec<Option<(LayKey, PItemKey)>>,
    /// Ordered list of layout keys for result extraction
    layout_order: Vec<LayKey>,
    orientations: Vec<RotationRange>,
}

impl BppEngine {
    pub fn new_with_orientations(
        pieces: &[Piece],
        sheet: &SheetConfig,
        orientations: Option<&[RotationRange]>,
    ) -> Self { ... }

    pub fn piece_count(&self) -> usize { self.pieces.len() }
    pub fn piece_area(&self, idx: usize) -> f64 { ... }
    pub fn sheet_config(&self) -> &SheetConfig { &self.sheet }
    pub fn rotation_range(&self, idx: usize) -> Option<&RotationRange> { ... }

    /// Number of open sheets (layouts)
    pub fn sheet_count(&self) -> usize { self.layout_order.len() }

    /// Open a new sheet, returns its index
    pub fn open_sheet(&mut self) -> usize {
        let (lkey, _pik) = self.problem.place_item(BPPlacement {
            layout_id: BPLayoutType::Closed { bin_id: 0 },
            // We need a dummy placement to open the bin -- instead,
            // we create a Layout directly
            ...
        });
        // Actually: construct Layout from Container and register it
        let container = self.instance.bins[0].container.clone();
        let layout = Layout::new(container);
        let lkey = self.problem.register_layout(layout);
        let idx = self.layout_order.len();
        self.layout_order.push(lkey);
        idx
    }

    /// Place item on a specific sheet. Returns true if collision-free.
    pub fn place_item_on_sheet(
        &mut self, idx: usize, sheet_idx: usize, x: f64, y: f64, rotation: f64,
    ) -> bool { ... }

    /// Force-place item on a specific sheet (allows collisions).
    pub fn force_place_item_on_sheet(
        &mut self, idx: usize, sheet_idx: usize, x: f64, y: f64, rotation: f64,
    ) { ... }

    /// Check collision cost on a specific sheet.
    pub fn collision_cost_on_sheet(
        &self, idx: usize, sheet_idx: usize, x: f64, y: f64, rotation: f64,
    ) -> f64 { ... }

    /// Get the Layout for a given sheet index (for GLS per-sheet optimization)
    pub fn layout_key(&self, sheet_idx: usize) -> Option<LayKey> {
        self.layout_order.get(sheet_idx).copied()
    }

    pub fn current_result(&self) -> NestingResult { ... }
    pub fn remove_item(&mut self, idx: usize) { ... }
    pub fn placement_of(&self, idx: usize) -> Option<(usize, Placement)> { ... }
}
```

#### Task 1.3: Create NestingEngine enum with delegation
**File:** `crates/thetta-nesting/src/engine.rs` (MODIFY)

```rust
#[derive(Clone)]
pub enum NestingEngine {
    Spp(SppEngine),
    Bpp(BppEngine),
}

impl NestingEngine {
    /// Create SPP engine (strip packing, for benchmarks)
    pub fn new(pieces: &[Piece], sheet: &SheetConfig) -> Self {
        NestingEngine::Spp(SppEngine::new(pieces, sheet))
    }

    /// Create SPP engine with orientations (backward compat)
    pub fn new_with_orientations(
        pieces: &[Piece], sheet: &SheetConfig,
        orientations: Option<&[RotationRange]>,
    ) -> Self {
        NestingEngine::Spp(SppEngine::new_with_orientations(pieces, sheet, orientations))
    }

    /// Create BPP engine (fixed-size sheet packing)
    pub fn new_bpp(
        pieces: &[Piece], sheet: &SheetConfig,
        orientations: Option<&[RotationRange]>,
    ) -> Self {
        NestingEngine::Bpp(BppEngine::new_with_orientations(pieces, sheet, orientations))
    }

    // Delegate common methods:
    pub fn piece_count(&self) -> usize { ... }
    pub fn piece_area(&self, idx: usize) -> f64 { ... }
    pub fn sheet_config(&self) -> &SheetConfig { ... }
    pub fn rotation_range(&self, idx: usize) -> Option<&RotationRange> { ... }
    pub fn collision_cost(&self, idx: usize, x: f64, y: f64, rotation: f64) -> f64 { ... }
    pub fn place_item(&mut self, idx: usize, x: f64, y: f64, rotation: f64) -> bool { ... }
    pub fn force_place_item(&mut self, idx: usize, x: f64, y: f64, rotation: f64) { ... }
    pub fn remove_item(&mut self, idx: usize) { ... }
    pub fn placement_of(&self, idx: usize) -> Option<Placement> { ... }
    pub fn current_result(&self) -> NestingResult { ... }
    pub fn strip_width(&self) -> f64 { ... }
    pub fn fit_strip(&mut self) { ... }
    pub fn change_strip_width(&mut self, width: f64) { ... }
    pub fn clear_placements(&mut self) { ... }
}
```

For `collision_cost`, `place_item`, `force_place_item` in BPP mode: these delegate to the first open sheet (sheet 0) by default, maintaining backward compatibility with LBF/GLS code that doesn't specify a sheet. The BPP-aware LBF will use `place_item_on_sheet` directly.

**Deliverable:** All existing tests pass with the refactored enum engine (SPP path unchanged).

### Phase 2: Multi-Bin LBF

**Goal:** LBF places pieces across multiple sheets, opening new bins when needed.

#### Task 2.1: Implement `run_lbf` for BPP mode
**File:** `crates/thetta-nesting/src/lbf.rs` (MODIFY)

The existing `run_lbf` works on a `NestingEngine` directly. For BPP, we add `run_lbf_bpp`:

```rust
pub fn run_lbf_bpp(
    engine: &mut BppEngine,
    config: LbfConfig,
    cancel: Option<&Arc<AtomicBool>>,
    mut on_placed: impl FnMut(&BppEngine),
) -> NestingResult {
    let mut order = (0..engine.piece_count()).collect::<Vec<_>>();
    order.sort_by(|a, b| {
        engine.piece_area(*b)
            .partial_cmp(&engine.piece_area(*a))
            .unwrap_or(Ordering::Equal)
    });

    let sheet = engine.sheet_config().clone();
    let grid_n = config.grid_n.max(2);

    // Ensure at least one sheet is open
    if engine.sheet_count() == 0 {
        engine.open_sheet();
    }

    for idx in order {
        if is_cancelled(cancel) { break; }

        let rotations = compute_rotations(engine, idx, &config);
        let mut placed = false;

        // Try each existing sheet
        for sheet_idx in 0..engine.sheet_count() {
            if let Some((x, y, r)) = find_lbf_position(
                engine, idx, sheet_idx, &sheet, grid_n, &rotations,
            ) {
                engine.place_item_on_sheet(idx, sheet_idx, x, y, r);
                placed = true;
                break;
            }
        }

        // If no existing sheet works, open a new one
        if !placed {
            let new_sheet_idx = engine.open_sheet();
            if let Some((x, y, r)) = find_lbf_position(
                engine, idx, new_sheet_idx, &sheet, grid_n, &rotations,
            ) {
                engine.place_item_on_sheet(idx, new_sheet_idx, x, y, r);
                placed = true;
            }
            // If still can't place (piece larger than sheet), leave unplaced
        }

        if placed {
            on_placed(engine);
        }
    }

    engine.current_result()
}

/// Grid search for bottom-left position on a specific sheet
fn find_lbf_position(
    engine: &BppEngine,
    idx: usize,
    sheet_idx: usize,
    sheet: &SheetConfig,
    grid_n: usize,
    rotations: &[f64],
) -> Option<(f64, f64, f64)> {
    let mut best: Option<(f64, f64, f64)> = None;

    for gx in 0..grid_n {
        for gy in 0..grid_n {
            let x = sheet.width * gx as f64 / (grid_n - 1) as f64;
            let y = sheet.height * gy as f64 / (grid_n - 1) as f64;

            for r in rotations {
                if engine.collision_cost_on_sheet(idx, sheet_idx, x, y, *r) == 0.0 {
                    let cand = (y, x, *r);
                    let better = best
                        .map(|b| cand.0 < b.0 || (cand.0 == b.0 && cand.1 < b.1))
                        .unwrap_or(true);
                    if better {
                        best = Some(cand);
                    }
                }
            }
        }
    }

    best.map(|(y, x, r)| (x, y, r))
}
```

#### Task 2.2: Unified LBF dispatch
**File:** `crates/thetta-nesting/src/lbf.rs` (MODIFY)

Keep existing `run_lbf` for SPP. Add a top-level dispatch:

```rust
pub fn run_lbf_dispatch(
    engine: &mut NestingEngine,
    config: LbfConfig,
    cancel: Option<&Arc<AtomicBool>>,
    on_placed: impl FnMut(&NestingEngine),
) -> NestingResult {
    match engine {
        NestingEngine::Spp(spp) => run_lbf(spp, config, cancel, ...),
        NestingEngine::Bpp(bpp) => run_lbf_bpp(bpp, config, cancel, ...),
    }
}
```

**Deliverable:** LBF places pieces across multiple sheets. Integration test: 10 rectangles that don't fit on one sheet produce 2+ sheets.

### Phase 3: Per-Sheet GLS Optimization

**Goal:** GLS runs independently on each sheet to resolve overlaps and improve density.

#### Task 3.1: Per-sheet GLS wrapper
**File:** `crates/thetta-nesting/src/gls.rs` (MODIFY)

Add `GLSOptimizer` support for BPP by running GLS on each layout independently:

```rust
pub fn run_gls_bpp(
    engine: &mut BppEngine,
    config: OptimizerConfig,
    on_update: impl FnMut(NestingResult),
    on_progress: impl FnMut(usize, f64, f64, usize),
    cancel: Option<&Arc<AtomicBool>>,
) -> NestingResult {
    // For each sheet, extract the pieces on that sheet,
    // create a temporary SppEngine for just those pieces,
    // run GLS, then apply the results back to the BppEngine.
    for sheet_idx in 0..engine.sheet_count() {
        if is_cancelled(cancel) { break; }

        let (sub_pieces, sub_orientations, index_map) =
            engine.pieces_on_sheet(sheet_idx);
        if sub_pieces.is_empty() { continue; }

        let mut sub_engine = SppEngine::new_with_orientations(
            &sub_pieces,
            engine.sheet_config(),
            Some(&sub_orientations),
        );
        // Seed sub_engine with current placements from LBF
        for (sub_idx, &orig_idx) in index_map.iter().enumerate() {
            if let Some((_si, placement)) = engine.placement_of(orig_idx) {
                sub_engine.force_place_item(sub_idx, placement.x, placement.y, placement.rotation);
            }
        }

        let mut gls = GLSOptimizer::new(sub_engine, config.clone());
        gls.run(|_| {}, |_, _, _, _| {}, Duration::MAX, cancel);

        // Write optimized placements back to BppEngine
        for (sub_idx, &orig_idx) in index_map.iter().enumerate() {
            if let Some(p) = gls.engine.placement_of(sub_idx) {
                engine.force_place_item_on_sheet(
                    orig_idx, sheet_idx, p.x, p.y, p.rotation,
                );
            }
        }

        // Report progress
        let result = engine.current_result();
        on_update(result);
    }

    engine.current_result()
}
```

This approach reuses the existing `GLSOptimizer` and `SppEngine` without modification -- each sheet is treated as an independent SPP sub-problem. The limitation is no cross-sheet piece migration, which is acceptable for Sprint 004.

#### Task 3.2: Unified GLS dispatch in runner
**File:** `crates/thetta-nesting/src/runner.rs` (MODIFY)

```rust
pub fn run_nesting(
    pieces: Vec<Piece>,
    sheet: SheetConfig,
    orientations: Option<Vec<RotationRange>>,
    config: OptimizerConfig,
    mode: NestingMode,
    tx: mpsc::Sender<NestingUpdate>,
    cancel: Arc<AtomicBool>,
) {
    match mode {
        NestingMode::Spp => {
            // Existing SPP pipeline: LBF -> GLS -> compress_strip
            run_nesting_spp(pieces, sheet, orientations, config, tx, cancel);
        }
        NestingMode::Bpp => {
            // BPP pipeline: LBF across bins -> per-sheet GLS
            run_nesting_bpp(pieces, sheet, orientations, config, tx, cancel);
        }
    }
}

fn run_nesting_bpp(
    pieces: Vec<Piece>,
    sheet: SheetConfig,
    orientations: Option<Vec<RotationRange>>,
    config: OptimizerConfig,
    tx: mpsc::Sender<NestingUpdate>,
    cancel: Arc<AtomicBool>,
) {
    let mut engine = BppEngine::new_with_orientations(
        &pieces, &sheet, orientations.as_deref(),
    );

    // Phase 1: LBF placement across sheets
    let baseline = run_lbf_bpp(&mut engine, LbfConfig {
        orientations: orientations.clone(),
        ..LbfConfig::default()
    }, Some(&cancel), |eng| {
        let _ = tx.send(NestingUpdate::Improved(eng.current_result()));
    });
    let _ = tx.send(NestingUpdate::Improved(baseline.clone()));

    if cancel.load(Ordering::Relaxed) {
        let _ = tx.send(NestingUpdate::Finished(baseline));
        return;
    }

    // Phase 2: Per-sheet GLS
    let gls_result = run_gls_bpp(
        &mut engine, config, |r| {
            let _ = tx.send(NestingUpdate::Improved(r));
        }, |iter, overlap, best, feasible| {
            let _ = tx.send(NestingUpdate::Progress {
                iteration: iter, total_overlap: overlap,
                best_overlap: best, feasible_count: feasible,
            });
        }, Some(&cancel),
    );

    let _ = tx.send(NestingUpdate::Finished(gls_result));
}
```

Note: Strip compression is not applicable to BPP (sheets have fixed width). Omitted from BPP pipeline.

**Deliverable:** BPP pipeline produces multi-sheet results with overlap-free placement.

### Phase 4: Multi-Sheet Viewport Rendering

**Goal:** Viewport renders all sheets side-by-side with labels and navigation.

#### Task 4.1: Render all sheets
**File:** `crates/thetta-app/src/viewport.rs` (MODIFY)

Replace the current `result.sheets.first()` single-sheet rendering with a loop over all sheets:

```rust
pub fn show_viewport(
    ui: &mut egui::Ui,
    state: &mut ViewportState,
    app: &AppState,
) -> Option<PieceId> {
    // ... existing setup ...

    let sheet_count = app.current_result
        .as_ref()
        .map(|r| r.sheets.len())
        .unwrap_or(1)
        .max(1);
    let gap = 50.0; // world-space gap between sheets
    let total_width = sheet_count as f64 * (sheet_w as f64 + gap) - gap;

    // Adjust world rect to encompass all sheets
    // ... existing zoom/pan code, but with wider world ...

    // Draw each sheet
    if let Some(result) = &app.current_result {
        for (sheet_idx, sheet) in result.sheets.iter().enumerate() {
            let x_offset = sheet_idx as f32 * (sheet_w + gap as f32);

            // Sheet outline
            painter.rect_stroke(
                to_screen.transform_rect(Rect::from_min_max(
                    Pos2::new(x_offset, 0.0),
                    Pos2::new(x_offset + sheet_w, sheet_h),
                )),
                0.0,
                Stroke::new(2.0, Color32::WHITE),
                egui::StrokeKind::Outside,
            );

            // Sheet label
            let label_pos = to_screen.transform_pos(
                Pos2::new(x_offset + sheet_w * 0.5, -15.0)
            );
            painter.text(
                label_pos,
                egui::Align2::CENTER_BOTTOM,
                format!("Sheet {}", sheet_idx + 1),
                egui::FontId::proportional(14.0),
                Color32::WHITE,
            );

            // Draw pieces on this sheet
            for placement in &sheet.placements {
                if let Some(piece) = app.pieces.iter().find(|p| p.id == placement.piece_id) {
                    let poly = placed_polygon(&piece.polygon, placement);
                    let points = poly.outer.iter().map(|p| {
                        to_screen.transform_pos(Pos2::new(
                            p.x as f32 + x_offset,
                            p.y as f32,
                        ))
                    }).collect::<Vec<_>>();

                    // ... existing triangulation + stroke + selection code ...
                }
            }
        }
    }

    click_hit
}
```

#### Task 4.2: Adjust zoom-to-fit for multiple sheets
**File:** `crates/thetta-app/src/viewport.rs` (MODIFY)

Update the base scale computation to consider total width across all sheets:

```rust
let total_w = sheet_count as f32 * sheet_w + (sheet_count - 1).max(0) as f32 * gap as f32;
let scale_x = rect.width() / total_w.max(1.0);
let scale_y = rect.height() / sheet_h;
let base_scale = scale_x.min(scale_y);
```

**Deliverable:** Multiple sheets render side-by-side. Clicking selects pieces on any sheet.

### Phase 5: Stats Aggregation and Panel Updates

**Goal:** Side panel shows per-sheet and aggregate statistics. Sheet count displayed.

#### Task 5.1: Aggregate stats across sheets
**File:** `crates/thetta-app/src/state.rs` (MODIFY)

Update `apply_result` to aggregate across all sheets:

```rust
fn apply_result(&mut self, result: NestingResult) {
    let total_placed: usize = result.sheets.iter()
        .map(|s| s.placements.len()).sum();
    let total_area: f64 = result.sheets.iter()
        .map(|s| s.utilization * self.sheet_config.width * self.sheet_config.height)
        .sum();
    let total_sheet_area = result.sheets.len() as f64
        * self.sheet_config.width * self.sheet_config.height;
    let avg_utilization = if total_sheet_area > 0.0 {
        (total_area / total_sheet_area) * 100.0
    } else { 0.0 };

    self.stats.pieces_placed = total_placed;
    self.stats.pieces_total = self.pieces.len();
    self.stats.utilization_pct = avg_utilization;
    self.stats.strip_length = result.sheets.first()
        .map(|s| s.strip_length)
        .unwrap_or(self.sheet_config.width);
    self.stats.sheet_count = result.sheets.len();
    self.current_result = Some(result);
}
```

#### Task 5.2: Add NestingMode to AppState
**File:** `crates/thetta-app/src/state.rs` (MODIFY)

```rust
pub struct AppState {
    // ... existing fields ...
    pub nesting_mode: NestingMode,
    // ... new stats field ...
}
```

Set `nesting_mode = NestingMode::Bpp` by default (for STEP and JSON workflows), override to `NestingMode::Spp` when loading benchmarks.

#### Task 5.3: Add sheet_count to NestingStats
**File:** `crates/thetta-app/src/state.rs` (MODIFY)

```rust
pub struct NestingStats {
    // ... existing fields ...
    pub sheet_count: usize,
}
```

#### Task 5.4: Update side panel with sheet info
**File:** `crates/thetta-app/src/panels.rs` (MODIFY)

In the Stats section, add:

```rust
ui.label(format!("Sheets: {}", app.stats.sheet_count));
if app.stats.sheet_count > 1 {
    // Per-sheet utilization breakdown
    if let Some(result) = &app.current_result {
        for (i, sheet) in result.sheets.iter().enumerate() {
            ui.label(format!(
                "  Sheet {}: {:.1}% ({} pcs)",
                i + 1,
                sheet.utilization * 100.0,
                sheet.placements.len(),
            ));
        }
    }
}
```

**Deliverable:** Stats panel shows sheet count and per-sheet utilization.

### Phase 6: Multiple STEP File Accumulation

**Goal:** Loading additional STEP files adds pieces without replacing existing ones.

#### Task 6.1: Support multiple StepModel sessions
**File:** `crates/thetta-app/src/state.rs` (MODIFY)

Replace `step_session: Option<Arc<Mutex<StepModel>>>` with:

```rust
pub step_sessions: Vec<StepSession>,

pub struct StepSession {
    pub model: Arc<Mutex<StepModel>>,
    pub piece_refs: Vec<StepPieceRef>,
    pub source_path: PathBuf,
    pub thickness_groups: Vec<ThicknessInfo>,
    pub selected_thickness_idx: usize,
}
```

#### Task 6.2: Update apply_step_import to accumulate
**File:** `crates/thetta-app/src/state.rs` (MODIFY)

When a new STEP file is imported, create a new `StepSession` and append it to `step_sessions`. The existing pieces from previous files remain. Nesting restarts with all accumulated pieces.

```rust
fn apply_step_import(&mut self, result: StepImportResult) {
    let path = result.model.source_path().to_path_buf();
    let session = StepSession {
        model: Arc::new(Mutex::new(result.model)),
        piece_refs: result.piece_refs.clone(),
        source_path: path.clone(),
        thickness_groups: result.thicknesses.clone(),
        selected_thickness_idx: /* best match */,
    };
    self.step_sessions.push(session);
    self.assign_file_color(&path);

    // Append pieces (don't replace)
    let count = result.pieces.len();
    self.pieces.extend(result.pieces);
    self.piece_orientations.extend(
        (0..count).map(|_| RotationRange::Continuous)
    );
    self.loaded_files.push(LoadedFile {
        path,
        piece_count: count,
    });
    self.stats.pieces_total = self.pieces.len();
}
```

#### Task 6.3: Update STEP export to use all sessions
**File:** `crates/thetta-app/src/state.rs` (MODIFY)

Export iterates all `StepSession`s to find the correct model for each piece:

```rust
pub fn start_step_export(&mut self, out_dir: PathBuf, file_stem: String) {
    let sessions = self.step_sessions.clone();
    let result = self.current_result.clone().unwrap();
    let (tx, rx) = mpsc::channel();
    // ... spawn thread that iterates sessions for piece lookup ...
}
```

#### Task 6.4: Per-file thickness selection in panel
**File:** `crates/thetta-app/src/panels.rs` (MODIFY)

Show thickness groups per loaded STEP file:

```rust
for (file_idx, session) in app.step_sessions.iter().enumerate() {
    ui.label(format!("File: {}", session.source_path.file_name()...));
    for (idx, group) in session.thickness_groups.iter().enumerate() {
        // Radio button for thickness selection within this file
    }
}
```

**Deliverable:** Loading multiple STEP files accumulates pieces and re-nests across sheets.

### Phase 7: Wire Up Pipeline + Pass NestingMode

**Goal:** `start_optimizer` passes `NestingMode` through to `run_optimizer`.

#### Task 7.1: Update run_optimizer signature
**File:** `crates/thetta-nesting/src/optimizer.rs` (MODIFY)

```rust
pub fn run_optimizer(
    pieces: Vec<Piece>,
    sheet_config: SheetConfig,
    orientations: Option<Vec<RotationRange>>,
    config: OptimizerConfig,
    mode: NestingMode,
    tx: mpsc::Sender<NestingUpdate>,
    cancel: Arc<AtomicBool>,
) {
    run_nesting(pieces, sheet_config, orientations, config, mode, tx, cancel);
}
```

#### Task 7.2: Update start_optimizer in AppState
**File:** `crates/thetta-app/src/state.rs` (MODIFY)

Pass `self.nesting_mode` to `run_optimizer`.

**Deliverable:** Full pipeline works end-to-end with mode switching.

### Phase 8: Testing

#### Task 8.1: BPP unit tests
**File:** `crates/thetta-nesting/tests/bpp_unit.rs` (NEW)

```rust
#[test]
fn bpp_instance_construction() { ... }

#[test]
fn bpp_engine_single_sheet_fits_all() {
    // All pieces fit on one sheet -> 1 sheet in result
}

#[test]
fn bpp_engine_overflow_to_second_sheet() {
    // 5 large pieces that need 2 sheets -> 2 sheets in result
}

#[test]
fn bpp_engine_piece_larger_than_sheet() {
    // Piece bigger than sheet -> unplaced
}
```

#### Task 8.2: Multi-bin LBF integration test
**File:** `crates/thetta-nesting/tests/bpp_integration.rs` (NEW)

```rust
#[test]
fn lbf_bpp_places_all_pieces_across_sheets() {
    let pieces = generate_pieces(20, 300.0, 200.0); // 20 pieces, each ~300x200
    let sheet = SheetConfig { width: 1000.0, height: 500.0, separation: 5.0 };
    let mut engine = BppEngine::new_with_orientations(&pieces, &sheet, None);
    let result = run_lbf_bpp(&mut engine, LbfConfig::default(), None, |_| {});

    assert!(result.unplaced.is_empty(), "all pieces should be placed");
    assert!(result.sheets.len() >= 2, "should need multiple sheets");
    // Verify no overlaps per sheet
    for sheet in &result.sheets {
        verify_no_overlaps(sheet);
    }
}
```

#### Task 8.3: Benchmark regression test
**File:** `crates/thetta-nesting/tests/benchmark_smoke.rs` (MODIFY)

Ensure existing benchmark tests still pass with the refactored engine. No changes needed if the SPP path is backward-compatible.

#### Task 8.4: End-to-end pipeline test
**File:** `crates/thetta-nesting/tests/e2e_bpp.rs` (NEW)

```rust
#[test]
fn full_bpp_pipeline() {
    let (tx, rx) = mpsc::channel();
    let cancel = Arc::new(AtomicBool::new(false));
    let pieces = generate_pieces(15, 400.0, 300.0);
    let sheet = SheetConfig::default(); // 2440x1220

    run_nesting(pieces, sheet, None, OptimizerConfig::default(),
                NestingMode::Bpp, tx, cancel.clone());

    // Collect all updates
    let mut final_result = None;
    while let Ok(update) = rx.recv_timeout(Duration::from_secs(30)) {
        if let NestingUpdate::Finished(r) = update {
            final_result = Some(r);
            break;
        }
    }
    let result = final_result.expect("should receive finished");
    assert!(result.unplaced.is_empty());
    assert!(result.sheets.len() >= 1);
}
```

#### Task 8.5: Build verification
```sh
cargo test --workspace
cargo test -p thetta-nesting
cargo build --release -p thetta-app
```

## Files Summary

### New Files

| File | Purpose | Est. LOC |
|------|---------|----------|
| `crates/thetta-nesting/tests/bpp_smoke.rs` | BPP instance construction smoke test | 40 |
| `crates/thetta-nesting/tests/bpp_unit.rs` | BPP engine unit tests (single/multi-sheet, overflow) | 120 |
| `crates/thetta-nesting/tests/bpp_integration.rs` | Multi-bin LBF + GLS integration tests | 100 |
| `crates/thetta-nesting/tests/e2e_bpp.rs` | Full BPP pipeline end-to-end test | 60 |
| **Total new** | | **~320** |

### Modified Files

| File | Changes | Est. Delta |
|------|---------|------------|
| `crates/thetta-nesting/Cargo.toml` | Add `"bpp"` to jagua-rs features | +1 |
| `crates/thetta-nesting/src/engine.rs` | Rename to `SppEngine`, add `BppEngine`, create `NestingEngine` enum with delegation | +300 |
| `crates/thetta-nesting/src/conversion.rs` | Add `pieces_to_bp_instance()` function | +60 |
| `crates/thetta-nesting/src/lbf.rs` | Add `run_lbf_bpp()` with multi-sheet placement, `find_lbf_position()` helper | +80 |
| `crates/thetta-nesting/src/gls.rs` | Add `run_gls_bpp()` per-sheet GLS wrapper | +60 |
| `crates/thetta-nesting/src/runner.rs` | Add `NestingMode` enum, `run_nesting_bpp()`, update `run_nesting()` signature | +60 |
| `crates/thetta-nesting/src/optimizer.rs` | Pass `NestingMode` through | +5 |
| `crates/thetta-nesting/src/compression.rs` | No changes (SPP only) | 0 |
| `crates/thetta-app/Cargo.toml` | Add `"bpp"` to jagua-rs features (if direct dep) | +1 |
| `crates/thetta-app/src/state.rs` | Add `NestingMode`, `StepSession` vec, `sheet_count` stat, update `apply_result()`, `apply_step_import()`, `start_step_export()` | +120 |
| `crates/thetta-app/src/viewport.rs` | Multi-sheet rendering loop, sheet labels, adjusted zoom-to-fit | +60 |
| `crates/thetta-app/src/panels.rs` | Sheet count display, per-sheet stats, per-file thickness groups | +40 |
| `crates/thetta-app/src/main.rs` | Pass `nesting_mode` to `start_optimizer` | +5 |
| `crates/thetta-nesting/tests/benchmark_smoke.rs` | Verify backward compat (may need minor signature update) | +5 |
| `crates/thetta-nesting/tests/engine_integration.rs` | Update for enum engine (may need minor signature update) | +5 |
| **Total modified** | | **~800 delta** |

### Estimated Total Effort: ~1120 LOC (new + delta)

Current codebase is ~14.7K LOC across all crates (post-Sprint 003).

## Definition of Done

### P0 -- Must Ship
- [ ] `BPInstance` can be constructed from `Vec<Piece>` + `SheetConfig` with a single rectangular bin type
- [ ] `BppEngine` wraps `BPProblem` and supports `place_item_on_sheet()`, `collision_cost_on_sheet()`, `current_result()`
- [ ] `NestingEngine` enum dispatches to `SppEngine` or `BppEngine` transparently
- [ ] LBF places pieces across multiple sheets, opening new bins when needed
- [ ] Pieces that exceed one sheet automatically overflow to additional sheets
- [ ] GLS runs per-sheet to resolve overlaps within each sheet
- [ ] `NestingResult.sheets` has one `Sheet` per bin with correct placements, utilization, and strip_length
- [ ] Viewport renders all sheets side-by-side with sheet outline and label
- [ ] Stats panel shows sheet count and aggregate utilization
- [ ] Benchmark mode (sparrow JSON) uses SPP with identical behavior to Sprint 003
- [ ] All existing tests pass (`cargo test --workspace`)
- [ ] No regressions in benchmark results

### P1 -- Should Ship
- [ ] Per-sheet utilization breakdown in stats panel
- [ ] Multiple STEP files accumulate pieces (not replace)
- [ ] Per-file thickness selection in the side panel
- [ ] STEP export iterates all sessions to find correct model per piece
- [ ] Click-to-select works across all sheets in viewport
- [ ] Zoom-to-fit adjusts for total width of all sheets
- [ ] Piece too large for any sheet appears in `unplaced` list
- [ ] Integration test: 20+ pieces across 3+ sheets, all placed, no overlaps

### P2 -- Stretch
- [ ] Cross-sheet GLS moves (migrate pieces between sheets for better balance)
- [ ] Sheet-count compression (try removing last sheet, redistribute pieces)
- [ ] Sheet tabs or pagination as alternative to side-by-side rendering
- [ ] Per-sheet export selection ("Export sheet 2 only")
- [ ] Visual indicator for which sheet a selected piece belongs to
- [ ] Maximum sheet count configuration in UI

### Cut if Behind Schedule
- Multiple STEP file accumulation (P1) -- keep single-file behavior
- Per-file thickness selection (P1) -- show only most recent file
- Cross-sheet GLS (P2)
- Sheet-count compression (P2)

## Risks

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| jagua-rs `BPProblem` API does not support opening bins without an initial placement (no `register_layout` public method) | High | **High** | `BPProblem.register_layout` is private. Workaround: place first item with `BPLayoutType::Closed { bin_id: 0 }` to open the bin, or make `register_layout` public in our fork. Since we own the fork, exposing it is straightforward. |
| `Importer.import_container()` does not exist -- BPP import path is broken in jagua-rs fork | Medium | **Confirmed** | Bypass the BPP import path entirely. Construct `Container::new_strip()` and `Bin::new()` directly. No changes to jagua-rs import module needed. |
| Per-sheet GLS using temporary `SppEngine` loses cross-sheet optimization opportunities | Medium | Medium | Acceptable for Sprint 004. Sequential sheet filling + per-sheet GLS produces good results for typical workloads. Cross-sheet moves deferred to future sprint. |
| BPP engine `collision_cost_on_sheet` requires accessing individual layout CDEs, which may have different API than SPP's single layout | Medium | Medium | `BPProblem.layouts` is a public `SlotMap<LayKey, Layout>`. Each `Layout` has its own CDE accessible via `layout.cde()`. Index by `LayKey` from `layout_order`. |
| Viewport performance with many sheets (10+) | Low | Low | Only render sheets visible in current viewport rect. Cull off-screen sheets. For typical usage (2-5 sheets), this is not an issue. |
| Piece ID conflicts when accumulating across multiple STEP files | Medium | Medium | STEP import uses `PieceId(solid_index)` which may collide across files. Mitigation: offset IDs by cumulative piece count, or use a global ID allocator. |
| `BPProblem.place_item()` decrements `item_demand_qtys` which we set to 1 per piece -- placing the same piece twice panics | High | Medium | Each piece has demand=1 and is placed exactly once. Ensure `force_place_item_on_sheet` removes before re-placing. The `remove_item` on BPProblem returns the placement and increments demand back. |
| Engine refactor breaks existing tests | Medium | Low | Phase 1 ensures SPP path is byte-for-byte identical via delegation. Run existing test suite after each phase. |

## Security

- No new network access. Fully local application.
- No new dependencies added (only enabling existing jagua-rs `bpp` feature).
- All jagua-rs BPP types are in our local fork -- no new external crate attack surface.
- Multiple STEP file loading uses existing `rfd` file dialog and OCCT import -- no new file I/O paths.
- No user-provided code execution.
- Risk: unbounded sheet creation if pieces are very small relative to sheet size. Mitigation: cap at 100 sheets (matches `Bin.stock = 100`).

## Dependencies

### New Rust Crates

None. This sprint only enables the existing `bpp` feature on the local jagua-rs fork.

### Modified Crate Features

| Crate | Change | Purpose |
|-------|--------|---------|
| `jagua-rs` | Add `features = ["bpp"]` alongside existing `["spp"]` | Enable BPP types: `BPInstance`, `BPProblem`, `BPPlacement`, `BPLayoutType`, `Bin`, `LayKey` |

### Existing Crates (Unchanged)

| Crate | Purpose |
|-------|---------|
| `thetta-geometry` | Shared types (no changes -- `NestingResult` already has `Vec<Sheet>`) |
| `thetta-step` | STEP import/export (no changes -- export already iterates sheets) |
| `jagua-rs` | Collision detection engine (no source changes, only feature flag) |
| `eframe` / `egui` | GUI framework |
| `rfd` | Native file dialogs |

### System Dependencies

No new system dependencies.

## Open Questions

1. **`BPProblem.register_layout` is private.** To open a new bin/sheet in the BPP engine, we need to either: (a) make `register_layout` public in our jagua-rs fork (1-line change), or (b) use the `place_item` API with `BPLayoutType::Closed { bin_id: 0 }` to open a new layout via the first piece placement. Option (a) is cleaner for our use case. **Recommend: make it `pub` in our fork.**

2. **Piece ID uniqueness across STEP files.** Current `PieceId(solid_index)` starts from 0 per file. When accumulating multiple files, IDs will collide (two different pieces both with `PieceId(0)`). Options: (a) offset by cumulative count, (b) hash source path + index, (c) global atomic counter. **Recommend: offset-based -- `PieceId(cumulative_offset + solid_index)`.**

3. **GLS time budget allocation across sheets.** With N sheets, should GLS spend `time_limit / N` per sheet, or `time_limit` total with early termination? Spending equal time per sheet is simpler. **Recommend: equal per-sheet budget, capped at `time_limit` total.**

4. **Should BPP mode use strip compression within each sheet?** Each bin has fixed width, so "strip compression" doesn't apply directly. However, we could try tighter packing within each sheet by running compression on the sub-SPP. **Recommend: skip for Sprint 004, add as P2 if utilization is poor.**

5. **Maximum sheet count.** The `Bin.stock = 100` caps at 100 sheets. Is this sufficient? For typical sheet-metal workflows (10-30 parts across 2-5 sheets), yes. **Recommend: keep stock=100, add UI cap as P2.**

6. **Viewport: separate zoom per sheet or unified zoom?** Unified zoom (all sheets at same scale) is simpler and more intuitive for comparison. **Recommend: unified zoom, which is the natural result of side-by-side rendering.**
