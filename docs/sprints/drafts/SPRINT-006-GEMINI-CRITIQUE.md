# Sprint 006 Draft Critique: Automatic Per-Part Tab Placement

This document provides a critique of the Claude and Codex drafts for SPRINT-006, evaluating their alignment with `RESEARCH-005` Section 4, architectural decisions, risk analysis, and Definition of Done (DoD).

## 1. Alignment with RESEARCH-005 Section 4

`RESEARCH-005` Section 4 defines a specific 5-step algorithm for tab placement, along with a material-dependent sizing heuristic (Section 4.5).
- **Core Algorithm (Steps 1-5):** Both drafts successfully capture the core mechanics: perimeter-based count, edge classification (straight=1.0, convex=0.8, concave=0.3), proportional distribution, spacing validation, and min-count recovery.
- **Tab Sizing (Section 4.5):** **Codex** correctly implements the material-based sizing (`tab_height` as a percentage of `material_thickness`, `tab_width = 1.5 * height`). **Claude** explicitly defers this to an "Open Question" and relies on static sliders with defaults, missing a key domain-specific heuristic from the research.

---

## 2. Claude Draft Critique

### Strengths
- **Geometric Rigor:** Provides clear, implementable math for edge classification (using cross-product signs at vertices).
- **Architecture:** Excellent separation of concerns, explicitly defining `thetta-geometry::tabs` as a pure math module that operates on placed contours.
- **UI/UX Detail:** Highly detailed breakdown of the UI panel controls and viewport rendering (including specific visual indicators like transparency and color).

### Weaknesses
- **Missed Requirements:** Fails to implement Section 4.5 (Material-based Tab Sizing). By deferring `MaterialType` and `material_thickness`, the draft forces the user to manually guess tab dimensions instead of relying on the research-backed 33%-50% thickness rule.
- **Edge Scoring Flaw:** Suggests averaging the scores of a segment's endpoint vertices. This can lead to a straight segment between two concave corners being scored as concave, rather than evaluating the segment's actual geometry.

### Gaps in Risk Analysis
- **Impossible Spacing:** Does not address the failure mode where `min_tabs` physically cannot fit on a part that barely exceeds `size_threshold` (e.g., perimeter is 31mm, but 3 tabs of 10mm + spacing requires >60mm).
- **Divide by Zero:** Fails to mention the risk of `tab_distance == 0` or invalid UI slider states causing panics.

### Missing Edge Cases
- **Inner Contours (Holes):** Assumes tabs only go on the outer ring. Large internal cutouts (e.g., a large window hole) often require tabs to prevent the scrap piece from flying out and damaging the tool.
- **Degenerate/Self-Intersecting Polygons:** No mention of how the algorithm behaves if the input contour is not a simple, valid polygon.

### Definition of Done Completeness
- Very thorough for the scope it defined, but fundamentally incomplete relative to the research because it lacks tests and criteria for material-dependent tab sizing.

---

## 3. Codex Draft Critique

### Strengths
- **Research Accuracy:** Faithfully includes the `MaterialType`, `material_thickness`, and the derived `1.5x` width ratio from Section 4.5.
- **Determinism Awareness:** Correctly identifies the risk of floating-point drift and explicitly plans for stable sorting keys (e.g., `edge_index`, `t`), which is critical for reproducible CAM outputs.
- **Data Flow:** Clearly diagrams the pipeline and correctly places tab generation as a post-nesting stage.

### Weaknesses
- **Vague Geometry:** Less specific than Claude on how to actually implement convex/concave classification and corner distance exclusion.
- **Incomplete UI Spec:** Adds `MaterialType` to the config but doesn't explain how the UI panel will expose it (e.g., an enum dropdown, thickness input) alongside the other sliders.

### Gaps in Risk Analysis
- **Recovery Failure:** Mentions "min-count recovery path" but doesn't analyze the risk of total recovery failure (e.g., what happens if relaxing corner exclusion still doesn't yield enough valid edge space?).
- **Viewport Performance:** Mentions "UI re-computation overhead" but doesn't address the specific rendering cost of thousands of tab markers in the viewport for dense, multi-sheet jobs.

### Missing Edge Cases
- **Inner Contours:** Like Claude, it explicitly restricts tabs to the outer contour, ignoring large scrap cutouts.
- **Zero-Thickness:** Doesn't handle the edge case where `material_thickness` is set to `0.0` or a negative value in the UI.

### Definition of Done Completeness
- Solid and covers the material types, but lacks explicit UI criteria for the material/thickness inputs and lacks specific visual verification steps for corner exclusion in the viewport.

---

## 4. Synthesis & Recommendations for Final Sprint Plan

The final sprint plan should merge the strengths of both drafts:

1. **Adopt Claude's Architectural Rigor:** Use Claude's pure-geometry module structure (`thetta-geometry::tabs`), explicit cross-product math, and detailed UI/Viewport rendering specifications.
2. **Mandate Codex's Material Sizing:** Implement Section 4.5 exactly as Codex proposed (`MaterialType`, `material_thickness`, and derived sizes). Provide UI toggles to switch between "Auto (Material)" and "Manual" sizing.
3. **Address Inner Contours:** Update the scope to allow tabbing on inner contours if their perimeter exceeds a distinct (likely larger) `inner_size_threshold`.
4. **Define Impossible Geometries:** Add explicit fallback logic (e.g., a "best effort" warning or reducing the `min_tabs` constraint) when `min_tabs` mathematically cannot fit on a part without overlapping.
5. **Ensure Determinism:** Enforce Codex's recommendation for stable float-sorting keys to guarantee identical tab placements across runs.