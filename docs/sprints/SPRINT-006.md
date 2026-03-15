# Sprint 006: Automatic Per-Part Tab Placement for CNC Routing

## Overview

Implement automatic holding-tab placement as a post-processing step after nesting. Given a `NestingResult` with placed pieces, the algorithm computes holding tabs for each piece's outer contour. Tabs are small uncut material bridges that keep CNC-routed parts attached to the stock sheet during cutting, preventing part ejection, rotation, and tool damage.

The algorithm implements the composite approach from RESEARCH-005 Section 4: CamBam's perimeter-proportional count formula, FreeCAD's edge-length-weighted distribution, and community-derived corner-avoidance rules. Tab count is `clamp(round(perimeter / tab_distance), min_tabs, max_tabs)` with a minimum of 3 tabs per part. Tabs are distributed across edges proportional to `edge_length * edge_score`, where straight edges score highest (1.0) and concave curves score lowest (0.3). Tab dimensions are derived from material type and thickness per Section 4.5.

The implementation spans two crates: new types and a `tabs` module in `thetta-geometry` for the pure-geometry algorithm, and UI/state integration in `thetta-app` for configuration, rendering, and export. Tab results are stored per-placement in `AppState`, keyed by `(sheet_index, placement_index)`, without modifying `NestingResult` or `Sheet` types. Shared/adjacent tabs between neighboring parts are explicitly deferred. This sprint targets per-part rectangular tabs only.

**Scope:** `Tab`, `TabConfig`, `MaterialType` types, `tabs.rs` algorithm module, `TabState` in app state, tab UI panel, viewport tab rendering, JSON export with tab metadata.
**Prerequisites:** Sprint 005 complete. Multi-sheet nesting operational. `NestingResult` with `Vec<Sheet>` and per-sheet placements. STEP import/export pipeline operational. Viewport renders placed pieces with zoom/pan across sheets.
**Deferred:** Shared/adjacent tabs between neighboring parts, triangular/ramped tab shapes, STEP geometry export of tabs, inner-contour (hole) tabs, `NestingUpdate::Improved` tab recomputation.

## Use Cases

### UC1: Automatic Tab Placement on Nesting Output
User imports parts, runs nesting, and tabs auto-generate when nesting finishes (`NestingUpdate::Finished`). Each placed piece above the size threshold receives 3-20 tabs distributed along its outer contour. No manual intervention required.

### UC2: Material-Aware Tab Sizing
User selects material type (soft wood, hard wood, plastic, composite) and enters material thickness. Tab height and width are automatically computed using Section 4.5 formulas. User can override with manual values.

### UC3: Configurable Tab Parameters
User adjusts tab spacing distance, min/max count, corner exclusion distance, and size threshold via the Tabs UI panel. Changes trigger re-computation with immediate visual feedback.

### UC4: Visual Verification in Viewport
Tabs render as small colored rectangles on piece edges in the viewport. User visually verifies placement before export. Tabs are drawn on top of piece contours with distinct coloring (orange/amber).

### UC5: Small Part Handling
Parts with perimeter below `size_threshold` (default 30mm) are automatically skipped. Status display shows "K pieces skipped" count.

### UC6: Multi-Sheet Tab Placement
Tabs are computed independently per sheet, per placement. Multi-sheet results produce per-sheet tab data with no cross-sheet dependencies.

### UC7: Deterministic Output
Same piece geometry + same tab config always produces identical tab positions. No randomness. Reproducible CNC output across runs.

### UC8: Export with Tab Metadata
Tab data is included in JSON export as structured metadata alongside placement data, enabling downstream toolpath generators to incorporate tabs.

## Architecture

### Component Overview

```
+-----------------------------------------------------------+
|                      thetta-app                            |
|  +----------+  +-----------+  +--------------------------+ |
|  | panels.rs|  |viewport.rs|  |       state.rs           | |
|  | Tab UI   |  | Tab render|  | TabState { config,       | |
|  | controls |  | overlays  |  |   results: HashMap<      | |
|  +----+-----+  +-----+-----+  |   (usize,usize),Vec<Tab>| |
|       |              |        |   > }                    | |
|       |              |        +-----------+--------------+ |
+-------+--------------+--------------------+----------------+
        |              |                    |
        |              |        +-----------v--------------+
        |              |        |    thetta-geometry        |
        |              |        |  +---------------------+ |
        |              +--------+  |    types.rs          | |
        |                       |  | Tab, TabConfig,      | |
        |                       |  | MaterialType         | |
        |                       |  +---------------------+ |
        |                       |  +---------------------+ |
        +-------------------+---+  |    tabs.rs           | |
                                |  | generate_tabs()      | |
                                |  | Edge scoring,        | |
                                |  | distribution,        | |
                                |  | validation           | |
                                |  +---------------------+ |
                                |  +---------------------+ |
                                |  | polygon_ops.rs       | |
                                |  | + perimeter()        | |
                                |  | + edge helpers       | |
                                |  +---------------------+ |
                                +--------------------------+
```

### Data Flow

```
NestingResult (sheets with placements)
        |
        v
For each Sheet (sheet_index):
  For each Placement (placement_index):
    |
    +-- 1. Look up Piece by piece_id
    +-- 2. Apply placement transform (translate + rotate) to get placed contour
    +-- 3. Call generate_tabs(placed_contour, tab_config) -> Vec<Tab>
    |       |
    |       +-- Compute perimeter; early-return [] if < size_threshold
    |       +-- Compute tab count: clamp(round(perim/distance), min, max)
    |       +-- Score each edge (straight=1.0, convex=0.8, concave=0.3, short=0.0)
    |       +-- Distribute tabs proportional to edge_length * score
    |       +-- Validate minimum spacing (no overlap, >= 2*tab_width)
    |       +-- Validate minimum count (force-place with relaxed exclusion if needed)
    |       +-- Return Vec<Tab>
    |
    +-- 4. Store at key (sheet_index, placement_index) in TabState.results
```

### Module Placement Rationale

The tab algorithm is pure geometry -- it takes a polygon and config, returns tab positions. It has no dependency on nesting internals, collision detection, or jagua-rs. Therefore it belongs in `thetta-geometry`, not `thetta-nesting`. This keeps the leaf crate's role as the canonical geometry library.

The app layer (`thetta-app`) handles orchestration: it owns `TabState` in `AppState`, triggers tab generation when nesting finishes, stores results, renders tabs in the viewport, and exposes the configuration UI.

### Design Decisions

1. **Per-placement tab storage, not per-PieceId.** A `HashMap<PieceId, Vec<Tab>>` would overwrite results when the same piece is placed multiple times (quantity > 1). Keying by `(sheet_index, placement_index)` handles this correctly.

2. **TabState separate from NestingResult.** Tab data is derived state computed after nesting. Modifying `NestingResult` or `Sheet` would require changes across all crates that consume these types. Separate storage in `AppState` keeps the change isolated.

3. **Trigger on `NestingUpdate::Finished` only.** Computing tabs on every `Improved` during GLS would trigger hundreds of recomputes. Users get tabs when nesting completes. Manual "Regenerate Tabs" button available for config changes.

4. **Rectangular tabs only.** RESEARCH-005 recommends rectangular for Phase 1. Triangular/ramped require different geometry and G-code generation, deferred to a future sprint.

5. **Material-derived sizing with manual override.** `MaterialType` + `material_thickness` computes default tab height/width via Section 4.5 formulas. User can toggle to manual mode and set explicit values.

## Implementation

### Phase 1: Core Types, Geometry Utilities, and Tab Algorithm

**Goal**: Add new types, geometry helpers, and the complete tab generation algorithm.

#### New Types

**`crates/thetta-geometry/src/types.rs`** -- Add the following types:

```rust
/// Material type for tab sizing heuristics (RESEARCH-005 Section 4.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaterialType {
    WoodSoft,
    WoodHard,
    Plastic,
    Composite,
}

/// A single holding tab on a piece contour.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tab {
    /// World-space position on the contour edge.
    pub position: Point2D,
    /// Index of the edge in the polygon's outer ring (edge from outer[i] to outer[i+1]).
    pub edge_index: usize,
    /// Parameter along the edge (0.0 = start vertex, 1.0 = end vertex).
    pub t: f64,
    /// Tab width in mm along cutting direction.
    pub width: f64,
    /// Tab height in mm from cut bottom.
    pub height: f64,
}

/// Configuration for automatic tab placement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TabConfig {
    /// Target distance between tabs in mm.
    pub tab_distance: f64,
    /// Absolute minimum tab count per part (prevents rotation).
    pub min_tabs: usize,
    /// Absolute maximum tab count per part.
    pub max_tabs: usize,
    /// Tab width in mm (manual mode), or computed from material.
    pub tab_width: f64,
    /// Tab height in mm (manual mode), or computed from material.
    pub tab_height: f64,
    /// Minimum perimeter in mm for a part to receive tabs.
    pub size_threshold: f64,
    /// Minimum distance from any polygon vertex (corner exclusion zone) in mm.
    pub corner_exclusion: f64,
    /// Material thickness in mm (used for auto-sizing).
    pub material_thickness: f64,
    /// Material type (used for auto-sizing).
    pub material_type: MaterialType,
    /// When true, tab_width and tab_height are computed from material properties.
    /// When false, manual tab_width and tab_height values are used directly.
    pub auto_size: bool,
}

impl Default for TabConfig {
    fn default() -> Self {
        Self {
            tab_distance: 50.0,
            min_tabs: 3,
            max_tabs: 20,
            tab_width: 10.0,
            tab_height: 6.0,
            size_threshold: 30.0,
            corner_exclusion: 5.0,
            material_thickness: 18.0,
            material_type: MaterialType::WoodSoft,
            auto_size: true,
        }
    }
}

impl TabConfig {
    /// Compute effective tab height based on material properties (Section 4.5).
    pub fn effective_tab_height(&self) -> f64 {
        if !self.auto_size {
            return self.tab_height;
        }
        let ratio = match self.material_type {
            MaterialType::WoodSoft => 0.40,
            MaterialType::WoodHard => 0.33,
            MaterialType::Plastic => 0.33,
            MaterialType::Composite => 0.50,
        };
        ratio * self.material_thickness
    }

    /// Compute effective tab width based on material properties (Section 4.5).
    /// Width = clamp(1.5 * height, 5.0, 25.0).
    pub fn effective_tab_width(&self) -> f64 {
        if !self.auto_size {
            return self.tab_width;
        }
        let height = self.effective_tab_height();
        (1.5 * height).clamp(5.0, 25.0)
    }

    /// Validated tab_distance (minimum 1.0 to prevent division issues).
    pub fn effective_tab_distance(&self) -> f64 {
        self.tab_distance.max(1.0)
    }
}
```

#### Geometry Utilities

**`crates/thetta-geometry/src/polygon_ops.rs`** -- Add the following functions:

```rust
/// Compute perimeter of outer contour only (sum of edge lengths).
pub fn perimeter(poly: &Polygon) -> f64 {
    let pts = &poly.outer;
    if pts.len() < 2 {
        return 0.0;
    }
    let mut sum = 0.0;
    for i in 0..pts.len() {
        let j = (i + 1) % pts.len();
        let dx = pts[j].x - pts[i].x;
        let dy = pts[j].y - pts[i].y;
        sum += (dx * dx + dy * dy).sqrt();
    }
    sum
}

/// Euclidean distance between two points.
pub fn edge_length(a: Point2D, b: Point2D) -> f64 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    (dx * dx + dy * dy).sqrt()
}

/// Linear interpolation along an edge: point_on_edge(a, b, 0.0) == a, point_on_edge(a, b, 1.0) == b.
pub fn point_on_edge(a: Point2D, b: Point2D, t: f64) -> Point2D {
    Point2D {
        x: a.x + t * (b.x - a.x),
        y: a.y + t * (b.y - a.y),
    }
}
```

Add unit tests for `perimeter` (rectangle 100x50 -> 300, triangle, circle approximation) and `point_on_edge` (midpoint, endpoints).

#### Tab Algorithm

**`crates/thetta-geometry/src/tabs.rs`** (new file)

Public API:

```rust
/// Generate holding tabs for a placed (world-space) outer contour.
///
/// Returns an empty Vec if the contour perimeter is below `config.size_threshold`,
/// if the polygon has fewer than 3 vertices, or if all edges are too short for tabs.
///
/// The input polygon MUST be the placed contour in world space (after applying
/// placement transform). The outer ring MUST be CCW winding.
pub fn generate_tabs(contour: &Polygon, config: &TabConfig) -> Vec<Tab>
```

Internal algorithm (implements RESEARCH-005 Section 4.4):

**Step 1: Perimeter check and tab count**
```
perimeter = polygon_ops::perimeter(contour)
if perimeter < config.size_threshold:
    return []
if contour.outer.len() < 3:
    return []

tab_distance = config.effective_tab_distance()
base_count = (perimeter / tab_distance).round() as usize
count = base_count.clamp(config.min_tabs, config.max_tabs)

tab_width = config.effective_tab_width()
tab_height = config.effective_tab_height()
```

**Step 2: Build scored edge list**

For each edge `i` in the outer ring (edge from `outer[i]` to `outer[(i+1) % n]`):

```rust
/// Classify a vertex by the turning angle at prev -> curr -> next.
/// For CCW outer contour:
///   cross > threshold  -> convex (score 1.0)
///   cross < -threshold -> concave (score 0.3)
///   |cross| <= threshold -> straight (score 0.8)
fn classify_vertex(prev: Point2D, curr: Point2D, next: Point2D) -> f64 {
    let cross = (curr.x - prev.x) * (next.y - curr.y)
              - (curr.y - prev.y) * (next.x - curr.x);
    const COLLINEAR_THRESHOLD: f64 = 1e-6;
    if cross > COLLINEAR_THRESHOLD {
        1.0  // convex turn (outer contour, CCW)
    } else if cross < -COLLINEAR_THRESHOLD {
        0.3  // concave turn
    } else {
        0.8  // straight / collinear
    }
}

/// Score an edge based on average of its two endpoint vertex classifications.
/// Returns 0.0 if edge is too short for a tab.
fn score_edge(start_score: f64, end_score: f64, length: f64, min_length: f64) -> f64 {
    if length < min_length {
        return 0.0;
    }
    (start_score + end_score) / 2.0
}
```

Minimum edge length for scoring: `tab_width + 2.0 * config.corner_exclusion`.

**Step 3: Edge-length-proportional distribution**

```
total_weighted_length = sum(edge_length[i] * edge_score[i] for all edges)
if total_weighted_length <= 0.0:
    return []  // all edges too short or zero-scored

// Allocate tabs per edge proportionally
for each edge i, sorted by (length * score) descending:
    fraction = (edge_length[i] * edge_score[i]) / total_weighted_length
    edge_tabs[i] = (count as f64 * fraction).round() as usize

// Handle rounding remainder
allocated = sum(edge_tabs)
if allocated < count:
    // Distribute remaining tabs to highest weighted-length edges
    // that received fewer than their fractional share
    Sort edges by fractional_remainder descending
    Assign +1 tab to top (count - allocated) edges
if allocated > count:
    // Remove excess from lowest weighted-length edges
    Sort edges by weighted_length ascending
    Subtract 1 tab from lowest edges until allocated == count
```

**Step 4: Place tabs on edges**

For each edge with `edge_tabs[i] > 0`:
```
usable_start = corner_exclusion / edge_length[i]  // t parameter
usable_end = 1.0 - usable_start
usable_range = usable_end - usable_start
if usable_range <= 0.0: skip edge (should not happen due to min_length filter)

spacing = usable_range / (edge_tabs[i] + 1) as f64
for j in 1..=edge_tabs[i]:
    t = usable_start + spacing * j as f64
    position = point_on_edge(outer[i], outer[(i+1) % n], t)
    Emit Tab { position, edge_index: i, t, width: tab_width, height: tab_height }
```

**Step 5: Validate minimum spacing**

Walk all emitted tabs in contour order (sorted by `edge_index` then `t`). For any pair of adjacent tabs closer than `2 * tab_width` (measured as arc distance along contour), remove the tab with the lower edge score. When scores are equal, remove the tab with higher `edge_index` (stable tie-breaking).

**Step 6: Validate minimum count (recovery)**

If after spacing validation, `tabs.len() < config.min_tabs`:
1. Identify edges with no tabs, sorted by `edge_length * edge_score` descending.
2. For each such edge, attempt to place a tab at the edge midpoint, ignoring `corner_exclusion` (relaxed).
3. After each placement, re-check spacing constraint. Skip if too close to existing tab.
4. Stop when `tabs.len() >= config.min_tabs` or all edges exhausted.

If recovery still yields fewer than `min_tabs`, accept the result (geometric infeasibility). Do not panic.

**Step 7: Final sort**

Sort output tabs by `(edge_index, t)` for deterministic output ordering.

**`crates/thetta-geometry/src/lib.rs`** -- Add `pub mod tabs;`

#### Unit Tests (in `tabs.rs`)

| Test | Input | Expected |
|------|-------|----------|
| `test_rectangle_tabs` | 100x50mm rectangle (perimeter 300mm, distance 50) | 6 tabs, at least 1 per long edge |
| `test_circle_tabs` | Radius 50mm circle (32 segments, perimeter ~314mm) | Tabs distributed approximately uniformly |
| `test_l_shape_tabs` | L-shape | Concave corner edges get fewer/no tabs |
| `test_tiny_part_no_tabs` | 5x5mm rectangle (perimeter 20mm < 30mm threshold) | Empty result |
| `test_large_part_max_tabs` | Very large perimeter (10000mm) | Capped at `max_tabs` (20) |
| `test_min_tabs_enforced` | Shape with mostly concave edges | Still gets at least `min_tabs` (3) |
| `test_deterministic` | Same input twice | Identical output |
| `test_no_tabs_at_corners` | Any polygon | No tab `t` within `corner_exclusion / edge_length` of 0.0 or 1.0 |
| `test_tab_spacing` | Dense polygon | No two tabs closer than `2 * tab_width` along contour |
| `test_all_edges_too_short` | Polygon with all edges < `tab_width + 2 * corner_exclusion` | Empty result (no panic) |
| `test_material_sizing` | `MaterialType::WoodSoft`, thickness 18mm | Height ~7.2mm, width ~10.8mm |
| `test_material_sizing_hard_wood` | `MaterialType::WoodHard`, thickness 18mm | Height ~5.94mm, width ~8.91mm |

### Phase 2: App Integration -- State, UI Panel, and Tab Generation Trigger

**Goal**: Wire tab config into `AppState`, add UI controls, trigger tab generation on nesting completion.

#### Tab State

**`crates/thetta-app/src/state.rs`** -- Add:

```rust
/// Holds tab configuration and computed results.
pub struct TabState {
    /// User-configurable tab parameters.
    pub config: TabConfig,
    /// Computed tabs, keyed by (sheet_index, placement_index).
    /// placement_index is the index into Sheet.placements for that sheet.
    pub results: HashMap<(usize, usize), Vec<Tab>>,
}

impl Default for TabState {
    fn default() -> Self {
        Self {
            config: TabConfig::default(),
            results: HashMap::new(),
        }
    }
}
```

Add `pub tab_state: TabState` field to `AppState`.

**Tab generation method** on `AppState`:

```rust
impl AppState {
    /// Compute tabs for all placements in the current nesting result.
    pub fn generate_tabs(&mut self) {
        self.tab_state.results.clear();
        let Some(result) = &self.current_result else { return };
        let pieces = &self.pieces;

        for (sheet_idx, sheet) in result.sheets.iter().enumerate() {
            for (placement_idx, placement) in sheet.placements.iter().enumerate() {
                // Look up piece by piece_id
                let Some(piece) = pieces.iter().find(|p| p.id == placement.piece_id) else {
                    continue;
                };

                // Apply placement transform to get world-space contour
                let center = Point2D { x: 0.0, y: 0.0 };
                let rotated = polygon_ops::rotate(&piece.polygon, placement.rotation, center);
                let placed = polygon_ops::translate(&rotated, placement.x, placement.y);

                // Generate tabs
                let tabs = tabs::generate_tabs(&placed, &self.tab_state.config);
                if !tabs.is_empty() {
                    self.tab_state.results.insert((sheet_idx, placement_idx), tabs);
                }
            }
        }
    }
}
```

**Trigger tab generation**:
- In `poll_updates`, when receiving `NestingUpdate::Finished(result)`: after storing the result, call `self.generate_tabs()`.
- Clear `tab_state.results` when nesting restarts (in `start_optimizer`).
- Clear `tab_state.results` when pieces change (delete, new import).

#### UI Panel

**`crates/thetta-app/src/panels.rs`** -- Add "Tabs" section after nesting controls:

Controls:
- `material_type` dropdown: WoodSoft, WoodHard, Plastic, Composite
- `material_thickness` slider (1.0-50.0mm, default 18.0)
- `auto_size` checkbox (default true). When checked, show computed width/height as read-only labels.
- `tab_width` slider (3.0-30.0mm) -- only editable when `auto_size` is false
- `tab_height` slider (1.0-20.0mm) -- only editable when `auto_size` is false
- `tab_distance` slider (10.0-200.0mm, default 50.0)
- `min_tabs` / `max_tabs` integer inputs (1-50 range)
- `corner_exclusion` slider (0.0-20.0mm, default 5.0)
- `size_threshold` slider (0.0-100.0mm, default 30.0)
- "Regenerate Tabs" button
- Status line: "N tabs across M pieces (K pieces skipped)"

Return `PanelAction::RegenerateTabs` when:
- "Regenerate Tabs" button clicked
- Any slider value changes (on release, not during drag)

**`crates/thetta-app/src/main.rs`** -- Handle `PanelAction::RegenerateTabs`:
- Call `app_state.generate_tabs()`

### Phase 3: Viewport Rendering and Export

**Goal**: Render tabs visually and include tab data in JSON export.

#### Viewport Rendering

**`crates/thetta-app/src/viewport.rs`** -- Add `draw_tabs` function, called after piece contours are drawn:

```rust
fn draw_tabs(
    painter: &egui::Painter,
    tab_state: &TabState,
    sheet_idx: usize,
    sheet: &Sheet,
    x_offset: f32,
    to_screen: &emath::RectTransform,
) {
    let tab_color = Color32::from_rgba_premultiplied(255, 165, 0, 200); // orange, slight transparency

    for (placement_idx, _placement) in sheet.placements.iter().enumerate() {
        let Some(tabs) = tab_state.results.get(&(sheet_idx, placement_idx)) else {
            continue;
        };

        for tab in tabs {
            // Draw tab as a small filled rectangle centered on tab.position
            // oriented along the edge, with width = tab.width in world space
            // Visual indicator size: min(tab.width, 4.0 / zoom_scale) to remain visible at low zoom
            let center = Pos2::new(
                tab.position.x as f32 + x_offset,
                tab.position.y as f32,
            );
            let half_w = (tab.width as f32 / 2.0).max(2.0);
            let half_h = half_w * 0.5; // visual aspect ratio

            let rect = Rect::from_center_size(
                to_screen.transform_pos(center),
                Vec2::new(
                    half_w * 2.0 * to_screen.scale().x,
                    half_h * 2.0 * to_screen.scale().y,
                ),
            );

            painter.rect_filled(rect, 1.0, tab_color);
        }
    }
}
```

Call `draw_tabs` in the per-sheet rendering loop, after drawing piece polygons.

Tab rendering requirements:
- Only render when `tab_state.results` is non-empty
- Respects viewport zoom and pan (uses same world-to-screen transform as pieces)
- Uses distinct color (orange `Color32::from_rgb(255, 165, 0)`) with slight transparency
- Minimum visual size so tabs remain visible at low zoom levels

#### JSON Export

**`crates/thetta-step/src/export.rs`** -- Modify `export_sheet_json` to accept optional tab data:

```rust
pub fn export_sheet_json(
    // ... existing parameters ...
    tabs: Option<&HashMap<(usize, usize), Vec<Tab>>>,
    sheet_index: usize,
) -> Result<(), StepExportError> {
    // ... existing JSON serialization ...

    // If tabs provided, add tab metadata to JSON output
    if let Some(tab_map) = tabs {
        // Serialize tabs for this sheet's placements
        let sheet_tabs: Vec<_> = sheet.placements.iter().enumerate()
            .filter_map(|(placement_idx, placement)| {
                tab_map.get(&(sheet_index, placement_idx)).map(|tabs| {
                    serde_json::json!({
                        "piece_id": placement.piece_id.0,
                        "placement_index": placement_idx,
                        "tabs": tabs.iter().map(|t| serde_json::json!({
                            "x": t.position.x,
                            "y": t.position.y,
                            "edge_index": t.edge_index,
                            "t": t.t,
                            "width": t.width,
                            "height": t.height,
                        })).collect::<Vec<_>>(),
                    })
                })
            })
            .collect();

        // Add "tabs" key to the JSON output object
        // ...
    }
}
```

**`crates/thetta-app/src/state.rs`** -- Pass `tab_state.results` to export functions when triggering export. Use `Option<&HashMap<...>>` so `None` preserves existing behavior with no tabs.

STEP (OCCT) export remains unchanged -- tab geometry in STEP is deferred.

## Files Summary

### New Files

| File | Purpose | Est. LOC |
|------|---------|----------|
| `crates/thetta-geometry/src/tabs.rs` | Core tab placement algorithm + unit tests | ~400 |

### Modified Files

| File | Changes | Est. Delta |
|------|---------|------------|
| `crates/thetta-geometry/src/types.rs` | Add `Tab`, `TabConfig`, `MaterialType` types | +100 |
| `crates/thetta-geometry/src/polygon_ops.rs` | Add `perimeter`, `edge_length`, `point_on_edge` | +40 |
| `crates/thetta-geometry/src/lib.rs` | Add `pub mod tabs;` | +1 |
| `crates/thetta-app/src/state.rs` | Add `TabState`, `generate_tabs()`, trigger logic | +80 |
| `crates/thetta-app/src/panels.rs` | Add tab configuration UI section | +60 |
| `crates/thetta-app/src/viewport.rs` | Add tab rendering overlay | +50 |
| `crates/thetta-step/src/export.rs` | Include tab metadata in JSON export | +40 |
| **Total** | | **~770** |

## Definition of Done

### P0 -- Must Ship

- [ ] `Tab`, `TabConfig`, `MaterialType` types exist in `thetta-geometry/src/types.rs` with `Serialize`/`Deserialize` derives
- [ ] `TabConfig::default()` returns documented defaults (distance=50, min=3, max=20, width=10, height=6, threshold=30, exclusion=5, thickness=18, material=WoodSoft, auto_size=true)
- [ ] `MaterialType` enum has variants: `WoodSoft`, `WoodHard`, `Plastic`, `Composite`
- [ ] `TabConfig::effective_tab_height()` returns correct values: WoodSoft 40%, WoodHard 33%, Plastic 33%, Composite 50% of `material_thickness`
- [ ] `TabConfig::effective_tab_width()` returns `clamp(1.5 * height, 5.0, 25.0)`
- [ ] `perimeter()` function returns correct values for rectangle (300mm for 100x50), triangle, circle approximation
- [ ] `generate_tabs()` produces 0 tabs for parts below `size_threshold`
- [ ] `generate_tabs()` produces 0 tabs for polygons with fewer than 3 vertices
- [ ] `generate_tabs()` produces at least `min_tabs` tabs for any part above threshold with sufficient edge length
- [ ] `generate_tabs()` produces at most `max_tabs` tabs regardless of perimeter
- [ ] Tab count matches `clamp(round(perimeter / tab_distance), min, max)` for simple shapes
- [ ] No tab is placed within `corner_exclusion` distance of any polygon vertex
- [ ] No two tabs are closer than `2 * tab_width` along the contour
- [ ] Straight edges receive more tabs (proportionally) than concave edges for mixed-geometry shapes
- [ ] `generate_tabs()` is deterministic: identical inputs produce identical outputs (byte-for-byte Vec comparison)
- [ ] `generate_tabs()` returns empty Vec (no panic) when all edges are too short for tabs
- [ ] Min-count recovery path works: shape with mostly concave edges still gets `min_tabs` when geometrically feasible
- [ ] Spacing-conflict resolution removes lower-scored tab, with stable tie-breaking by `edge_index`
- [ ] All unit tests pass: rectangle, circle, L-shape, tiny part, large part, min-tabs, determinism, corners, spacing, all-edges-short, material sizing
- [ ] `TabState` with `config` and `results: HashMap<(usize, usize), Vec<Tab>>` exists in `AppState`
- [ ] Tabs auto-generate when `NestingUpdate::Finished` is received
- [ ] Tabs clear when nesting restarts or pieces change
- [ ] UI panel displays tab configuration controls (material type, thickness, distance, min/max, exclusion, threshold)
- [ ] Auto-size toggle switches between material-derived and manual width/height
- [ ] Changing any tab config value and clicking "Regenerate Tabs" triggers tab recomputation
- [ ] Tabs render in viewport as colored rectangles on piece edges
- [ ] Tab rendering respects viewport zoom and pan
- [ ] Multi-sheet results produce independent per-sheet tab data with no cross-sheet interference
- [ ] JSON export includes tab metadata when tabs are computed
- [ ] `cargo test --workspace` passes with no regressions
- [ ] `cargo clippy --workspace` reports no new warnings

### P1 -- Should Ship

- [ ] Tab status line in panel: "N tabs across M pieces (K pieces skipped)"
- [ ] Tabs visually distinguish from piece contour edges (orange color with transparency)
- [ ] Minimum visual tab size ensures visibility at low zoom levels
- [ ] `generate_tabs()` handles CCW outer ring assumption via assertion at function entry (debug_assert ring area > 0)
- [ ] Integration test: transform polygon by placement, generate tabs, verify tab positions are in world space
- [ ] Tab rendering does not interfere with piece click-to-select hit testing

### P2 -- Stretch

- [ ] Tab rendering shows edge direction indicator (small arrow along edge at tab position)
- [ ] Tab count per piece shown on hover in viewport
- [ ] Per-piece tab override (click piece, adjust tab count for that piece only)
- [ ] Performance: tab generation for 100 pieces completes in < 50ms

### Cut if Behind Schedule

- JSON export tab metadata (Phase 3 export) -- tabs still visible in viewport, just not exported
- Auto-size material mode -- use manual sliders only
- Tab status line in panel -- tabs still render, just no summary text

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Edge classification incorrect for complex concave polygons | Medium | Medium | Use cross-product sign at each vertex; test with L-shape and T-shape fixtures; add visual debug mode showing edge scores |
| Rounding in tab distribution leaves edges with 0 tabs when they should have 1 | Medium | Low | Explicit remainder distribution: after initial round(), distribute leftovers to highest-fractional-remainder edges |
| All-edges-too-short for densely-segmented polygons (e.g., high-resolution circle) | Medium | Medium | Return empty Vec with no panic; `total_weighted_length <= 0.0` guard catches this case |
| World-space transform incorrect (rotation center mismatch) | Medium | High | Integration test verifies tabs on transformed polygon match expected positions; rotate around origin (consistent with nesting engine convention) |
| `generate_tabs` called on untransformed polygon | Medium | High | Function docs clearly state input must be placed (world-space) contour; debug_assert on polygon centroid being in expected quadrant |
| Floating-point non-determinism across runs | Low | Medium | Use indexed Vec iteration only (no HashMap iteration for output-affecting logic); stable sort with explicit tie-breaking by `(edge_index, t)` |
| Export signature change breaks existing callers | Low | Medium | Make `tabs` parameter `Option` -- `None` preserves existing behavior |
| `tab_distance == 0` or NaN in config | Low | Medium | `effective_tab_distance()` clamps to minimum 1.0mm; all config values validated before use; NaN inputs produce empty tab list |
| Performance on large sheet with many pieces | Low | Low | Algorithm is O(edges) per piece -- negligible compared to nesting time |

## Security Considerations

- **No network access**: Tab algorithm is pure local computation. No external API calls.
- **No file system writes** beyond existing export paths: Tab data piggybacks on existing JSON export.
- **No unsafe code**: All new code is safe Rust. No FFI, no raw pointers.
- **No new crate dependencies**: Tab algorithm uses only `std` math and existing `thetta-geometry` types.
- **Input validation**: `TabConfig` values are clamped in the UI (sliders have min/max bounds). The algorithm handles degenerate inputs (empty polygon, zero perimeter, NaN coordinates, all-short-edges) by returning empty tab lists rather than panicking.
- **Determinism**: No use of `HashMap` iteration order for output-affecting logic. Tab generation uses indexed `Vec` iteration and explicit sort with stable tie-breaking.
- **Panic safety**: No `unwrap()` on user-controlled data. All fallible operations return empty results or use safe defaults.

## Dependencies

- **No new external crate dependencies.** The tab algorithm uses only `std` math and existing `thetta-geometry` types.
- **Internal dependencies unchanged**: `thetta-app` already depends on `thetta-geometry`. No new crate edges in the dependency graph.
- **Build**: No changes to any `Cargo.toml` dependency sections. Only `lib.rs` module declaration changes.
