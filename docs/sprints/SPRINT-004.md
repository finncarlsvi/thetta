# Sprint 004: Multi-Sheet Nesting

## Overview

Implement multi-sheet nesting so pieces that exceed one fixed-size sheet overflow onto additional sheets automatically. This sprint adds a `MultiSheetEngine` orchestrator that manages multiple independent SPP engines (one per sheet), adapts LBF for fill-and-overflow placement, runs GLS independently per sheet, renders all sheets side-by-side in the viewport, and aggregates statistics across sheets.

**Scope:** `MultiSheetEngine` orchestrator, multi-sheet LBF with overflow, per-sheet independent GLS and compression, side-by-side viewport rendering, per-sheet + aggregate stats, `NestingMode` dispatch in runner/optimizer.
**Prerequisites:** Sprint 003 complete. The nesting engine wraps jagua-rs `SPProblem` and accepts `Vec<Piece>` with optional `Vec<RotationRange>` orientations. It produces `NestingResult` with `Vec<Sheet>`. STEP import/export pipeline is operational. The app supports background optimizer with cancel token.
**Deferred:** Multiple STEP file loading (Sprint 005), cross-sheet GLS piece migration, sheet-count minimization compression, per-sheet export selection UI, heterogeneous sheet sizes.

## Use Cases

### UC1: Automatic Multi-Sheet Overflow
- User loads a STEP file (or JSON polygon file) with many pieces
- App nests pieces into the first 2440×1220mm sheet
- When a piece cannot fit on the current sheet after grid search, a new sheet is opened
- LBF tries all existing sheets before opening a new one (fill-and-spill)
- Process repeats until all pieces are placed or all pieces have been attempted
- Pieces that exceed sheet dimensions in any orientation are reported as unplaced

### UC2: View All Sheets Side-by-Side
- After nesting, the viewport shows all sheets laid out horizontally with gaps
- Each sheet has a label above it ("Sheet 1", "Sheet 2", etc.)
- Zoom and pan cover all sheets in a unified coordinate space
- Click-to-select works across all sheets
- Sheet boundary rectangles are drawn for each sheet

### UC3: Per-Sheet Statistics
- Stats panel shows aggregate: total utilization, total pieces placed, sheet count
- Each sheet label in the viewport shows its individual utilization percentage
- "Placed: 47/50 on 3 sheets" summary in stats

### UC4: Per-Sheet GLS Optimization
- After LBF places all pieces, GLS runs independently on each sheet
- Each sheet gets `total_time / sheet_count` time budget
- GLS improves within-sheet packing quality using existing optimizer unchanged
- Compression runs independently per sheet (binary search for minimum strip width)
- Progressive updates show improvements as each sheet is optimized

### UC5: Benchmark Backward Compatibility
- Loading a sparrow benchmark JSON file uses SPP mode (single strip, no sheet overflow)
- All existing benchmark results are preserved
- Benchmark mode is detected by `NestingMode::Strip` vs `NestingMode::Sheet`

### UC6: Sheet Config Changes Trigger Re-nest
- User changes sheet width/height/separation → optimizer restarts with new dimensions
- All sheets use the same dimensions (single template)

### UC7: Delete Piece Across Sheets
- User clicks a piece on any sheet → piece is selected
- Delete removes it from whichever sheet it's on → triggers re-nest

### UC8: Oversized Piece Warning
- If a piece's bounding box exceeds sheet dimensions in all orientations, it appears in `unplaced`
- Stats panel shows "N pieces too large for sheet" when applicable

## Architecture

### Crate Structure (Unchanged from Sprint 003)

```
thetta/
├── Cargo.toml                    (workspace: resolver = "2")
├── crates/
│   ├── thetta-geometry/          (2D types — unchanged)
│   ├── thetta-nesting/           (NestingEngine, LBF, GLS, MultiSheetEngine — MODIFIED)
│   ├── thetta-app/               (egui GUI — MODIFIED)
│   ├── thetta-step/              (STEP import/export — unchanged)
│   └── jagua-rs/                 (forked collision detection — unchanged)
```

### Key Design Decision: Multiple SPP Instances (Not BPP)

The jagua-rs BPP module (`crates/jagua-rs/src/probs/bpp/`) is **dead code** — the files exist on disk but `probs/mod.rs` only declares `pub mod spp;`. The BPP code has never been compiled, and its dependencies (`slotmap`, `itertools`, `rayon`) are missing from `Cargo.toml`. Enabling it would require unknown debugging effort for untested code.

Instead, we use **multiple independent SPP engines** — one `NestingEngine` per sheet. This:
- Requires zero jagua-rs fork changes
- Reuses all existing LBF, GLS, and compression code unchanged
- Is simpler to reason about (each sheet is independent)
- Maps cleanly to the existing `NestingResult { sheets: Vec<Sheet> }` type

### New Types

```rust
/// Orchestrates multiple independent SPP engines, one per sheet.
pub struct MultiSheetEngine {
    sheets: Vec<NestingEngine>,    // One engine per sheet
    pieces: Vec<Piece>,            // All pieces (shared reference data)
    sheet_config: SheetConfig,     // Template applied to each sheet
    orientations: Vec<RotationRange>,
    /// piece_idx → (sheet_idx, engine_local_idx)
    assignment: Vec<Option<(usize, usize)>>,
}

/// Determines whether to use strip packing or multi-sheet mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NestingMode {
    Strip,  // SPP: single infinite strip (benchmarks)
    Sheet,  // Multi-sheet: fixed-size sheets with overflow
}
```

### Data Flow

```
Vec<Piece> + SheetConfig + NestingMode
         |
         v
  ┌─────────────────────────────────────┐
  │ NestingMode::Strip                  │
  │   → run_nesting() (existing SPP)    │
  │                                     │
  │ NestingMode::Sheet                  │
  │   → run_nesting_multisheet()        │
  │     1. MultiSheetEngine::new()      │
  │     2. run_lbf_multisheet()         │
  │        (fill sheet 1, overflow      │
  │         to sheet 2, etc.)           │
  │     3. Per-sheet GLS                │
  │        (existing GLSOptimizer       │
  │         on each sheet's engine)     │
  │     4. Per-sheet compression        │
  │        (existing compress_strip     │
  │         on each sheet's pieces)     │
  └──────────┬──────────────────────────┘
             │
             v
  NestingResult { sheets: Vec<Sheet>, unplaced: Vec<PieceId> }
             │
             v
  ┌──────────────────┐
  │ Viewport         │
  │ (side-by-side)   │
  │ Sheet1 Sheet2 .. │
  └──────────────────┘
```

### Threading Model (Unchanged pattern)

```
Main Thread (egui)                    Background Thread
┌───────────────────┐               ┌─────────────────────────┐
│ - Renders all      │ ◀── mpsc ── │ - Owns MultiSheetEngine  │
│   sheets in        │              │   (Vec<NestingEngine>)   │
│   viewport         │              │ - Runs LBF multisheet    │
│ - Polls updates    │ ── cancel ─▶│ - Runs GLS per sheet     │
│   each frame       │  AtomicBool │ - Runs compression/sheet │
└───────────────────┘               └─────────────────────────┘
```

Same `mpsc::channel` + `Arc<AtomicBool>` pattern as existing `start_optimizer()`.

### Design Decisions

1. **Multiple SPP instances, not jagua-rs BPP.** BPP module is dead code. Multiple independent engines are simpler, proven, and require zero fork changes.

2. **Fill-and-spill LBF.** When a piece doesn't fit on the current sheet, try all existing sheets before opening a new one. This fills gaps left by earlier placements and produces fewer total sheets.

3. **Per-sheet independent GLS.** Reuses existing `GLSOptimizer` completely unchanged. Each sheet's engine is extracted, optimized, and written back. Time budget divided equally across sheets.

4. **Per-sheet independent compression.** Reuses existing `compress_strip` unchanged. Each sheet is compressed independently. This doesn't minimize total sheet count (that's a deferred cross-sheet optimization), but it tightens within-sheet packing.

5. **Side-by-side viewport.** All sheets rendered horizontally with 100mm gap in world space. Unified zoom/pan. Each sheet gets a label with utilization. Simpler than tabs, gives full overview at a glance.

6. **`NestingMode` enum for dispatch.** Benchmark loads set `NestingMode::Strip` (existing SPP path). Everything else defaults to `NestingMode::Sheet`. Explicit enum avoids fragile inference from sheet dimensions.

7. **Piece assignment via separate map.** `MultiSheetEngine.assignment` maps `piece_idx → (sheet_idx, local_idx)`. This avoids modifying the per-sheet `NestingEngine` which uses consecutive 0-based indices internally.

## Implementation

### Phase 1: MultiSheetEngine

**Goal:** Core multi-sheet orchestrator that manages multiple SPP engines.

#### Task 1.1: MultiSheetEngine struct
**File:** `crates/thetta-nesting/src/multi_sheet.rs` (NEW)

```rust
pub struct MultiSheetEngine {
    sheets: Vec<NestingEngine>,
    all_pieces: Vec<Piece>,
    sheet_config: SheetConfig,
    orientations: Vec<RotationRange>,
    /// piece_idx (global) → Some((sheet_idx, local_idx in that sheet's engine))
    assignment: Vec<Option<(usize, usize)>>,
}

impl MultiSheetEngine {
    pub fn new(pieces: &[Piece], sheet: &SheetConfig, orientations: &[RotationRange]) -> Self;

    /// Add a new sheet, returns its index
    pub fn open_sheet(&mut self) -> usize;

    /// Number of open sheets
    pub fn sheet_count(&self) -> usize;

    /// Get a specific sheet's engine (for GLS)
    pub fn sheet_engine(&self, sheet_idx: usize) -> &NestingEngine;
    pub fn sheet_engine_mut(&mut self, sheet_idx: usize) -> &mut NestingEngine;

    /// Pieces assigned to a specific sheet (global indices)
    pub fn pieces_on_sheet(&self, sheet_idx: usize) -> Vec<usize>;

    /// Try to place piece on a specific sheet
    pub fn place_on_sheet(&mut self, piece_idx: usize, sheet_idx: usize, x: f64, y: f64, rotation: f64) -> bool;

    /// Remove piece from whichever sheet it's on
    pub fn remove_piece(&mut self, piece_idx: usize);

    /// Aggregate result across all sheets
    pub fn current_result(&self) -> NestingResult;
}
```

**Key detail:** Each sheet's `NestingEngine` only knows about the pieces assigned to it. When a piece is assigned to sheet N, it's added to that engine's piece list. The `assignment` map tracks where each global piece index lives.

**Implementation approach:** Rather than pre-loading all pieces into every engine (wasteful), each sheet engine is created with only the pieces assigned to it. This means piece indices are local to each engine. The `assignment` map stores `(sheet_idx, local_idx)` for coordinate translation.

Actually, simpler approach: create each sheet engine with ALL pieces but only place assigned ones. This way local_idx == global_idx. The engine handles unplaced pieces gracefully (they're just in the `unplaced` list). This avoids index remapping entirely.

```rust
impl MultiSheetEngine {
    pub fn open_sheet(&mut self) -> usize {
        let engine = NestingEngine::new_with_orientations(
            &self.all_pieces,
            &self.sheet_config,
            Some(&self.orientations),
        );
        self.sheets.push(engine);
        self.sheets.len() - 1
    }

    pub fn place_on_sheet(&mut self, piece_idx: usize, sheet_idx: usize, x: f64, y: f64, rotation: f64) -> bool {
        if self.sheets[sheet_idx].place_item(piece_idx, x, y, rotation) {
            self.assignment[piece_idx] = Some(sheet_idx);
            true
        } else {
            false
        }
    }

    pub fn current_result(&self) -> NestingResult {
        let sheets = self.sheets.iter().map(|engine| {
            let result = engine.current_result();
            result.sheets.into_iter().next().unwrap_or_default()
        }).collect();

        let unplaced = self.all_pieces.iter().enumerate()
            .filter(|(idx, _)| self.assignment[*idx].is_none())
            .map(|(_, p)| p.id)
            .collect();

        NestingResult { sheets, unplaced }
    }
}
```

Wait — this means every sheet engine has ALL pieces registered in its jagua-rs instance, each with demand=1. If piece 5 is placed on sheet 0, sheet 1's engine still has demand for piece 5. This works correctly because we only call `place_item` on the assigned sheet, and `current_result` per sheet only returns pieces actually placed on that sheet.

But jagua-rs demand tracking: `SPProblem::place_item` calls `register_included_item` which decrements `item_demand_qtys[item_id]`. If we place piece 5 on sheet 0, sheet 0's demand for piece 5 goes to 0. Sheet 1 still has demand=1 for piece 5, which is fine — we just never place it there. No conflict.

**Deliverable:** `MultiSheetEngine` struct with open_sheet, place_on_sheet, remove_piece, current_result.

#### Task 1.2: Register module
**File:** `crates/thetta-nesting/src/lib.rs` (MODIFY)
- Add `pub mod multi_sheet;`

### Phase 2: Multi-Sheet LBF

**Goal:** Fill-and-spill LBF that overflows to additional sheets.

#### Task 2.1: Multi-sheet LBF function
**File:** `crates/thetta-nesting/src/lbf.rs` (MODIFY — add new function)

```rust
pub fn run_lbf_multisheet(
    engine: &mut MultiSheetEngine,
    config: LbfConfig,
    cancel: Option<&Arc<AtomicBool>>,
    mut on_progress: impl FnMut(&MultiSheetEngine),
) -> NestingResult {
    // Sort pieces by area (descending) — same as existing LBF
    let mut order = (0..engine.piece_count()).collect::<Vec<_>>();
    order.sort_by(|a, b| engine.piece_area(*b).partial_cmp(&engine.piece_area(*a)).unwrap_or(Ordering::Equal));

    // Ensure at least one sheet exists
    if engine.sheet_count() == 0 {
        engine.open_sheet();
    }

    for piece_idx in order {
        if is_cancelled() { break; }

        let rotations = /* same rotation sampling as existing LBF */;

        // Try all existing sheets (fill-and-spill)
        let mut placed = false;
        for sheet_idx in 0..engine.sheet_count() {
            if let Some((x, y, r)) = find_lbf_position(
                engine.sheet_engine(sheet_idx),
                piece_idx,
                &rotations,
                &config,
            ) {
                engine.place_on_sheet(piece_idx, sheet_idx, x, y, r);
                placed = true;
                break;
            }
        }

        // If no existing sheet fits, open a new one
        if !placed {
            let new_idx = engine.open_sheet();
            if let Some((x, y, r)) = find_lbf_position(
                engine.sheet_engine(new_idx),
                piece_idx,
                &rotations,
                &config,
            ) {
                engine.place_on_sheet(piece_idx, new_idx, x, y, r);
                placed = true;
            }
            // If still not placed: piece too large for sheet → stays in unplaced
        }

        if placed {
            on_progress(engine);
        }
    }

    // Fit strip on each sheet
    for sheet_idx in 0..engine.sheet_count() {
        engine.sheet_engine_mut(sheet_idx).fit_strip();
    }

    engine.current_result()
}
```

#### Task 2.2: Extract `find_lbf_position` helper
**File:** `crates/thetta-nesting/src/lbf.rs` (MODIFY — refactor)

Extract the existing grid-search logic from `run_lbf` into a reusable helper:

```rust
/// Find the best LBF position for a piece on a single engine.
/// Returns Some((x, y, rotation)) or None if no valid position found.
fn find_lbf_position(
    engine: &NestingEngine,
    piece_idx: usize,
    rotations: &[f64],
    config: &LbfConfig,
) -> Option<(f64, f64, f64)> {
    // Grid search with strip expansion — same logic as current run_lbf inner loop
}
```

The existing `run_lbf` is refactored to call `find_lbf_position` internally. This avoids code duplication.

**Note on strip expansion for fixed sheets:** The current LBF expands strip width by 15% when a piece doesn't fit. For multi-sheet mode, the sheet width is fixed. So `find_lbf_position` should NOT expand the strip — if the piece doesn't fit at the current width, it returns None and the caller tries the next sheet. Add a `fixed_width: bool` parameter to control this behavior.

### Phase 3: Multi-Sheet GLS + Compression

**Goal:** Run existing GLS and compression independently per sheet.

#### Task 3.1: Per-sheet GLS in runner
**File:** `crates/thetta-nesting/src/runner.rs` (MODIFY — add new function)

```rust
pub fn run_nesting_multisheet(
    pieces: Vec<Piece>,
    sheet: SheetConfig,
    orientations: Option<Vec<RotationRange>>,
    config: OptimizerConfig,
    tx: mpsc::Sender<NestingUpdate>,
    cancel: Arc<AtomicBool>,
) {
    let ors = orientations.as_deref()
        .unwrap_or(&default_orientations(pieces.len()));
    let mut ms_engine = MultiSheetEngine::new(&pieces, &sheet, ors);

    // Phase 1: Multi-sheet LBF
    let tx_lbf = tx.clone();
    let baseline = run_lbf_multisheet(
        &mut ms_engine,
        LbfConfig { orientations: orientations.clone(), ..LbfConfig::default() },
        Some(&cancel),
        |eng| { let _ = tx_lbf.send(NestingUpdate::Improved(eng.current_result())); },
    );
    let _ = tx.send(NestingUpdate::Improved(baseline.clone()));

    if cancel.load(Ordering::Relaxed) {
        let _ = tx.send(NestingUpdate::Finished(baseline));
        return;
    }

    // Phase 2: Per-sheet GLS
    let time_per_sheet = Duration::from_secs(60) / ms_engine.sheet_count().max(1) as u32;
    let mut best_result = baseline;

    for sheet_idx in 0..ms_engine.sheet_count() {
        if cancel.load(Ordering::Relaxed) { break; }

        let sheet_pieces = ms_engine.pieces_on_sheet(sheet_idx);
        if sheet_pieces.is_empty() { continue; }

        // Extract pieces for this sheet and run GLS on a temporary engine
        let sub_pieces: Vec<Piece> = sheet_pieces.iter()
            .map(|&idx| pieces[idx].clone())
            .collect();
        let sub_ors: Vec<RotationRange> = sheet_pieces.iter()
            .map(|&idx| ors[idx].clone())
            .collect();

        let mut sub_engine = NestingEngine::new_with_orientations(
            &sub_pieces, &sheet, Some(&sub_ors),
        );

        // Seed sub-engine with current placements from LBF
        for (local_idx, &global_idx) in sheet_pieces.iter().enumerate() {
            if let Some(placement) = ms_engine.sheet_engine(sheet_idx).placement_of(global_idx) {
                sub_engine.force_place_item(local_idx, placement.x, placement.y, placement.rotation);
            }
        }

        let tx2 = tx.clone();
        let mut gls = GLSOptimizer::new(sub_engine, config.clone());
        gls.run(
            |_r| { /* GLS improved within this sheet */ },
            move |iter, overlap, best, feasible| {
                let _ = tx2.send(NestingUpdate::Progress {
                    iteration: iter, total_overlap: overlap,
                    best_overlap: best, feasible_count: feasible,
                });
            },
            time_per_sheet,
            Some(&cancel),
        );

        // Write back GLS improvements to MultiSheetEngine
        if gls.best_feasible.is_some() {
            for (local_idx, &global_idx) in sheet_pieces.iter().enumerate() {
                if let Some(p) = gls.engine.placement_of(local_idx) {
                    ms_engine.remove_piece(global_idx);
                    ms_engine.place_on_sheet(global_idx, sheet_idx, p.x, p.y, p.rotation);
                }
            }
        }

        best_result = ms_engine.current_result();
        let _ = tx.send(NestingUpdate::Improved(best_result.clone()));
    }

    if cancel.load(Ordering::Relaxed) {
        let _ = tx.send(NestingUpdate::Finished(best_result));
        return;
    }

    // Phase 3: Per-sheet compression
    for sheet_idx in 0..ms_engine.sheet_count() {
        if cancel.load(Ordering::Relaxed) { break; }

        let sheet_pieces = ms_engine.pieces_on_sheet(sheet_idx);
        if sheet_pieces.is_empty() { continue; }

        let sub_pieces: Vec<Piece> = sheet_pieces.iter()
            .map(|&idx| pieces[idx].clone())
            .collect();
        let sub_ors: Option<Vec<RotationRange>> = Some(
            sheet_pieces.iter().map(|&idx| ors[idx].clone()).collect()
        );

        let compressed = compress_strip(
            &sub_pieces, sheet.clone(), config.compression_step,
            sub_ors.as_deref(), Some(&cancel),
        );

        // Write back compressed placements
        if compressed.unplaced.is_empty() {
            // Clear and re-place with compressed positions
            for &global_idx in &sheet_pieces {
                ms_engine.remove_piece(global_idx);
            }
            if let Some(comp_sheet) = compressed.sheets.first() {
                for (local_idx, &global_idx) in sheet_pieces.iter().enumerate() {
                    if let Some(p) = comp_sheet.placements.iter()
                        .find(|p| p.piece_id == sub_pieces[local_idx].id)
                    {
                        ms_engine.place_on_sheet(global_idx, sheet_idx, p.x, p.y, p.rotation);
                    }
                }
            }
        }

        best_result = ms_engine.current_result();
        let _ = tx.send(NestingUpdate::Improved(best_result.clone()));
    }

    let _ = tx.send(NestingUpdate::Finished(best_result));
}
```

### Phase 4: NestingMode Dispatch

**Goal:** Route optimizer to SPP or multi-sheet based on context.

#### Task 4.1: NestingMode enum
**File:** `crates/thetta-nesting/src/runner.rs` (MODIFY)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NestingMode {
    Strip,  // SPP: existing single-strip path
    Sheet,  // Multi-sheet with overflow
}
```

#### Task 4.2: Dispatch in optimizer
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
    match mode {
        NestingMode::Strip => run_nesting(pieces, sheet_config, orientations, config, tx, cancel),
        NestingMode::Sheet => run_nesting_multisheet(pieces, sheet_config, orientations, config, tx, cancel),
    }
}
```

#### Task 4.3: Wire mode through AppState
**File:** `crates/thetta-app/src/state.rs` (MODIFY)

Add `pub nesting_mode: NestingMode` to `AppState`. Default: `NestingMode::Sheet`. `load_benchmark_file` sets `NestingMode::Strip`. Pass mode through `start_optimizer()` → `run_optimizer()`.

### Phase 5: Viewport Multi-Sheet Rendering

**Goal:** Render all sheets side-by-side with labels.

#### Task 5.1: Side-by-side viewport layout
**File:** `crates/thetta-app/src/viewport.rs` (MODIFY)

Replace the single-sheet rendering with a loop over all sheets:

```rust
pub fn show_viewport(ui: &mut egui::Ui, state: &mut ViewportState, app: &AppState) -> Option<PieceId> {
    // ... existing setup (allocate_painter, zoom, pan) ...

    let sheet_count = app.current_result.as_ref()
        .map(|r| r.sheets.len().max(1))
        .unwrap_or(1);
    let gap = 100.0; // mm gap between sheets

    // Compute total world width for all sheets
    let total_width = sheet_count as f32 * sheet_w + (sheet_count - 1).max(0) as f32 * gap as f32;

    // Adjust world rect to encompass all sheets
    // Center on the midpoint of all sheets
    let center_x = total_width * 0.5;
    let center = Pos2::new(center_x, sheet_h * 0.5) + state.pan;
    // ... (recompute world rect and to_screen with total_width) ...

    for sheet_idx in 0..sheet_count {
        let x_offset = sheet_idx as f32 * (sheet_w + gap as f32);

        // Draw sheet boundary
        painter.rect_stroke(
            to_screen.transform_rect(Rect::from_min_max(
                Pos2::new(x_offset, 0.0),
                Pos2::new(x_offset + sheet_w, sheet_h),
            )),
            0.0,
            Stroke::new(2.0, Color32::WHITE),
            egui::StrokeKind::Outside,
        );

        // Draw sheet label with utilization
        if let Some(result) = &app.current_result {
            if let Some(sheet) = result.sheets.get(sheet_idx) {
                let label = format!("Sheet {} ({:.0}%)", sheet_idx + 1, sheet.utilization * 100.0);
                let label_pos = to_screen.transform_pos(Pos2::new(x_offset, -30.0));
                painter.text(label_pos, egui::Align2::LEFT_BOTTOM, label,
                    egui::FontId::proportional(14.0), Color32::WHITE);

                // Draw used-width indicator
                let used_x = x_offset + sheet.strip_length.clamp(0.0, app.sheet_config.width) as f32;
                let a = to_screen.transform_pos(Pos2::new(used_x, 0.0));
                let b = to_screen.transform_pos(Pos2::new(used_x, sheet_h));
                painter.line_segment([a, b], Stroke::new(2.0, Color32::from_rgb(255, 180, 0)));

                // Draw pieces on this sheet
                for placement in &sheet.placements {
                    // Same piece rendering as before, but offset by x_offset
                    let adjusted = Placement {
                        x: placement.x + x_offset as f64,
                        ..placement.clone()
                    };
                    // ... render piece at adjusted position ...
                }
            }
        }
    }
}
```

**Key change:** Pieces are rendered at `(placement.x + sheet_offset, placement.y)`. Click-to-select must reverse this offset to identify which sheet a click lands on.

#### Task 5.2: Auto-fit zoom for multi-sheet
Adjust the initial zoom calculation to fit all sheets. Replace `sheet_w` with `total_width` in the scale computation so the view starts showing all sheets.

### Phase 6: Stats + Panel Updates

**Goal:** Aggregate statistics across sheets and show sheet count.

#### Task 6.1: Aggregate stats in apply_result
**File:** `crates/thetta-app/src/state.rs` (MODIFY)

Update `apply_result` (or wherever stats are computed from `NestingResult`) to aggregate across all sheets:

```rust
fn apply_result(&mut self, result: &NestingResult) {
    self.stats.pieces_placed = result.sheets.iter()
        .map(|s| s.placements.len())
        .sum();
    self.stats.pieces_total = self.pieces.len();

    // Aggregate utilization: total piece area / total sheet area used
    let total_sheet_area: f64 = result.sheets.iter()
        .map(|s| s.strip_length * self.sheet_config.height)
        .sum();
    let total_piece_area: f64 = result.sheets.iter()
        .flat_map(|s| &s.placements)
        .filter_map(|p| self.pieces.iter().find(|piece| piece.id == p.piece_id))
        .map(|piece| polygon_ops::area(&piece.polygon))
        .sum();
    self.stats.utilization_pct = if total_sheet_area > 0.0 {
        (total_piece_area / total_sheet_area) * 100.0
    } else {
        0.0
    };

    self.stats.strip_length = result.sheets.iter()
        .map(|s| s.strip_length)
        .fold(0.0_f64, f64::max); // max strip length across sheets

    self.stats.sheet_count = result.sheets.len();
}
```

#### Task 6.2: Show sheet count in stats panel
**File:** `crates/thetta-app/src/panels.rs` (MODIFY)

Add to the Stats section:
```rust
if app.stats.sheet_count > 1 {
    ui.label(format!("Sheets: {}", app.stats.sheet_count));
}
```

#### Task 6.3: Add sheet_count to NestingStats
**File:** `crates/thetta-app/src/state.rs` (MODIFY)

Add `pub sheet_count: usize` to `NestingStats`.

### Phase 7: Testing

#### Task 7.1: MultiSheetEngine unit tests
**File:** `crates/thetta-nesting/tests/multi_sheet.rs` (NEW)

- Create engine with 2 large rectangles that don't fit on one sheet → verify 2 sheets, 1 piece each
- Create engine with 4 small rectangles that all fit → verify 1 sheet, 4 pieces
- Place and remove piece → verify assignment map updates
- `current_result()` returns correct sheet count and piece IDs per sheet
- Oversized piece (wider than sheet) → stays in unplaced

#### Task 7.2: Multi-sheet LBF integration test
**File:** `crates/thetta-nesting/tests/multi_sheet.rs` (EXTEND)

- 10 rectangles sized to fill ~2 sheets → LBF produces 2 sheets, all placed, no overlaps
- Verify fill-and-spill: pieces placed on sheet 1 first, overflow to sheet 2
- Verify `strip_length` per sheet is ≤ sheet width

#### Task 7.3: NestingMode dispatch test
**File:** `crates/thetta-nesting/tests/multi_sheet.rs` (EXTEND)

- `NestingMode::Strip` → single sheet in result (same as before)
- `NestingMode::Sheet` with many pieces → multiple sheets in result

#### Task 7.4: Benchmark regression test
**File:** `crates/thetta-nesting/tests/benchmark_smoke.rs` (MODIFY)

- Verify jakobs1 still produces valid single-sheet SPP results
- Strip length unchanged (within tolerance) from pre-Sprint-004 baseline

#### Task 7.5: Build verification
```sh
cargo build --release
cargo test --workspace
cargo test -p thetta-nesting
cargo test -p thetta-step --features occt
```

## Files Summary

### New Files

| File | Purpose | Est. LOC |
|------|---------|----------|
| `crates/thetta-nesting/src/multi_sheet.rs` | MultiSheetEngine orchestrator | 200 |
| `crates/thetta-nesting/tests/multi_sheet.rs` | Multi-sheet unit + integration tests | 150 |
| **Total new** | | **~350** |

### Modified Files

| File | Changes | Est. Delta |
|------|---------|------------|
| `crates/thetta-nesting/src/lib.rs` | Add `pub mod multi_sheet;` | +1 |
| `crates/thetta-nesting/src/lbf.rs` | Extract `find_lbf_position` helper, add `run_lbf_multisheet` | +80 |
| `crates/thetta-nesting/src/runner.rs` | Add `NestingMode`, `run_nesting_multisheet` | +120 |
| `crates/thetta-nesting/src/optimizer.rs` | Add mode dispatch | +10 |
| `crates/thetta-app/src/state.rs` | Add `nesting_mode`, `sheet_count` to stats, aggregate stats, pass mode through | +40 |
| `crates/thetta-app/src/viewport.rs` | Multi-sheet side-by-side rendering, adjusted zoom, click offset | +80 |
| `crates/thetta-app/src/panels.rs` | Sheet count display in stats | +5 |
| `crates/thetta-nesting/tests/benchmark_smoke.rs` | Regression assertion | +5 |
| **Total modified** | | **~340 delta** |

### Estimated Total Effort: ~690 LOC (new + delta)

Smaller than previous sprints (~1300 LOC each) because the architecture reuses all existing LBF, GLS, and compression code unchanged. The main new code is the `MultiSheetEngine` orchestrator and the multi-sheet runner function.

## Definition of Done

### P0 — Must Ship
- [ ] `MultiSheetEngine` creates and manages multiple independent SPP engines
- [ ] Multi-sheet LBF places pieces across multiple sheets with fill-and-spill strategy
- [ ] Pieces that don't fit on one 2440×1220mm sheet overflow to additional sheets
- [ ] All pieces placed on benchmark instances (existing SPP path unchanged)
- [ ] `NestingMode::Strip` produces identical results to pre-Sprint-004 (benchmark regression)
- [ ] `NestingMode::Sheet` is default for non-benchmark workflows
- [ ] Viewport renders all sheets side-by-side with labels and utilization
- [ ] Click-to-select works across all sheets
- [ ] Stats show aggregate: total utilization, pieces placed, sheet count
- [ ] Per-sheet GLS optimization runs with existing optimizer unchanged
- [ ] Per-sheet compression runs with existing compress_strip unchanged
- [ ] No panics or crashes on normal usage
- [ ] All existing tests pass (`cargo test --workspace`)
- [ ] Integration test: >20 pieces across 3+ sheets, all placed, no overlaps

### P1 — Should Ship
- [ ] Oversized pieces (exceed sheet dimensions) reported in unplaced with reason
- [ ] Per-sheet utilization labels visible in viewport
- [ ] Used-width indicator line drawn per sheet
- [ ] Auto-fit zoom adjusts to show all sheets on initial load
- [ ] Empty sheets removed from result (no phantom sheets)
- [ ] Zoom/pan works smoothly across multi-sheet viewport

### P2 — Stretch
- [ ] Cross-sheet piece migration in GLS (move pieces between sheets to balance utilization)
- [ ] Sheet-count minimization (try to eliminate last sheet by redistributing)
- [ ] Per-sheet export selection ("Export sheet 2 only")
- [ ] Sheet overview mini-map in side panel

### Cut if Behind Schedule
- Per-sheet compression (Phase 3 step 3) — strip_length indicator still works, just won't be tightened
- Oversized piece detection (P1) — pieces just appear in `unplaced` without explanation
- Auto-fit zoom (P1) — user can manually zoom out

## Risks

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| All-pieces-on-all-engines memory overhead: each sheet engine creates a full SPP instance with all pieces | Medium | Medium | For N pieces and M sheets, this is O(N*M) jagua-rs items. Fine for <100 pieces and <10 sheets. If problematic, switch to piece-subset engines (adds index remapping complexity). |
| Per-sheet GLS seeding: extracting placements from MultiSheetEngine and writing back may cause coordinate drift | Medium | Medium | Use exact `placement_of()` values. Integration test verifies round-trip placement fidelity. |
| Per-sheet GLS time budget: with many sheets, each gets very little time | Low | Medium | Minimum 10s per sheet, cap total GLS time at 5 minutes. User can restart for more time. |
| Per-sheet compression rebuilds engine from scratch, losing GLS improvements | Medium | High | Accept this trade-off for Sprint 004. Compression uses LBF which is deterministic. GLS improvements are already captured in the `Improved` result before compression runs. |
| Fill-and-spill LBF opens many sheets for irregularly-shaped pieces | Low | Medium | The fill-and-spill tries all existing sheets before opening new ones. Worst case: one piece per sheet for extremely large pieces. |
| Viewport performance with many sheets (>10 sheets rendering simultaneously) | Low | Low | egui handles this fine for polygon rendering. Consider culling off-screen sheets if >20. |
| `find_lbf_position` with `fixed_width=true` may fail to place pieces that would fit with slight expansion | Medium | Medium | Sheet width is already generous (2440mm). If this becomes an issue, allow minimal expansion (1-2%) before giving up. |
| Piece deletion from a specific sheet requires knowing which sheet it's on | Low | Low | `assignment` map provides direct lookup. Click-to-select already identifies the piece ID; removal goes through `MultiSheetEngine.remove_piece`. |

## Security

- No new network access. Fully local application.
- No new dependencies. All new code is pure Rust using existing crate APIs.
- No changes to jagua-rs fork.
- No new file I/O paths. Existing STEP import/export handles multi-sheet via `NestingResult.sheets` (already implemented in Sprint 003).
- No new unsafe code. `MultiSheetEngine` is safe Rust wrapping safe `NestingEngine` API.

## Dependencies

### No New Dependencies

This sprint adds no new Rust crates or system dependencies. All functionality is built on:

| Existing Crate | Usage |
|----------------|-------|
| `thetta-nesting` (NestingEngine) | One instance per sheet |
| `jagua-rs` (SPProblem, CDEngine) | Unchanged, used through NestingEngine |
| `eframe`/`egui` | Viewport rendering (unchanged API) |

### No jagua-rs Fork Changes

The multi-SPP approach deliberately avoids touching jagua-rs. The dead BPP module remains unused.

## Open Questions

1. **Memory for all-pieces-on-all-engines:** Each sheet engine creates a full `SPInstance` with all pieces. For 100 pieces across 5 sheets, that's 500 jagua-rs items (each with collision geometry). Is this a real concern? Likely fine for typical sheet-metal jobs (<50 parts, <5 sheets). Monitor and optimize if needed.

2. **Per-sheet compression vs global:** Currently each sheet is compressed independently. A global optimization would try to remove the last sheet entirely by redistributing its pieces. This requires cross-sheet logic (deferred). Worth doing in Sprint 005?

3. **GLS seeding fidelity:** When extracting placements from `MultiSheetEngine` and re-creating sub-engines for GLS, does the sub-engine's collision geometry perfectly match the original? Both use the same `pieces_to_sp_instance` with the same parameters, so they should. Verify with integration test.

4. **Auto-zoom computation:** With multiple sheets, the initial zoom should fit all sheets. The current zoom logic centers on `(sheet_w/2, sheet_h/2)`. For multi-sheet, center on `(total_width/2, sheet_h/2)` and scale to fit `total_width × sheet_h`.

5. **Piece coloring across sheets:** The existing `piece_color()` method colors by source file. This works across sheets automatically — same piece on any sheet gets the same color. No changes needed.
