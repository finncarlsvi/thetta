# Sprint 006: Automatic Per-Part Tab Placement for CNC Routing

**Agent**: Claude
**Date**: 2026-03-15
**Based on**: SPRINT-006-INTENT, RESEARCH-005 Section 4, ARCHITECTURE.md

---

## Overview

This sprint adds automatic tab placement as a post-processing step after nesting. Given a `NestingResult` with placed pieces, the algorithm computes a set of holding tabs for each piece's outer contour. Tabs are small uncut material bridges that keep CNC-routed parts attached to the stock sheet during cutting — preventing part ejection, rotation, and tool damage.

The algorithm implements the composite approach from RESEARCH-005 Section 4: CamBam's perimeter-proportional count formula, FreeCAD's edge-length-weighted distribution, and community-derived corner-avoidance rules. Tab count is computed as `clamp(round(perimeter / tab_distance), min_tabs, max_tabs)` with a minimum of 3 tabs per part to prevent rotation. Tabs are distributed across edges proportional to `edge_length * edge_score`, where straight edges score highest and concave curves score lowest.

The implementation spans three crates: new types in `thetta-geometry`, a new `tabs` module in `thetta-geometry` for the pure-geometry algorithm, and UI integration in `thetta-app` for configuration controls and viewport rendering. Shared/adjacent tabs between neighboring parts are explicitly deferred to a future sprint. This sprint targets per-part tabs only.

---

## Use Cases

1. **Automatic tab placement on nesting output** — User imports parts, runs nesting, and clicks "Generate Tabs" (or it runs automatically). Each placed piece receives 3–20 tabs distributed along its outer contour. No manual intervention required for typical parts.

2. **Configurable tab parameters** — User adjusts tab spacing distance, min/max count, tab width, tab height, and corner exclusion distance via the UI panel. Changes trigger re-computation with immediate visual feedback.

3. **Small part handling** — Parts with perimeter below a configurable threshold (default 30mm) are automatically skipped. The user sees a visual indicator or status note that these parts received no tabs.

4. **Visual verification in viewport** — Tabs render as small colored rectangles on piece edges in the viewport. The user can visually verify placement before export. Tabs are drawn on top of piece contours with distinct coloring.

5. **Multi-sheet tab placement** — Tabs are computed independently per sheet, per piece. Multi-sheet results produce per-sheet tab data with no cross-sheet dependencies.

6. **Deterministic output** — Same piece geometry + same tab config always produces the same tab positions. No randomness in the algorithm. This ensures reproducible CNC output across runs.

7. **Export with tab metadata** — Tab data is included in JSON export as structured metadata alongside placement data, enabling downstream toolpath generators to incorporate tabs.

---

## Architecture

### Component Overview

```
┌─────────────────────────────────────────────────────────┐
│                     thetta-app                          │
│  ┌──────────┐  ┌───────────┐  ┌──────────────────────┐ │
│  │ panels.rs│  │viewport.rs│  │      state.rs        │ │
│  │ Tab UI   │  │ Tab render│  │ TabConfig + TabResult │ │
│  │ controls │  │ overlays  │  │ storage + trigger     │ │
│  └────┬─────┘  └─────┬─────┘  └──────────┬───────────┘ │
│       │              │                    │             │
└───────┼──────────────┼────────────────────┼─────────────┘
        │              │                    │
        │              │        ┌───────────▼───────────┐
        │              │        │   thetta-geometry      │
        │              │        │  ┌─────────────────┐  │
        │              └────────┤  │    types.rs      │  │
        │                       │  │ Tab, TabConfig,  │  │
        │                       │  │ TabResult        │  │
        │                       │  └─────────────────┘  │
        │                       │  ┌─────────────────┐  │
        └───────────────────────┤  │    tabs.rs       │  │
                                │  │ generate_tabs()  │  │
                                │  │ Edge scoring,    │  │
                                │  │ distribution,    │  │
                                │  │ validation       │  │
                                │  └─────────────────┘  │
                                │  ┌─────────────────┐  │
                                │  │ polygon_ops.rs   │  │
                                │  │ + perimeter()    │  │
                                │  │ + edge helpers   │  │
                                │  └─────────────────┘  │
                                └───────────────────────┘
```

### Data Flow

```
NestingResult (sheets with placements)
        │
        ▼
For each Sheet:
  For each Placement:
    │
    ├─ 1. Look up Piece by piece_id
    ├─ 2. Apply placement transform (translate + rotate) to get placed contour
    ├─ 3. Call generate_tabs(placed_contour, tab_config) → Vec<Tab>
    │       │
    │       ├─ Compute perimeter
    │       ├─ Compute tab count via clamp(round(perim/distance), min, max)
    │       ├─ Score each edge (straight=1.0, convex=0.8, concave=0.3)
    │       ├─ Distribute tabs proportional to edge_length * score
    │       ├─ Validate minimum spacing (no overlap)
    │       └─ Validate minimum count (force-place if needed)
    │
    └─ 4. Collect into TabResult { piece_tabs: HashMap<PieceId, Vec<Tab>> }
```

### Module Placement Rationale

The tab algorithm is **pure geometry** — it takes a polygon and config, returns tab positions. It has no dependency on nesting internals, collision detection, or jagua-rs. Therefore it belongs in `thetta-geometry`, not `thetta-nesting`. This keeps the leaf crate's role as the canonical geometry library and avoids adding a dependency from `thetta-nesting` back into `thetta-geometry`.

The app layer (`thetta-app`) handles orchestration: it owns `TabConfig` in `AppState`, triggers tab generation after nesting completes, stores the `TabResult`, renders tabs in the viewport, and exposes configuration UI.

### Edge Classification

Edges are classified by the turning angle at each vertex of the polygon contour:

```
For vertices A → B → C:
  cross = (B-A) × (C-B)

  If polygon is CCW (outer contour):
    cross > 0  →  convex turn (outside curve)
    cross < 0  →  concave turn (inside curve)
    cross ≈ 0  →  straight / collinear

  Edge B→C inherits the classification of vertex B
  Edge score is the average of its two endpoint vertex classifications
```

Short edges (length < `tab_width + 2 * corner_exclusion`) receive score 0.0 and are skipped entirely.

---

## Implementation

### Phase 1: Core Types and Geometry Utilities

**Goal**: Add new types and geometry helpers needed by the tab algorithm.

#### Files and Tasks

**`crates/thetta-geometry/src/types.rs`**
- Add `Tab` struct: `{ position: Point2D, edge_index: usize, t: f64, width: f64, height: f64 }`
  - `position`: world-space point on the contour edge
  - `edge_index`: index of the edge in the polygon's outer ring
  - `t`: parameter along the edge (0.0 = start vertex, 1.0 = end vertex)
  - `width`: tab width in mm along cutting direction
  - `height`: tab height in mm from cut bottom
- Add `TabConfig` struct with all configurable parameters:
  - `tab_distance: f64` (default 50.0)
  - `min_tabs: usize` (default 3)
  - `max_tabs: usize` (default 20)
  - `tab_width: f64` (default 10.0)
  - `tab_height: f64` (default 6.0)
  - `size_threshold: f64` (default 30.0)
  - `corner_exclusion: f64` (default 5.0)
- Add `TabResult` struct: `{ piece_tabs: HashMap<PieceId, Vec<Tab>> }`
- Derive `Debug, Clone, Serialize, Deserialize` on all new types
- Implement `Default` for `TabConfig` with the above defaults

**`crates/thetta-geometry/src/polygon_ops.rs`**
- Add `perimeter(poly: &Polygon) -> f64`: sum of edge lengths of outer contour
- Add `edge_length(a: Point2D, b: Point2D) -> f64`: Euclidean distance helper
- Add `point_on_edge(a: Point2D, b: Point2D, t: f64) -> Point2D`: linear interpolation
- Add unit tests for `perimeter` (rectangle, triangle, circle)
- Add unit tests for `point_on_edge` (midpoint, endpoints)

### Phase 2: Tab Placement Algorithm

**Goal**: Implement the core tab generation algorithm from RESEARCH-005 Section 4.

#### Files and Tasks

**`crates/thetta-geometry/src/tabs.rs`** (new file)
- Implement `generate_tabs(contour: &Polygon, config: &TabConfig) -> Vec<Tab>`
  - Step 1: Compute perimeter, early-return empty if below `size_threshold`
  - Step 2: Compute tab count via `clamp(round(perimeter / tab_distance), min_tabs, max_tabs)`
  - Step 3: Build edge list with lengths and scores (see edge classification)
  - Step 4: Distribute tabs proportional to `edge_length * edge_score`
  - Step 5: Validate minimum spacing — remove tabs closer than `tab_width * 2`
  - Step 6: Validate minimum count — force-place on best edges if under `min_tabs`
- Implement `classify_vertex(prev: Point2D, curr: Point2D, next: Point2D) -> f64`
  - Returns score: 1.0 for convex, 0.8 for near-straight, 0.3 for concave
- Implement `score_edge(v_start_score: f64, v_end_score: f64, edge_length: f64, min_length: f64) -> f64`
  - Returns 0.0 if edge too short, otherwise average of vertex scores
- Implement `distribute_tabs_on_edge(edge_start: Point2D, edge_end: Point2D, edge_length: f64, count: usize, corner_exclusion: f64) -> Vec<(f64, Point2D)>`
  - Places `count` tabs evenly along the edge, respecting corner exclusion zones
  - Returns list of `(t_parameter, world_position)` pairs

**`crates/thetta-geometry/src/lib.rs`**
- Add `pub mod tabs;` declaration

#### Unit Tests (in `tabs.rs`)

- `test_rectangle_tabs`: 100x50mm rectangle → expect 3 tabs (perimeter 300mm / 50mm distance = 6, but edges are short enough that 3–6 tabs placed)
- `test_circle_tabs`: radius 50mm circle (32 segments) → expect tabs distributed approximately uniformly
- `test_l_shape_tabs`: L-shape → concave corner edges get fewer/no tabs
- `test_tiny_part_no_tabs`: 5x5mm rectangle (perimeter 20mm < 30mm threshold) → empty result
- `test_large_part_max_tabs`: very large perimeter → capped at `max_tabs`
- `test_min_tabs_enforced`: contrived shape with mostly concave edges → still gets `min_tabs`
- `test_deterministic`: same input twice → identical output
- `test_no_tabs_at_corners`: verify no tab `t` parameter is within `corner_exclusion / edge_length` of 0.0 or 1.0
- `test_tab_spacing`: verify no two tabs are closer than `tab_width * 2` (measuring contour distance)

### Phase 3: App Integration — State and UI Panel

**Goal**: Wire tab config into `AppState`, add UI controls, trigger tab generation.

#### Files and Tasks

**`crates/thetta-app/src/state.rs`**
- Add `tab_config: TabConfig` field to `AppState`
- Add `tab_result: Option<TabResult>` field to `AppState`
- Add `generate_tabs(&mut self, pieces: &[Piece])` method:
  - Iterates `self.nesting_result.sheets`
  - For each placement, looks up piece, applies transform, calls `generate_tabs`
  - Stores result in `self.tab_result`
- Call `generate_tabs` automatically when nesting finishes (in `poll_updates` when receiving `NestingUpdate::Finished`)
- Clear `tab_result` when nesting restarts or pieces change

**`crates/thetta-app/src/panels.rs`**
- Add "Tabs" section to the control panel (after nesting controls)
- Controls:
  - `tab_distance` slider (10.0–200.0mm, default 50.0)
  - `min_tabs` / `max_tabs` integer inputs
  - `tab_width` slider (3.0–30.0mm)
  - `tab_height` slider (1.0–20.0mm)
  - `corner_exclusion` slider (0.0–20.0mm)
  - `size_threshold` slider (0.0–100.0mm)
  - "Regenerate Tabs" button
- Tab status display: "N tabs across M pieces (K pieces skipped)"
- Return `PanelAction::RegenerateTabs` when button clicked or slider changed

### Phase 4: Viewport Rendering

**Goal**: Render tabs as visual indicators on piece edges in the viewport.

#### Files and Tasks

**`crates/thetta-app/src/viewport.rs`**
- Add `draw_tabs` function called after piece contours are drawn
- For each tab in `TabResult`:
  - Compute tab rectangle in world space: centered on `tab.position`, oriented along the edge, width = `tab.width`, extending outward from edge by a small visual indicator size
  - Transform to screen space using existing viewport transform
  - Draw as filled rectangle with distinct color (e.g., orange/amber `Color32::from_rgb(255, 165, 0)`)
  - Draw with slight transparency so underlying contour remains visible
- Tabs only render when `tab_result` is `Some`
- Tab rendering respects zoom/pan (uses same world-to-screen transform as pieces)

### Phase 5: Export Integration

**Goal**: Include tab data in JSON export output.

#### Files and Tasks

**`crates/thetta-step/src/export.rs`**
- In `export_sheet_json`, include tab data in JSON output when available
- Add optional `tabs: Option<&TabResult>` parameter to export functions
- Tab data serialized as array of `{ piece_id, tabs: [{ x, y, edge_index, t, width, height }] }`

**`crates/thetta-app/src/state.rs`**
- Pass `tab_result` to export functions when triggering export

---

## Files Summary

| File | Action | Description |
|------|--------|-------------|
| `crates/thetta-geometry/src/types.rs` | Modify | Add `Tab`, `TabConfig`, `TabResult` types |
| `crates/thetta-geometry/src/polygon_ops.rs` | Modify | Add `perimeter`, `edge_length`, `point_on_edge` functions |
| `crates/thetta-geometry/src/tabs.rs` | Create | Core tab placement algorithm (~300 lines) |
| `crates/thetta-geometry/src/lib.rs` | Modify | Add `pub mod tabs;` |
| `crates/thetta-app/src/state.rs` | Modify | Add `TabConfig`/`TabResult` to state, generation trigger |
| `crates/thetta-app/src/panels.rs` | Modify | Add tab configuration UI section |
| `crates/thetta-app/src/viewport.rs` | Modify | Add tab rendering overlay |
| `crates/thetta-step/src/export.rs` | Modify | Include tab metadata in JSON export |

---

## Definition of Done

- [ ] `Tab`, `TabConfig`, `TabResult` types exist in `thetta-geometry/src/types.rs` with serde derives
- [ ] `TabConfig::default()` returns documented defaults (distance=50, min=3, max=20, width=10, height=6, threshold=30, exclusion=5)
- [ ] `perimeter()` function returns correct values for rectangle, triangle, circle test shapes
- [ ] `generate_tabs()` produces 0 tabs for parts below `size_threshold`
- [ ] `generate_tabs()` produces at least `min_tabs` tabs for any part above threshold
- [ ] `generate_tabs()` produces at most `max_tabs` tabs regardless of perimeter
- [ ] Tab count matches `clamp(round(perimeter / tab_distance), min, max)` for simple shapes
- [ ] No tab is placed within `corner_exclusion` distance of any polygon vertex
- [ ] No two tabs are closer than `tab_width * 2` along the contour
- [ ] Straight edges receive more tabs (proportionally) than concave edges
- [ ] `generate_tabs()` is deterministic: identical inputs produce identical outputs
- [ ] All unit tests pass: rectangle, circle, L-shape, tiny part, large part, min-tabs enforcement, determinism, corner avoidance, spacing
- [ ] UI panel displays tab configuration controls with sliders and inputs
- [ ] Changing any tab config slider triggers tab regeneration
- [ ] Tabs render in viewport as colored rectangles on piece edges
- [ ] Tab rendering respects viewport zoom and pan
- [ ] Tabs auto-generate when nesting finishes
- [ ] Tabs clear when nesting restarts
- [ ] Multi-sheet results produce independent per-sheet tab data
- [ ] JSON export includes tab metadata when tabs are computed
- [ ] `cargo test --workspace` passes with no regressions
- [ ] `cargo clippy --workspace` reports no new warnings

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Edge classification incorrect for complex concave polygons | Medium | Medium | Use cross-product sign at each vertex; test with L-shape and T-shape fixtures; add visual debug mode showing edge scores |
| Rounding in tab distribution leaves some edges with 0 tabs when they should have 1 | Medium | Low | After proportional allocation, sweep edges and assign remainders to highest-scored unassigned edges |
| Tab rectangles in viewport are hard to see at low zoom | Low | Low | Use distinct color (orange) with outline; scale minimum visual size with zoom |
| Very short edges (< tab_width) dominate polygon (e.g., discretized circle) | Medium | Medium | Score 0.0 for edges shorter than `tab_width + 2 * corner_exclusion`; distribute across longer segments |
| `generate_tabs` called on untransformed polygon instead of placed contour | Medium | High | Document clearly that input must be placed (world-space) contour; add assertion that checks contour is non-degenerate |
| Export signature change breaks existing callers | Low | Medium | Make `tabs` parameter `Option` — `None` preserves existing behavior |
| Performance on large sheet with many pieces | Low | Low | Algorithm is O(edges) per piece — negligible compared to nesting time. No mitigation needed. |

---

## Security Considerations

- **No network access**: Tab algorithm is pure local computation. No external API calls.
- **No file system writes** beyond existing export paths: Tab data piggybacks on existing JSON export.
- **No unsafe code**: All new code is safe Rust. No FFI, no raw pointers.
- **Input validation**: `TabConfig` values are clamped in the UI (sliders have min/max bounds). The algorithm handles degenerate inputs (empty polygon, zero perimeter, NaN coordinates) by returning empty tab lists rather than panicking.
- **Determinism**: No use of `HashMap` iteration order for output-affecting logic. Tab generation uses indexed `Vec` iteration only.

---

## Dependencies

- **No new external crate dependencies**. The tab algorithm uses only `std` math and existing `thetta-geometry` types.
- **Internal dependencies**: `thetta-app` already depends on `thetta-geometry` — no new crate edges.
- **Build**: No changes to `Cargo.toml` dependency sections. Only `lib.rs` module declarations change.

---

## Open Questions

1. **Tab storage location**: This draft stores tabs in a separate `TabResult` on `AppState` (keyed by `PieceId`). Alternative: add a `tabs: Vec<Tab>` field directly to `Sheet`. The separate structure avoids modifying the core `NestingResult`/`Sheet` types that nesting produces, keeping tab placement cleanly decoupled. **Recommendation**: Separate `TabResult` for Phase 1; consider merging into `Sheet` if export or downstream consumers need tighter coupling.

2. **Material type enum**: RESEARCH-005 proposes material-dependent tab height (40% for soft wood, 33% for hard wood/plastic, 50% for composite). This sprint uses a single configurable `tab_height` with a sensible default. Should a `MaterialType` enum be added now, or deferred until the sizing formula is validated against real CNC jobs? **Recommendation**: Defer — keep it simple with a single `tab_height` parameter. Users can adjust manually.

3. **Tab shape**: This sprint implements rectangular tabs only. Triangular and ramped/3D tabs require different geometry and G-code generation. **Recommendation**: Rectangular only for Sprint 006. Tab shape enum can be added later.

4. **STEP export**: Tabs as geometry in STEP output (extruded solids at tab positions) would be more useful than JSON metadata for some workflows, but requires OCCT geometry construction. **Recommendation**: JSON metadata for Sprint 006. STEP tab geometry is a separate effort.

5. **Default tab_distance**: 50mm default comes from RESEARCH-005's cross-domain synthesis (PCB 50–75mm, CamBam perimeter division). For typical 2440x1220mm wood sheets with parts in the 100–500mm range, this produces 3–10 tabs per part. Is this appropriate? **Recommendation**: Ship with 50mm default, collect user feedback.

6. **Auto-generate vs. manual trigger**: Should tabs generate automatically whenever nesting finishes, or only when the user clicks "Generate Tabs"? **Recommendation**: Auto-generate on nesting finish (can be toggled off). Users who don't need tabs can ignore them; users who do get immediate results.
