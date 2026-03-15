# Sprint 006 Merge Notes

## Draft Strengths Adopted

### From Claude Draft
- **Geometric rigor for edge classification**: Explicit cross-product math for convex/concave vertex classification with clear formulas (`cross = (B-A) x (C-B)`, sign interpretation relative to CCW winding)
- **Detailed edge scoring approach**: Edge score as average of endpoint vertex classifications, with short-edge filter (`tab_width + 2 * corner_exclusion`)
- **Pure-geometry module architecture**: Clear rationale for placing tab algorithm in `thetta-geometry` with no nesting dependency
- **Comprehensive unit test matrix**: 9 specific test cases covering rectangle, circle, L-shape, tiny part, large part, min-tabs enforcement, determinism, corner avoidance, and spacing validation
- **Viewport rendering specification**: Specific color choice (orange/amber), transparency, world-to-screen transform integration
- **Use case depth**: 7 use cases including deterministic output guarantee and multi-sheet independence

### From Codex Draft
- **Per-placement indexing model**: `TabsForPlacement` with `sheet_index` and `placement_index` correctly handles multiple placements of the same `PieceId` -- critical fix over Claude's `HashMap<PieceId, Vec<Tab>>` which would overwrite duplicate placements
- **`MaterialType` enum with per-material sizing**: Faithfully implements Section 4.5 material-dependent tab height (wood_soft=0.40, wood_hard=0.33, plastic=0.33, composite=0.50) and width formula (`clamp(1.5 * tab_height, 5.0, 25.0)`)
- **Determinism awareness**: Explicit plan for stable sorting keys (`edge_index`, `t`) to prevent floating-point tie-breaker drift
- **Spacing conflict resolution**: Drop lower-score candidate first when tabs violate minimum spacing
- **Min-count recovery with corner-exclusion relaxation**: Explicit relaxation strategy when min_tabs cannot be satisfied under normal constraints
- **Comprehensive security section**: Addresses NaN coordinates, extreme config values, panic safety, and malformed polygon robustness

### From Gemini Draft
- **Streamlined 3-phase structure**: Core Geometry & Types -> App State & UI Integration -> Rendering & Export. Reduces coordination overhead compared to 5-phase plans
- **`tabs.rs` module name**: More idiomatic Rust than Codex's `tab_placement.rs`
- **No-regression DoD item**: "No regressions in existing nesting benchmarks or full pipeline tests"
- **Trigger strictly on `NestingUpdate::Finished`**: Correct scoping for Sprint 006; avoids recomputing tabs on every GLS iteration
- **Material-aware sizing as first-class feature**: Includes `MaterialType` in the core type set from the start

## Valid Critiques Accepted

| Critique | Source | Action |
|----------|--------|--------|
| Claude's `HashMap<PieceId, Vec<Tab>>` cannot distinguish multiple placements of same PieceId | Codex critique | Replaced with per-placement indexing: `HashMap<(usize, usize), Vec<Tab>>` keyed by `(sheet_index, placement_index)` |
| Missing winding order prerequisite for convex/concave classification | Claude critique, Codex critique | Added explicit winding order verification step; `Polygon.outer` is already documented as CCW in `types.rs`, so assert rather than normalize |
| All-edges-too-short fallback undefined | Claude critique | Defined behavior: return empty `Vec<Tab>` with no panic; log a warning. If perimeter exceeds threshold but all edges are too short, treat as geometric infeasibility |
| Rounding remainder in per-edge tab distribution | Claude critique | Added explicit remainder strategy: after initial `round()` allocation, distribute remaining tabs one-by-one to highest-weighted-length edges that received fewer than their fractional share |
| World-space transform validation needed | Codex critique | Added integration test that verifies tabs computed on a transformed polygon match expected world-space positions |
| Gemini's STEP geometry export is scope creep for Sprint 006 | Claude critique | STEP tab geometry deferred; Sprint 006 is JSON metadata only |
| Missing `size_threshold` in Gemini's algorithm description | Claude critique | Included as explicit early-return check in Step 1 |
| No test for min-count recovery and spacing-conflict resolution | Claude critique | Added specific DoD items and test cases |
| Material sizing formulas missing from Claude draft | Gemini critique, Claude critique of Codex | Included Section 4.5 formulas as `MaterialType` enum with concrete height/width derivation |
| `tab_distance == 0` or invalid slider states could panic | Gemini critique | Added input validation: clamp `tab_distance` to minimum 1.0mm in `TabConfig` validation |

## Critiques Rejected

| Critique | Source | Reasoning |
|----------|--------|-----------|
| Should tab inner contours (holes) for large cutouts | Gemini critique | Deferred to future sprint. Inner-contour tabs require different UX (user must identify which holes need tabs) and different sizing heuristics. Per-part outer-contour tabs is the correct Phase 1 scope. |
| Edge scoring should evaluate segment geometry, not average endpoint vertex scores | Gemini critique of Claude | For polyline contours (which is what Thetta uses), averaging endpoint vertex classifications is a reasonable approximation. A straight segment between two concave corners should still score lower because tab removal is harder near concave regions. The averaging approach is simpler and sufficient for Sprint 006. |
| Should compute tabs on `NestingUpdate::Improved` for intermediate visual feedback | Claude critique of Codex | Sprint 006 scope is `Finished` only. Computing on every `Improved` would trigger hundreds of recomputes during GLS. Users can click "Regenerate Tabs" manually if needed during optimization. |
| Collinear consecutive edges should be merged before scoring | Claude critique | Edge merging adds complexity and is unnecessary for typical STEP-imported polygons which already have clean edge segmentation. Collinear edges will both score ~1.0 (straight) and distribute tabs proportionally, which is correct behavior. |
| Should add `min_tabs` infeasibility handling when perimeter is geometrically too small | Gemini critique, Codex critique | Already handled by `size_threshold` early return. If perimeter exceeds threshold but edges are all too short, the all-edges-too-short fallback (return empty) applies. Adding partial-tab placement for barely-above-threshold parts adds complexity without practical value. |

## Interview Refinements

1. **Tab storage**: Per-placement indexing with `TabState { config: TabConfig, results: HashMap<(usize, usize), Vec<Tab>> }` stored in `AppState` separately from `NestingResult`. Do NOT modify `NestingResult` or `Sheet` types.
2. **Material sizing**: Include `MaterialType` enum with per-material height/width formulas from RESEARCH-005 Section 4.5.
3. **Tab shape**: Rectangular only for this sprint. Triangular/ramped deferred.
