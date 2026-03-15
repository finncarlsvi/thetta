# Sprint 004: Multi-Sheet Bin Packing + Multiple STEP Files

## Overview

Implement multi-sheet bin packing so that pieces overflow onto additional fixed-size sheets, and support loading multiple STEP files with pieces accumulating across files. This sprint bridges the gap between the current single-strip SPP engine and the product vision in IDEA.md.

**Scope:** Dedicated BPP engine struct (not dual-mode enum), fill-and-spill LBF strategy, cross-sheet GLS piece swapping, sheet-elimination compression, tabbed viewport with per-sheet navigation, multi-file piece accumulation with globally unique IDs, stats aggregation across sheets.

**Prerequisites:** Sprint 003 complete. The nesting engine wraps jagua-rs `SPProblem` and produces `NestingResult` with `Vec<Sheet>`. The app has STEP import/export, native file dialogs, and background optimizer with cancel token. jagua-rs BPP types (`BPInstance`, `BPProblem`, `BPPlacement`, `BPLayoutType`, `LayKey`) exist but are unused.

**Deferred:** Per-sheet piece drag reordering, heterogeneous sheet sizes (different bin templates), BPP-aware benchmark mode, manual sheet assignment UI.

## Use Cases

### UC1: Automatic Multi-Sheet Overflow

- User loads a STEP file containing more pieces than fit on a single 2440x1220mm sheet
- The optimizer automatically places pieces across multiple sheets
- Each sheet is filled greedily before opening a new one (fill-and-spill)
- The nesting result contains N sheets, each with its own placements and utilization
- Status bar shows "Placed 47/47 on 3 sheets"

### UC2: Tabbed Sheet Navigation in Viewport

- After nesting, the viewport shows the currently selected sheet tab
- A tab bar at the top of the viewport shows "Sheet 1 | Sheet 2 | Sheet 3" with utilization percentages
- Clicking a tab switches the viewport to that sheet
- The active sheet renders exactly as the current single-sheet viewport does (zoom, pan, click-to-select all work)
- An "All Sheets" tab renders a minimap overview with sheets side-by-side (read-only, no click selection)

### UC3: Per-Sheet and Aggregate Statistics

- The Stats panel shows aggregate stats: total pieces placed, total sheets used, average utilization
- Below the aggregate, a collapsible "Per Sheet" section lists each sheet with its utilization, piece count, and strip length
- When a specific sheet tab is active, the main stats line reflects that sheet's numbers

### UC4: Cross-Sheet GLS Optimization

- After the initial fill-and-spill LBF placement, GLS runs across all sheets
- GLS can move a piece from a crowded sheet to a less-full sheet if it reduces total overlap or improves utilization balance
- If a sheet becomes empty after piece moves, it is eliminated
- The compression phase attempts to eliminate the last sheet by redistributing its pieces to earlier sheets

### UC5: Load Multiple STEP Files

- User clicks "Load STEP" multiple times, each time selecting a different file
- Pieces from each file accumulate (existing pieces are preserved)
- Each file's pieces get globally unique `PieceId` values (offset-based allocation)
- File colors in the viewport distinguish pieces by source file (existing `file_colors` mechanism)
- The thickness selector applies per-file: each loaded file can have a different active thickness
- Piece list in the side panel groups pieces by source file

### UC6: Benchmark Mode Backward Compatibility

- Loading a sparrow JSON benchmark file still uses the SPP engine (strip packing)
- The `NestingEngine` (SPP) struct is untouched; a new `BinPackEngine` struct handles BPP
- All existing benchmark tests and integration tests pass without modification
- The runner detects which engine to use based on whether sheet dimensions are finite

### UC7: STEP Export with Multiple Sheets

- STEP export already iterates `result.sheets` and writes one file per sheet
- With multi-sheet results, this produces `basename_sheet_001.step`, `basename_sheet_002.step`, etc.
- Export status shows "Exported 3 sheets" with per-sheet piece counts

## Architecture

### Crate Structure (Post-Sprint 004)

```
thetta/
+-- crates/
|   +-- thetta-geometry/          (unchanged -- NestingResult already has Vec<Sheet>)
|   +-- thetta-nesting/
|   |   +-- src/
|   |   |   +-- engine.rs         (UNCHANGED -- SPP engine for benchmarks)
|   |   |   +-- bin_pack.rs       (NEW -- BinPackEngine wrapping BPProblem)
|   |   |   +-- lbf.rs            (MODIFIED -- add run_lbf_bp for multi-sheet)
|   |   |   +-- gls.rs            (MODIFIED -- add cross-sheet variant)
|   |   |   +-- compression.rs    (MODIFIED -- add sheet elimination)
|   |   |   +-- conversion.rs     (MODIFIED -- add pieces_to_bp_instance)
|   |   |   +-- runner.rs         (MODIFIED -- add run_nesting_bp pipeline)
|   |   |   +-- optimizer.rs      (MODIFIED -- dispatch to SPP or BPP)
|   +-- thetta-app/
|   |   +-- src/
|   |   |   +-- state.rs          (MODIFIED -- multi-file, active sheet, BPP dispatch)
|   |   |   +-- viewport.rs       (MODIFIED -- tab bar, per-sheet rendering)
|   |   |   +-- panels.rs         (MODIFIED -- per-sheet stats, file grouping)
|   +-- thetta-step/              (unchanged)
|   +-- jagua-rs/                 (unchanged -- BPP types already exist)
```

### Dependency Graph (Sprint 004 Additions)

```
thetta-app
+-- thetta-nesting
|   +-- thetta-geometry
|   +-- jagua-rs (features = ["spp", "bpp"])   <-- ADD "bpp" feature
+-- thetta-step
|   +-- thetta-geometry
+-- thetta-geometry
+-- jagua-rs (features = ["spp", "bpp"])
+-- eframe / egui
+-- rfd
```

Note: jagua-rs may need the `bpp` feature enabled. Check `Cargo.toml` feature gates. If BPP types are behind a feature, add it. If they are always compiled (no feature gate), no change needed.

### Data Flow

```
Vec<Piece> + SheetConfig (finite width+height)
        |
        v
+----------------------------+
| pieces_to_bp_instance()    |   Creates BPInstance with one Bin template
| (conversion.rs)            |   stock = ceil(total_area / sheet_area) + 2
+-------------+--------------+
              |
              v
+----------------------------+
| BinPackEngine::new()       |   Wraps BPProblem (SlotMap<LayKey, Layout>)
| (bin_pack.rs)              |   No layouts open initially
+-------------+--------------+
              |
              v
+----------------------------+
| run_lbf_bp()               |   Fill-and-spill:
| (lbf.rs)                   |   1. Sort pieces by area (desc)
|                            |   2. For each piece, try all open sheets
|                            |   3. If no fit, open new sheet via BPP
|                            |   4. Place using grid+rotation search
+-------------+--------------+
              |
              v
+----------------------------+
| GLSBinPackOptimizer::run() |   Cross-sheet optimization:
| (gls.rs)                   |   1. Pick highest-penalty piece (any sheet)
|                            |   2. Sample positions across ALL sheets
|                            |   3. Move to best sheet+position
|                            |   4. Empty sheets auto-close
+-------------+--------------+
              |
              v
+----------------------------+
| compress_sheets()          |   Sheet elimination:
| (compression.rs)           |   1. Sort sheets by utilization (asc)
|                            |   2. For last sheet's pieces, try LBF into
|                            |      earlier sheets only (no new bins)
|                            |   3. If all fit, eliminate last sheet
|                            |   4. Repeat until elimination fails
+-------------+--------------+
              |
              v
NestingResult { sheets: Vec<Sheet>, unplaced: Vec<PieceId> }
```

### New Types

```rust
// crates/thetta-nesting/src/bin_pack.rs

/// A dedicated BPP engine, separate from NestingEngine (SPP).
/// This is NOT an enum variant -- it's a standalone struct.
/// Rationale: the SPP engine manages a single Layout with strip-width
/// semantics (fit_strip, change_strip_width) that make no sense for BPP.
/// Sharing these behind an enum would require dead branches everywhere.
#[derive(Clone)]
pub struct BinPackEngine {
    pieces: Vec<Piece>,
    sheet: SheetConfig,
    pub instance: BPInstance,
    pub problem: BPProblem,
    /// Maps (piece_index) -> Option<(LayKey, PItemKey)>
    pk_map: Vec<Option<(LayKey, PItemKey)>>,
    orientations: Vec<RotationRange>,
}

impl BinPackEngine {
    pub fn new(
        pieces: &[Piece],
        sheet: &SheetConfig,
        orientations: Option<&[RotationRange]>,
    ) -> Self;

    pub fn piece_count(&self) -> usize;
    pub fn piece_area(&self, idx: usize) -> f64;
    pub fn sheet_config(&self) -> &SheetConfig;
    pub fn rotation_range(&self, idx: usize) -> Option<&RotationRange>;

    /// Number of currently open layouts (sheets)
    pub fn sheet_count(&self) -> usize;

    /// Iterator over open layout keys
    pub fn sheet_keys(&self) -> Vec<LayKey>;

    /// Which sheet a piece is placed on (None if unplaced)
    pub fn piece_sheet(&self, idx: usize) -> Option<LayKey>;

    /// Place item on a specific sheet. Returns false if collision.
    pub fn place_item(
        &mut self,
        idx: usize,
        sheet_key: LayKey,
        x: f64, y: f64, rotation: f64,
    ) -> bool;

    /// Open a new sheet and place item there. Returns the new LayKey.
    pub fn place_item_new_sheet(
        &mut self,
        idx: usize,
        x: f64, y: f64, rotation: f64,
    ) -> Option<LayKey>;

    /// Force-place (allows overlaps, for GLS)
    pub fn force_place_item(
        &mut self,
        idx: usize,
        sheet_key: LayKey,
        x: f64, y: f64, rotation: f64,
    );

    pub fn remove_item(&mut self, idx: usize);

    /// Collision cost on a specific sheet
    pub fn collision_cost(
        &self,
        idx: usize,
        sheet_key: LayKey,
        x: f64, y: f64, rotation: f64,
    ) -> f64;

    /// Placement of a piece (includes which sheet)
    pub fn placement_of(&self, idx: usize) -> Option<(LayKey, Placement)>;

    /// Build NestingResult from current state
    pub fn current_result(&self) -> NestingResult;

    /// Remove a sheet if it's empty
    pub fn remove_empty_sheets(&mut self);

    /// Per-sheet utilization
    pub fn sheet_utilization(&self, key: LayKey) -> f64;
}
```

```rust
// Addition to crates/thetta-nesting/src/conversion.rs

/// Create a BPInstance from pieces and a finite sheet config.
/// The bin is a rectangle of sheet.width x sheet.height.
/// Stock is set generously: ceil(total_piece_area / sheet_area) + 2.
pub fn pieces_to_bp_instance(
    pieces: &[Piece],
    sheet: &SheetConfig,
    orientations: Option<&[Vec<f64>]>,
) -> Result<BPInstance>;
```

```rust
// Additions to crates/thetta-app/src/state.rs

pub struct AppState {
    // ... existing fields ...

    /// Which sheet tab is active in the viewport (0-indexed)
    pub active_sheet_idx: usize,

    /// Global piece ID counter for uniqueness across files
    pub next_piece_id: u64,
}
```

```rust
// Additions to crates/thetta-app/src/viewport.rs

/// Renders the tab bar above the viewport and returns the selected sheet index.
fn show_sheet_tabs(
    ui: &mut egui::Ui,
    result: &NestingResult,
    active: usize,
) -> usize;
```

### Design Decisions

1. **Separate `BinPackEngine` struct, not dual-mode enum.** The intent document leans toward an enum variant. I disagree. The SPP engine has `strip_width`, `fit_strip`, `change_strip_width` -- none of which apply to BPP. The BPP engine has `sheet_keys`, `place_item_new_sheet`, `remove_empty_sheets` -- none of which apply to SPP. An enum would require dead arms in every method or trait objects with runtime dispatch. Two clean structs with a shared `NestingResult` output type is simpler and more maintainable. The runner dispatches at the top level.

2. **Fill-and-spill LBF, not BPP-native.** The intent asks "should we try all existing sheets before opening new ones?" Yes, absolutely. The strategy: for each piece (sorted by area desc), iterate all open sheets and find the best LBF position across all of them. If no sheet has a collision-free position, open a new sheet via `BPProblem::place_item(BPPlacement { layout_id: BPLayoutType::Closed { bin_id: 0 }, ... })`. This is strictly better than always placing on the last sheet because it fills gaps in earlier sheets.

3. **Cross-sheet GLS.** The intent asks whether GLS should optimize each sheet independently or move pieces between sheets. Cross-sheet movement is essential for sheet elimination. The GLS penalty selection already picks the globally worst piece -- we just sample positions across all open sheets instead of one. The modification to the existing GLS loop is minimal: instead of `uniform_sample(&sheet)`, we do `uniform_sample_any_sheet(&sheets)` which returns `(LayKey, Sample)`.

4. **Sheet-elimination compression replaces strip compression.** For BPP mode, `compress_strip` makes no sense. Instead, `compress_sheets` tries to eliminate the last (least-utilized) sheet by re-running LBF on its pieces against only the existing earlier sheets with no new bin opening allowed. If all pieces from the last sheet fit, we remove it. Repeat until elimination fails. This directly minimizes sheet count.

5. **Tabbed viewport, not side-by-side.** Side-by-side was the intent's lean. I disagree for usability: with 5+ sheets, side-by-side becomes unusably small. A tab bar costs one line of vertical space and gives full-resolution view of each sheet. An "All Sheets" minimap tab provides the overview when needed.

6. **Globally unique PieceIds via offset counter.** The intent notes potential ID conflicts between JSON and STEP pieces. The fix: `AppState` maintains a `next_piece_id: u64` counter. When loading any file (JSON or STEP), incoming piece IDs are remapped to `PieceId(next_piece_id + i)`. The `StepPieceRef` mapping uses these remapped IDs. This guarantees uniqueness across any number of loaded files.

7. **Per-file thickness tracking.** When multiple STEP files are loaded, each may have different thicknesses. The current `selected_thickness_idx` is per-session. For multi-file support, we store `Vec<StepFileSession>` where each session owns its `StepModel`, thickness groups, and selected index. Pieces from all sessions are merged into the global `pieces` vec.

### Threading Model

Unchanged from Sprint 003. The optimizer thread receives `pieces`, `sheet_config`, and `orientations` by value. The only change is that `run_optimizer` now dispatches to either `run_nesting` (SPP) or `run_nesting_bp` (BPP) based on whether the sheet has finite width. Both send `NestingUpdate::Improved` / `NestingUpdate::Finished` through the same channel.

```
Main Thread (egui)                    Optimizer Thread
+-------------------+               +----------------------+
| start_optimizer() | -- pieces --> | if sheet.width finite |
|                   |               |   run_nesting_bp()   |
|                   |               | else                 |
|                   |               |   run_nesting()      |
| poll_updates()    | <-- mpsc --  | sends NestingUpdate   |
+-------------------+               +----------------------+
```

## Implementation

### Phase 0: BPP Conversion + BinPackEngine

**Goal:** Create `BPInstance` from pieces and a new `BinPackEngine` that wraps `BPProblem`.

#### Task 0.1: Add `pieces_to_bp_instance` to conversion.rs

**File:** `crates/thetta-nesting/src/conversion.rs` (MODIFY)

```rust
use jagua_rs::probs::bpp::entities::{BPInstance, Bin};
use jagua_rs::entities::Container;

pub fn pieces_to_bp_instance(
    pieces: &[Piece],
    sheet: &SheetConfig,
    orientations: Option<&[Vec<f64>]>,
) -> Result<BPInstance> {
    ensure!(!pieces.is_empty(), "at least one piece is required");
    ensure!(
        sheet.width.is_finite() && sheet.width > 0.0
            && sheet.height.is_finite() && sheet.height > 0.0,
        "BPP requires finite sheet dimensions"
    );

    let ext_items: Vec<ExtBPItem> = pieces
        .iter()
        .enumerate()
        .map(|(idx, piece)| {
            let shape = polygon_to_ext_simple_polygon(&piece.polygon)?;
            let allowed_orientations = orientations
                .and_then(|ori| ori.get(idx))
                .map(|angles| angles.iter().map(|r| r.to_degrees() as f32).collect());

            Ok(ExtBPItem {
                base: ExtItem {
                    id: idx as u64,
                    allowed_orientations,
                    shape: ExtShape::SimplePolygon(shape),
                    min_quality: None,
                },
                demand: 1,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    // Create a rectangular bin (Container) matching sheet dimensions
    let total_area: f64 = pieces.iter().map(|p| polygon_ops::area(&p.polygon)).sum();
    let sheet_area = sheet.width * sheet.height;
    let stock = ((total_area / sheet_area).ceil() as usize + 2).max(3);

    let importer = Importer::new(
        DEFAULT_CDE_CONFIG,
        None,
        (sheet.separation > 0.0).then_some(sheet.separation as f32),
        None,
    );

    let ext_bin = ExtBin {
        base: ExtContainer {
            id: 0,
            shape: ExtShape::Rectangle {
                x_min: 0.0,
                y_min: 0.0,
                width: sheet.width as f32,
                height: sheet.height as f32,
            },
        },
        stock,
        cost: 1,
    };

    // Use jagua-rs BPP import
    let ext_instance = ExtBPInstance {
        name: "thetta-bp".to_string(),
        items: ext_items,
        bins: vec![ext_bin],
    };

    import_bp_instance(&importer, &ext_instance)
}
```

Note: The `ExtContainer` type and `Importer::import_container` method need verification. If `jagua-rs` doesn't expose `ExtContainer` or an `import_container` on `Importer`, we can construct the `Container` directly via `Container::new_strip(0, width, height, cde_config, shape_modify_config)` and build the `Bin` and `BPInstance` manually without going through the ext_repr layer. This is the more likely path since the SPP import also constructs `Strip` directly.

**Fallback construction (more likely):**
```rust
pub fn pieces_to_bp_instance(
    pieces: &[Piece],
    sheet: &SheetConfig,
    orientations: Option<&[Vec<f64>]>,
) -> Result<BPInstance> {
    // ... item import same as above via Importer::import_item ...

    let total_area: f64 = pieces.iter().map(|p| polygon_ops::area(&p.polygon)).sum();
    let sheet_area = sheet.width * sheet.height;
    let stock = ((total_area / sheet_area).ceil() as usize + 2).max(3);

    let shape_modify_config = ShapeModifyConfig {
        offset: (sheet.separation > 0.0).then_some(sheet.separation as f32 / 2.0),
        simplify_tolerance: None,
        narrow_concavity_cutoff: None,
    };

    let container = Container::new_strip(
        0, // bin_id
        sheet.width as f32,
        sheet.height as f32,
        DEFAULT_CDE_CONFIG,
        shape_modify_config,
    )?;

    let bin = Bin::new(container, stock, 1);
    Ok(BPInstance::new(items, vec![bin]))
}
```

#### Task 0.2: Create BinPackEngine

**File:** `crates/thetta-nesting/src/bin_pack.rs` (NEW)

Core implementation wrapping `BPProblem`. Key methods:

```rust
impl BinPackEngine {
    pub fn new(
        pieces: &[Piece],
        sheet: &SheetConfig,
        orientations: Option<&[RotationRange]>,
    ) -> Self {
        let per_piece_rad = /* same mapping as NestingEngine */;
        let instance = pieces_to_bp_instance(pieces, sheet, per_piece_rad.as_deref())
            .expect("failed to create BPInstance");
        let problem = BPProblem::new(instance.clone());

        Self {
            pieces: pieces.to_vec(),
            sheet: sheet.clone(),
            instance,
            problem,
            pk_map: vec![None; pieces.len()],
            orientations: orientations
                .map(|o| o.to_vec())
                .unwrap_or_else(|| default_orientations(pieces.len())),
        }
    }

    pub fn place_item_new_sheet(
        &mut self,
        idx: usize,
        x: f64, y: f64, rotation: f64,
    ) -> Option<LayKey> {
        let d_transf = to_internal_bp_transformation(&self.instance, idx, x, y, rotation);
        let placement = BPPlacement {
            layout_id: BPLayoutType::Closed { bin_id: 0 },
            item_id: idx,
            d_transf,
        };
        let (lkey, pik) = self.problem.place_item(placement);
        self.pk_map[idx] = Some((lkey, pik));
        Some(lkey)
    }

    pub fn place_item(
        &mut self,
        idx: usize,
        sheet_key: LayKey,
        x: f64, y: f64, rotation: f64,
    ) -> bool {
        if self.collision_cost(idx, sheet_key, x, y, rotation) > 0.0 {
            return false;
        }
        if self.pk_map[idx].is_some() {
            self.remove_item(idx);
        }
        let d_transf = to_internal_bp_transformation(&self.instance, idx, x, y, rotation);
        let placement = BPPlacement {
            layout_id: BPLayoutType::Open(sheet_key),
            item_id: idx,
            d_transf,
        };
        let (lkey, pik) = self.problem.place_item(placement);
        self.pk_map[idx] = Some((lkey, pik));
        true
    }

    pub fn collision_cost(
        &self,
        idx: usize,
        sheet_key: LayKey,
        x: f64, y: f64, rotation: f64,
    ) -> f64 {
        let layout = &self.problem.layouts[sheet_key];
        let item = self.instance.item(idx);
        let d_transf = to_internal_bp_transformation(&self.instance, idx, x, y, rotation);
        let transformed = item.shape_cd.transform_clone(&d_transf.compose());

        let own_hkey = self.pk_map[idx]
            .filter(|(lk, _)| *lk == sheet_key)
            .and_then(|(_, pik)| layout.cde().haz_key_from_pi_key(pik));

        let mut collector = CountingCollector::new(own_hkey);
        layout.cde().collect_poly_collisions(&transformed, &mut collector);
        collector.count() as f64
    }

    pub fn current_result(&self) -> NestingResult {
        let mut sheets = Vec::new();
        for (lkey, layout) in &self.problem.layouts {
            let mut placements = Vec::new();
            let mut placed_area = 0.0;

            for (pik, pi) in &layout.placed_items {
                let idx = pi.item_id;
                let ext = to_external_bp_transformation(&self.instance, idx, &pi.d_transf);
                let (x, y) = ext.translation();
                placements.push(Placement {
                    piece_id: self.pieces[idx].id,
                    x: x as f64,
                    y: y as f64,
                    rotation: ext.rotation() as f64,
                });
                placed_area += polygon_ops::area(&self.pieces[idx].polygon);
            }

            let sheet_area = self.sheet.width * self.sheet.height;
            sheets.push(Sheet {
                placements,
                utilization: if sheet_area > 0.0 { placed_area / sheet_area } else { 0.0 },
                strip_length: self.sheet.width,
            });
        }

        let unplaced = self.pieces.iter().enumerate()
            .filter_map(|(idx, p)| self.pk_map[idx].is_none().then_some(p.id))
            .collect();

        NestingResult { sheets, unplaced }
    }
}
```

#### Task 0.3: Internal/external transform helpers for BPP

**File:** `crates/thetta-nesting/src/conversion.rs` (MODIFY)

```rust
use jagua_rs::probs::bpp::entities::BPInstance;
use jagua_rs::entities::Instance; // BPInstance implements Instance trait

pub fn to_internal_bp_transformation(
    instance: &BPInstance,
    item_id: usize,
    x: f64, y: f64, rotation: f64,
) -> DTransformation {
    let ext = DTransformation::new(rotation as f32, (x as f32, y as f32));
    let pre = &instance.item(item_id).shape_orig.pre_transform;
    ext_to_int_transformation(&ext, pre)
}

pub fn to_external_bp_transformation(
    instance: &BPInstance,
    item_id: usize,
    int_transf: &DTransformation,
) -> DTransformation {
    let pre = &instance.item(item_id).shape_orig.pre_transform;
    int_to_ext_transformation(int_transf, pre)
}
```

### Phase 1: Fill-and-Spill LBF

**Goal:** Multi-sheet LBF that fills existing sheets before opening new ones.

#### Task 1.1: `run_lbf_bp` function

**File:** `crates/thetta-nesting/src/lbf.rs` (MODIFY)

```rust
use crate::bin_pack::BinPackEngine;
use jagua_rs::probs::bpp::entities::LayKey;

pub fn run_lbf_bp(
    engine: &mut BinPackEngine,
    config: LbfConfig,
    cancel: Option<&Arc<AtomicBool>>,
    mut on_placed: impl FnMut(&BinPackEngine),
) -> NestingResult {
    let mut order = (0..engine.piece_count()).collect::<Vec<_>>();
    order.sort_by(|a, b| {
        engine.piece_area(*b)
            .partial_cmp(&engine.piece_area(*a))
            .unwrap_or(Ordering::Equal)
    });

    let sheet = engine.sheet_config().clone();
    let grid_n = config.grid_n.max(2);

    let is_cancelled = || {
        cancel.as_ref().is_some_and(|c| c.load(AtomicOrdering::Relaxed))
    };

    for idx in order {
        if is_cancelled() { break; }

        let rotation_range = config.orientations.as_ref()
            .and_then(|v| v.get(idx))
            .or_else(|| engine.rotation_range(idx));
        let rotations = rotation_range
            .map(|r| rotation_range_to_samples(r, config.rotations_n))
            .unwrap_or_else(|| rotation_range_to_samples(&RotationRange::Continuous, config.rotations_n));

        // Try all existing sheets first (fill-and-spill)
        let mut placed = false;
        let existing_keys = engine.sheet_keys();

        for &sheet_key in &existing_keys {
            if is_cancelled() { break; }
            if let Some(pos) = find_lbf_position(engine, idx, sheet_key, &sheet, grid_n, &rotations) {
                engine.place_item(idx, sheet_key, pos.0, pos.1, pos.2);
                placed = true;
                break;
            }
        }

        // If no existing sheet works, open a new one
        if !placed && !is_cancelled() {
            if let Some(pos) = find_lbf_position_new_sheet(engine, idx, &sheet, grid_n, &rotations) {
                engine.place_item_new_sheet(idx, pos.0, pos.1, pos.2);
                placed = true;
            }
        }

        if placed {
            on_placed(engine);
        }
    }

    engine.current_result()
}

/// Find best LBF position on an existing sheet
fn find_lbf_position(
    engine: &BinPackEngine,
    idx: usize,
    sheet_key: LayKey,
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
                if engine.collision_cost(idx, sheet_key, x, y, *r) == 0.0 {
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

Note: `find_lbf_position_new_sheet` temporarily opens a sheet, probes, and either commits or rolls back. Implementation: open the sheet via `BPProblem`, find a position, and if found place the piece; otherwise remove the empty layout. Alternatively, since the first piece placed on a new sheet always fits (sheets are generously sized), we can just place at (0, 0, 0) as a fallback and let GLS improve it.

### Phase 2: Cross-Sheet GLS

**Goal:** GLS that can move pieces between sheets and eliminate empty sheets.

#### Task 2.1: `GLSBinPackOptimizer`

**File:** `crates/thetta-nesting/src/gls.rs` (MODIFY)

```rust
use crate::bin_pack::BinPackEngine;

pub struct GLSBinPackOptimizer {
    pub engine: BinPackEngine,
    pub penalty_weights: Vec<f64>,
    pub config: OptimizerConfig,
    pub best_feasible: Option<NestingResult>,
    pub rng: StdRng,
}

impl GLSBinPackOptimizer {
    pub fn new(engine: BinPackEngine, config: OptimizerConfig) -> Self {
        let penalty_weights = vec![1.0; engine.piece_count()];
        Self {
            engine,
            penalty_weights,
            config,
            best_feasible: None,
            rng: StdRng::seed_from_u64(42),
        }
    }

    pub fn run<F, P>(
        &mut self,
        mut on_update: F,
        mut on_progress: P,
        cancel: Option<&Arc<AtomicBool>>,
    ) -> NestingResult
    where
        F: FnMut(NestingResult),
        P: FnMut(usize, f64, f64, usize),
    {
        // Initial overlap check
        let initial_overlap = self.total_overlap();
        if initial_overlap == 0.0 {
            let result = self.engine.current_result();
            if result.unplaced.is_empty() {
                self.best_feasible = Some(result);
            }
        }

        let mut best_total_overlap = initial_overlap;
        let mut iteration = 0usize;
        let mut feasible_count = 0usize;

        while !is_cancelled(&cancel) {
            iteration += 1;

            // Pick piece with highest weighted collision cost across ALL sheets
            let best_idx = self.pick_worst_piece();
            let Some(idx) = best_idx else { break; };

            // Sample positions across ALL open sheets (cross-sheet movement)
            let sheet_keys = self.engine.sheet_keys();
            let mut all_samples = Vec::with_capacity(self.config.samples_per_move);

            for &sk in &sheet_keys {
                let per_sheet = self.config.samples_per_move / sheet_keys.len().max(1);
                for _ in 0..per_sheet {
                    let sample = uniform_sample(
                        &mut self.rng,
                        self.engine.sheet_config(),
                        self.engine.rotation_range(idx),
                    );
                    let score = self.engine.collision_cost(idx, sk, sample.x, sample.y, sample.rotation);
                    all_samples.push((sk, sample, score));
                }
            }

            // Pick best across all sheets
            all_samples.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));
            let candidates: Vec<_> = all_samples.into_iter().take(5).collect();

            let mut best_candidate = None;
            let mut best_score = f64::INFINITY;

            for (sk, seed, _) in candidates {
                let refined = coordinate_descent_refine(
                    seed,
                    |s| self.engine.collision_cost(idx, sk, s.x, s.y, s.rotation),
                    10.0,
                    self.config.refinement_depth,
                    self.engine.rotation_range(idx),
                );
                let score = self.engine.collision_cost(idx, sk, refined.x, refined.y, refined.rotation);
                if score < best_score {
                    best_score = score;
                    best_candidate = Some((sk, refined));
                }
            }

            if let Some((sk, cand)) = best_candidate {
                self.engine.force_place_item(idx, sk, cand.x, cand.y, cand.rotation);
            }

            // Update penalties
            for idx in 0..self.engine.piece_count() {
                if let Some((lk, placement)) = self.engine.placement_of(idx) {
                    if self.engine.collision_cost(idx, lk, placement.x, placement.y, placement.rotation) > 0.0 {
                        self.penalty_weights[idx] += self.config.penalty_increment;
                    }
                } else {
                    self.penalty_weights[idx] += self.config.penalty_increment;
                }
            }

            // Remove empty sheets
            self.engine.remove_empty_sheets();

            let current_overlap = self.total_overlap();
            if current_overlap < best_total_overlap {
                best_total_overlap = current_overlap;
            }
            if current_overlap == 0.0 {
                let feasible = self.engine.current_result();
                if feasible.unplaced.is_empty() {
                    feasible_count += 1;
                    let is_better = self.best_feasible.as_ref()
                        .map(|prev| feasible.sheets.len() < prev.sheets.len())
                        .unwrap_or(true);
                    if is_better {
                        self.best_feasible = Some(feasible.clone());
                        on_update(feasible);
                    }
                }
            }

            on_progress(iteration, current_overlap, best_total_overlap, feasible_count);
        }

        self.best_feasible.clone().unwrap_or_else(|| self.engine.current_result())
    }
}
```

### Phase 3: Sheet-Elimination Compression

**Goal:** Minimize sheet count by redistributing pieces from the least-used sheet.

#### Task 3.1: `compress_sheets` function

**File:** `crates/thetta-nesting/src/compression.rs` (MODIFY)

```rust
use crate::bin_pack::BinPackEngine;
use crate::lbf::{run_lbf_bp, LbfConfig, find_lbf_position};

/// Try to eliminate the last (least-utilized) sheet by redistributing its
/// pieces to earlier sheets. Repeat until elimination fails.
pub fn compress_sheets(
    pieces: &[Piece],
    sheet: SheetConfig,
    orientations: Option<&[RotationRange]>,
    cancel: Option<&Arc<AtomicBool>>,
    baseline: &NestingResult,
) -> NestingResult {
    let is_cancelled = || cancel.as_ref().is_some_and(|c| c.load(Ordering::Relaxed));

    // Start from baseline result
    let mut best = baseline.clone();

    loop {
        if is_cancelled() || best.sheets.len() <= 1 {
            break;
        }

        // Find least-utilized sheet (candidate for elimination)
        let last_idx = best.sheets.len() - 1;
        let victim_pieces: Vec<PieceId> = best.sheets[last_idx]
            .placements.iter().map(|p| p.piece_id).collect();

        if victim_pieces.is_empty() {
            // Empty sheet -- just remove it
            best.sheets.pop();
            continue;
        }

        // Try to place victim pieces into earlier sheets only
        // Build a new engine with only the earlier sheets' pieces, then LBF the victims in
        let mut earlier_pieces: Vec<Piece> = Vec::new();
        for sheet_result in &best.sheets[..last_idx] {
            for placement in &sheet_result.placements {
                if let Some(p) = pieces.iter().find(|p| p.id == placement.piece_id) {
                    earlier_pieces.push(p.clone());
                }
            }
        }
        for pid in &victim_pieces {
            if let Some(p) = pieces.iter().find(|p| p.id == *pid) {
                earlier_pieces.push(p.clone());
            }
        }

        // Run full BPP with stock = last_idx (only allow existing number of sheets)
        let mut trial_sheet = sheet.clone();
        let mut engine = BinPackEngine::new_with_max_sheets(
            &earlier_pieces,
            &trial_sheet,
            orientations,
            last_idx, // max sheets = current count minus 1
        );

        let trial = run_lbf_bp(
            &mut engine,
            LbfConfig {
                orientations: orientations.map(|o| o.to_vec()),
                ..LbfConfig::default()
            },
            cancel,
            |_| {},
        );

        if trial.unplaced.is_empty() && trial.sheets.len() < best.sheets.len() {
            best = trial;
        } else {
            break; // Cannot eliminate another sheet
        }
    }

    best
}
```

### Phase 4: Runner + Optimizer Dispatch

**Goal:** Wire up BPP pipeline alongside existing SPP pipeline.

#### Task 4.1: `run_nesting_bp` function

**File:** `crates/thetta-nesting/src/runner.rs` (MODIFY)

```rust
use crate::bin_pack::BinPackEngine;
use crate::compression::compress_sheets;
use crate::gls::GLSBinPackOptimizer;

pub fn run_nesting_bp(
    pieces: Vec<Piece>,
    sheet: SheetConfig,
    orientations: Option<Vec<RotationRange>>,
    config: OptimizerConfig,
    tx: mpsc::Sender<NestingUpdate>,
    cancel: Arc<AtomicBool>,
) {
    let mut engine = BinPackEngine::new(&pieces, &sheet, orientations.as_deref());
    let tx_lbf = tx.clone();

    let baseline = run_lbf_bp(
        &mut engine,
        LbfConfig {
            orientations: orientations.clone(),
            ..LbfConfig::default()
        },
        Some(&cancel),
        |eng| { let _ = tx_lbf.send(NestingUpdate::Improved(eng.current_result())); },
    );
    let _ = tx.send(NestingUpdate::Improved(baseline.clone()));

    if cancel.load(Ordering::Relaxed) {
        let _ = tx.send(NestingUpdate::Finished(baseline));
        return;
    }

    // GLS cross-sheet optimization
    let mut gls = GLSBinPackOptimizer::new(engine, config.clone());
    let tx2 = tx.clone();
    let gls_result = gls.run(
        |r| { let _ = tx.send(NestingUpdate::Improved(r)); },
        move |iteration, total_overlap, best_overlap, feasible_count| {
            let _ = tx2.send(NestingUpdate::Progress {
                iteration, total_overlap, best_overlap, feasible_count,
            });
        },
        Some(&cancel),
    );

    let best_so_far = if gls.best_feasible.is_some() { gls_result } else { baseline };

    if cancel.load(Ordering::Relaxed) {
        let _ = tx.send(NestingUpdate::Finished(best_so_far));
        return;
    }

    // Sheet elimination compression
    let compressed = compress_sheets(
        &pieces,
        sheet,
        orientations.as_deref(),
        Some(&cancel),
        &best_so_far,
    );
    let _ = tx.send(NestingUpdate::Improved(compressed.clone()));
    let _ = tx.send(NestingUpdate::Finished(compressed));
}
```

#### Task 4.2: Dispatch in optimizer.rs

**File:** `crates/thetta-nesting/src/optimizer.rs` (MODIFY)

```rust
pub fn run_optimizer(
    pieces: Vec<Piece>,
    sheet_config: SheetConfig,
    orientations: Option<Vec<RotationRange>>,
    config: OptimizerConfig,
    tx: mpsc::Sender<NestingUpdate>,
    cancel: Arc<AtomicBool>,
) {
    let use_bpp = sheet_config.width.is_finite()
        && sheet_config.height.is_finite()
        && sheet_config.width > 0.0
        && sheet_config.height > 0.0;

    if use_bpp {
        run_nesting_bp(pieces, sheet_config, orientations, config, tx, cancel);
    } else {
        run_nesting(pieces, sheet_config, orientations, config, tx, cancel);
    }
}
```

Wait -- the current `SheetConfig::default()` already has finite width (2440.0) and height (1220.0). So this dispatch would route ALL nesting to BPP by default. Benchmarks load custom `SheetConfig` via `load_benchmark` which sets `width` from the benchmark data. We need a different dispatch signal.

**Revised dispatch:** Add a `NestingMode` enum to control routing explicitly.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NestingMode {
    StripPacking,  // SPP -- single infinite strip
    BinPacking,    // BPP -- fixed-size sheets
}

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
        NestingMode::StripPacking => run_nesting(pieces, sheet_config, orientations, config, tx, cancel),
        NestingMode::BinPacking => run_nesting_bp(pieces, sheet_config, orientations, config, tx, cancel),
    }
}
```

`AppState::start_optimizer` passes `NestingMode::BinPacking` for STEP/native workflows and `NestingMode::StripPacking` when `self.benchmark.is_some()`.

### Phase 5: Viewport Tab Bar

**Goal:** Tab-based sheet navigation in the viewport.

#### Task 5.1: Tab bar rendering

**File:** `crates/thetta-app/src/viewport.rs` (MODIFY)

```rust
pub fn show_viewport(
    ui: &mut egui::Ui,
    state: &mut ViewportState,
    app: &AppState,
) -> Option<PieceId> {
    let available = ui.available_size();

    // Sheet tab bar
    let mut active_sheet = app.active_sheet_idx;
    if let Some(result) = &app.current_result {
        if result.sheets.len() > 1 {
            ui.horizontal(|ui| {
                for (i, sheet) in result.sheets.iter().enumerate() {
                    let label = format!(
                        "Sheet {} ({:.0}%)",
                        i + 1,
                        sheet.utilization * 100.0,
                    );
                    if ui.selectable_label(i == active_sheet, label).clicked() {
                        active_sheet = i;
                    }
                }
            });
        }
    }

    // Render the active sheet (same as current single-sheet code, but using
    // result.sheets[active_sheet] instead of result.sheets.first())
    let (rect, response) = ui.allocate_exact_size(/* remaining */available, Sense::click_and_drag());
    // ... existing rendering logic, but indexing sheets[active_sheet] ...
}
```

The `active_sheet_idx` is stored on `AppState` so tab clicks persist across frames. When a new `NestingResult` arrives, clamp `active_sheet_idx` to `0..sheets.len()`.

#### Task 5.2: Update `apply_result` for multi-sheet stats

**File:** `crates/thetta-app/src/state.rs` (MODIFY)

```rust
fn apply_result(&mut self, result: NestingResult) {
    let total_placed: usize = result.sheets.iter().map(|s| s.placements.len()).sum();
    let total_area: f64 = result.sheets.iter().map(|s| {
        s.placements.iter()
            .filter_map(|p| self.pieces.iter().find(|piece| piece.id == p.piece_id))
            .map(|piece| polygon_ops::area(&piece.polygon))
            .sum::<f64>()
    }).sum();
    let total_sheet_area = result.sheets.len() as f64 * self.sheet_config.width * self.sheet_config.height;
    let utilization = if total_sheet_area > 0.0 { total_area / total_sheet_area * 100.0 } else { 0.0 };

    self.stats.pieces_placed = total_placed;
    self.stats.pieces_total = self.pieces.len();
    self.stats.utilization_pct = utilization;
    self.stats.strip_length = result.sheets.first().map(|s| s.strip_length).unwrap_or(0.0);

    // Clamp active sheet
    if self.active_sheet_idx >= result.sheets.len() {
        self.active_sheet_idx = result.sheets.len().saturating_sub(1);
    }

    self.current_result = Some(result);
}
```

### Phase 6: Multi-File Loading + Unique IDs

**Goal:** Support loading multiple STEP files with globally unique piece IDs.

#### Task 6.1: ID remapping on load

**File:** `crates/thetta-app/src/state.rs` (MODIFY)

Add `next_piece_id: u64` field. On any file load, remap:

```rust
pub fn load_json_file(&mut self, path: &Path) -> Result<usize, String> {
    // ... existing parsing ...
    for p in &mut parsed {
        p.id = PieceId(self.next_piece_id);
        self.next_piece_id += 1;
        p.source_file = path.to_path_buf();
    }
    // ... rest unchanged ...
}

fn apply_step_import(&mut self, result: StepImportResult) {
    // Remap piece IDs to be globally unique
    let mut pieces = result.pieces;
    let mut refs = result.piece_refs;
    for (piece, pref) in pieces.iter_mut().zip(refs.iter_mut()) {
        let new_id = PieceId(self.next_piece_id);
        self.next_piece_id += 1;
        pref.piece_id = new_id;
        piece.id = new_id;
    }
    // ... rest of existing logic ...
}
```

#### Task 6.2: Multi-file STEP sessions

**File:** `crates/thetta-app/src/state.rs` (MODIFY)

Replace single `step_session` with a vec:

```rust
pub struct StepFileSession {
    pub model: Arc<Mutex<StepModel>>,
    pub piece_refs: Vec<StepPieceRef>,
    pub thickness_groups: Vec<ThicknessInfo>,
    pub selected_thickness_idx: usize,
    pub source_path: PathBuf,
}

pub struct AppState {
    // Replace:
    //   step_session, step_piece_refs, thickness_groups, selected_thickness_idx, step_source_path
    // With:
    pub step_sessions: Vec<StepFileSession>,
    // ... rest unchanged ...
}
```

Each "Load STEP" click adds a new session. Thickness selection operates on a specific session index. `replace_step_pieces` removes only pieces from the affected session.

### Phase 7: Panel Updates

**Goal:** Show sheet count, per-sheet stats, and multi-file info in the side panel.

#### Task 7.1: Stats panel updates

**File:** `crates/thetta-app/src/panels.rs` (MODIFY)

```rust
// In Stats section:
egui::CollapsingHeader::new("Stats").default_open(true).show(ui, |ui| {
    if let Some(result) = &app.current_result {
        ui.label(format!("Sheets: {}", result.sheets.len()));
    }
    ui.label(format!("Utilization: {:.2}%", app.stats.utilization_pct));
    ui.label(format!(
        "Placed: {}/{}",
        app.stats.pieces_placed, app.stats.pieces_total
    ));
    ui.label(format!("Elapsed: {:.1}s", app.stats.elapsed.as_secs_f64()));

    // Per-sheet breakdown
    if let Some(result) = &app.current_result {
        if result.sheets.len() > 1 {
            egui::CollapsingHeader::new("Per Sheet").default_open(false).show(ui, |ui| {
                for (i, sheet) in result.sheets.iter().enumerate() {
                    ui.label(format!(
                        "Sheet {}: {:.1}% ({} pcs)",
                        i + 1,
                        sheet.utilization * 100.0,
                        sheet.placements.len(),
                    ));
                }
            });
        }
    }
});
```

### Phase 8: Testing

#### Task 8.1: BPP conversion unit tests

**File:** `crates/thetta-nesting/src/conversion.rs` (MODIFY -- add tests)

```rust
#[test]
fn bp_instance_creates_correct_bin_dimensions() {
    let pieces = vec![rect_piece(0, 100.0, 50.0)];
    let sheet = SheetConfig { width: 200.0, height: 100.0, separation: 0.0 };
    let instance = pieces_to_bp_instance(&pieces, &sheet, None).unwrap();
    assert_eq!(instance.bins.len(), 1);
    let bin_area = instance.bins[0].container.area();
    assert!((bin_area - 20000.0).abs() < 1.0); // 200 * 100
}

#[test]
fn bp_instance_stock_exceeds_piece_count() {
    let pieces: Vec<Piece> = (0..10).map(|i| rect_piece(i, 100.0, 50.0)).collect();
    let sheet = SheetConfig { width: 200.0, height: 100.0, separation: 0.0 };
    let instance = pieces_to_bp_instance(&pieces, &sheet, None).unwrap();
    assert!(instance.bins[0].stock >= 3); // at least ceil(50000/20000)+2 = 5
}
```

#### Task 8.2: Multi-sheet integration test

**File:** `crates/thetta-nesting/tests/multi_sheet.rs` (NEW)

```rust
#[test]
fn pieces_overflow_to_multiple_sheets() {
    // Create 20 large pieces that cannot fit on one 200x100 sheet
    let pieces: Vec<Piece> = (0..20)
        .map(|i| rect_piece(i, 80.0, 40.0))
        .collect();
    let sheet = SheetConfig { width: 200.0, height: 100.0, separation: 2.0 };

    let (tx, rx) = mpsc::channel();
    let cancel = Arc::new(AtomicBool::new(false));
    run_nesting_bp(pieces.clone(), sheet, None, OptimizerConfig::default(), tx, cancel);

    let mut final_result = None;
    while let Ok(update) = rx.recv() {
        if let NestingUpdate::Finished(r) = update {
            final_result = Some(r);
            break;
        }
    }

    let result = final_result.unwrap();
    assert!(result.sheets.len() > 1, "Expected multiple sheets");
    assert!(result.unplaced.is_empty(), "All pieces should be placed");

    let total_placed: usize = result.sheets.iter().map(|s| s.placements.len()).sum();
    assert_eq!(total_placed, 20);
}

#[test]
fn single_sheet_sufficient_stays_single() {
    let pieces = vec![rect_piece(0, 10.0, 10.0), rect_piece(1, 10.0, 10.0)];
    let sheet = SheetConfig { width: 200.0, height: 100.0, separation: 0.0 };

    let (tx, rx) = mpsc::channel();
    let cancel = Arc::new(AtomicBool::new(false));
    run_nesting_bp(pieces, sheet, None, OptimizerConfig::default(), tx, cancel);

    let mut final_result = None;
    while let Ok(update) = rx.recv() {
        if let NestingUpdate::Finished(r) = update {
            final_result = Some(r);
            break;
        }
    }

    let result = final_result.unwrap();
    assert_eq!(result.sheets.len(), 1);
}
```

#### Task 8.3: Benchmark regression test

**File:** `crates/thetta-nesting/tests/benchmark_smoke.rs` (MODIFY -- verify unchanged)

Existing benchmark tests use `run_nesting` (SPP path). Verify they still pass. The optimizer dispatch sends benchmarks through `NestingMode::StripPacking`, so no change expected.

#### Task 8.4: GUI smoke test

Manual verification:
1. Launch app, load STEP file with many parts
2. Verify multiple sheet tabs appear
3. Click between tabs, verify rendering
4. Check per-sheet stats in panel
5. Load second STEP file, verify pieces accumulate
6. Export, verify multiple STEP files produced

## Files Summary

### New Files

| File | Purpose | Est. LOC |
|------|---------|----------|
| `crates/thetta-nesting/src/bin_pack.rs` | `BinPackEngine` wrapping `BPProblem` | 280 |
| `crates/thetta-nesting/tests/multi_sheet.rs` | Multi-sheet integration tests | 90 |
| **Total new** | | **~370** |

### Modified Files

| File | Changes | Est. Delta |
|------|---------|------------|
| `crates/thetta-nesting/src/conversion.rs` | Add `pieces_to_bp_instance`, BP transform helpers | +80 |
| `crates/thetta-nesting/src/lbf.rs` | Add `run_lbf_bp`, `find_lbf_position` | +90 |
| `crates/thetta-nesting/src/gls.rs` | Add `GLSBinPackOptimizer` with cross-sheet sampling | +120 |
| `crates/thetta-nesting/src/compression.rs` | Add `compress_sheets` for sheet elimination | +60 |
| `crates/thetta-nesting/src/runner.rs` | Add `run_nesting_bp` pipeline, `NestingMode` | +50 |
| `crates/thetta-nesting/src/optimizer.rs` | Dispatch based on `NestingMode` | +15 |
| `crates/thetta-nesting/src/lib.rs` | Add `pub mod bin_pack;` | +1 |
| `crates/thetta-nesting/Cargo.toml` | Add `bpp` feature to jagua-rs dep (if needed) | +2 |
| `crates/thetta-app/src/state.rs` | Multi-file sessions, `active_sheet_idx`, `next_piece_id`, `NestingMode` dispatch, aggregate stats | +100 |
| `crates/thetta-app/src/viewport.rs` | Tab bar rendering, per-sheet viewport, active sheet index | +60 |
| `crates/thetta-app/src/panels.rs` | Sheet count in stats, per-sheet breakdown, file grouping | +40 |
| `crates/thetta-app/src/main.rs` | Pass `NestingMode` to optimizer | +5 |
| `crates/thetta-geometry/src/types.rs` | No changes (NestingResult already supports Vec<Sheet>) | +0 |
| **Total modified** | | **~623 delta** |

### Estimated Total Effort: ~993 LOC (new + delta)

Current codebase is ~14.7K LOC (post-Sprint 003). This sprint is smaller than Sprint 003 because we are reusing existing jagua-rs BPP types and the overall pipeline structure is already established.

## Definition of Done

### P0 -- Must Ship

- [ ] `pieces_to_bp_instance` creates a valid `BPInstance` with rectangular bin matching sheet dimensions
- [ ] `BinPackEngine` wraps `BPProblem` and supports place/remove/collision_cost across multiple layouts
- [ ] Fill-and-spill LBF places pieces across multiple sheets when they don't fit on one
- [ ] `NestingResult` contains correct multi-sheet data (per-sheet placements, utilization)
- [ ] `run_nesting_bp` pipeline runs LBF -> GLS -> compression for BPP mode
- [ ] Optimizer dispatches to BPP for STEP/native workflows and SPP for benchmarks
- [ ] Viewport renders the active sheet via tab selection
- [ ] Stats show aggregate utilization and piece count across all sheets
- [ ] All existing tests pass (`cargo test --workspace`)
- [ ] Benchmark regression: `jakobs1` and other benchmarks produce valid SPP results unchanged

### P1 -- Should Ship

- [ ] Cross-sheet GLS moves pieces between sheets to reduce overlap
- [ ] Sheet-elimination compression removes the last sheet when possible
- [ ] Tab bar shows per-sheet utilization percentages
- [ ] Per-sheet stats breakdown in side panel
- [ ] Loading multiple STEP files accumulates pieces with unique IDs
- [ ] STEP export produces correct number of files matching sheet count
- [ ] Empty sheets are automatically removed from results

### P2 -- Stretch

- [ ] "All Sheets" minimap tab showing overview
- [ ] Piece list grouped by source file in side panel
- [ ] Per-file thickness selector (each STEP file has independent thickness)
- [ ] GLS feasibility comparison uses total utilization across sheets (not just sheet count)
- [ ] Animated tab transitions
- [ ] Sheet drag-reordering

### Cut if Behind Schedule

- "All Sheets" minimap (P2)
- Per-file thickness (P2)
- Animated transitions (P2)
- Sheet drag-reordering (P2)

## Risks

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| jagua-rs BPP types behind a feature gate that is not enabled | High | Medium | Check `crates/jagua-rs/Cargo.toml` features. If gated, add `bpp` feature to dependency. If not gated, no action. |
| `Container::new_strip` creates a strip-style container (unbounded width) not a bounded rectangle | High | Medium | Verify `new_strip` creates proper bounded bbox. The constructor takes explicit width+height and builds a rectangular `SPolygon`, so it should work for fixed bins too despite the name. |
| BPP `place_item` with `BPLayoutType::Closed` panics if stock exhausted | High | Low | Set stock generously (total_area/sheet_area + 2). Add a check before opening new bins. |
| Cross-sheet GLS doubles sampling time (N sheets x M samples) | Medium | High | Divide `samples_per_move` across sheets. With 3-5 sheets, each gets 200-333 samples from 1000 total. Acceptable. |
| `BinPackEngine::collision_cost` needs layout's CDE which requires `cde()` method | Medium | Medium | `Layout` has a `cde()` method (used in SPP engine). Verify it's public. If not, access via `problem.layouts[key]` which owns the Layout. |
| Tab bar in viewport consumes vertical space, reducing nesting view | Low | High | Tab bar is ~20px. On 1440p display this is negligible. Hide tabs when only 1 sheet. |
| `NestingMode` enum adds a parameter to `run_optimizer` -- breaks existing call sites | Low | High | Only `AppState::start_optimizer` calls it. One call site to update. |
| Piece ID remapping breaks `StepPieceRef` lookup during export | Medium | Medium | Remap IDs in `StepPieceRef` at the same time as pieces. Keep the mapping consistent. Test with round-trip. |
| `BPProblem::remove_item` requires both `LayKey` and `PItemKey` -- different from SPP API | Medium | Low | `pk_map` stores `(LayKey, PItemKey)` tuples. The BPP API is well-defined for this. |

## Security

- No new network access. Fully local application.
- No new crate dependencies (jagua-rs BPP types already in workspace).
- No new `unsafe` code. `BinPackEngine` uses only safe Rust over jagua-rs public API.
- File I/O unchanged from Sprint 003 (native dialogs, user-specified paths only).
- No user-provided code, scripts, or macros.

## Dependencies

### New Rust Crates

None. All required types are already in the `jagua-rs` workspace member.

### Modified Dependencies

| Crate | Change | Purpose |
|-------|--------|---------|
| `jagua-rs` | Potentially add `bpp` feature flag | Enable BPP types if feature-gated |

### Existing Crates (Unchanged)

| Crate | Purpose |
|-------|---------|
| `thetta-geometry` | `NestingResult`, `Sheet`, `Piece`, `Placement` -- all already support multi-sheet |
| `thetta-nesting` | Engine, LBF, GLS, runner -- extended with BPP variants |
| `thetta-step` | STEP import/export -- unchanged, already iterates sheets for export |
| `jagua-rs` | BPP types: `BPInstance`, `BPProblem`, `BPPlacement`, `BPLayoutType`, `LayKey`, `Bin` |
| `eframe` / `egui` | GUI framework |
| `rfd` | File dialogs |

## Open Questions

1. **Is `Container::new_strip` suitable for bounded rectangular bins?** The name suggests strip semantics but the implementation creates a bounded rectangular polygon from (0,0) to (width, height). If the CDE exterior hazard correctly clips to this rectangle, it works for BPP bins. Verify with a placement test: place a piece partially outside the container and confirm `collision_cost > 0`.

2. **Are BPP types behind a feature gate in jagua-rs?** Check `crates/jagua-rs/Cargo.toml`. If `pub mod bpp` is conditional on a feature, we need to enable it. The existing dependency uses `features = ["spp"]`.

3. **Should benchmarks ever use BPP?** Currently no -- benchmarks define a strip height and expect infinite-width strip packing. But some academic BPP benchmarks exist. Defer to a future sprint.

4. **GLS comparison metric for BPP.** For SPP, "better" means shorter strip. For BPP, "better" means fewer sheets. If two solutions have the same sheet count, prefer higher minimum-sheet utilization (more balanced). The current draft uses `sheets.len()` as the primary comparison.

5. **Should the compression phase re-run GLS on the reduced sheet set?** The draft only uses LBF for redistribution. A GLS pass after each sheet elimination would produce tighter packing but significantly increase runtime. Recommend deferring GLS-in-compression to a future sprint.

6. **Default `NestingMode` for non-benchmark, non-STEP workflows (plain JSON load)?** Propose: JSON polygon files use `BinPacking` mode when sheet width is finite, matching the IDEA.md vision. Only benchmark JSON files (detected by `name` + `items` keys) use `StripPacking`. This means users loading custom JSON piece files also get multi-sheet support.

7. **`Layout::cde()` visibility.** The SPP engine accesses `self.problem.layout.cde()` directly. For BPP, we access `self.problem.layouts[key].cde()`. Verify that `Layout::cde()` is `pub`. If it is `pub(crate)`, we need to access it through the `BPProblem` API or make it public in our fork.
