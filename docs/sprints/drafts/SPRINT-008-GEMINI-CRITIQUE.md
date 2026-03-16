# Critique of Sprint 008 Drafts

## Claude Draft Evaluation

### Strengths
- **Implementation Depth:** Provides a highly detailed architectural plan with specific module breakdowns (`hole_classify.rs`, `drill.rs`, `pocket.rs`). This makes the transition to implementation much smoother and prevents `toolpath.rs` from becoming bloated.
- **Algorithm Specifics:** Explicitly defines the mathematical heuristic for hole classification (circularity = 4π × area / perimeter²), providing a solid, testable starting point for development.
- **Clear Data Flow:** The step-by-step data flow execution diagram for `generate.rs` is excellent and directly addresses the complex multi-operation orchestration.
- **Addresses Open Questions:** Actively answers the open questions from the intent document (e.g., choosing per-sheet ordering, contour-parallel over zigzag, retract-rapid-plunge entry).

### Weaknesses
- **Pocket Entry Strategy:** The chosen "retract-rapid-plunge" between offset rings is functional but creates inefficient toolpaths with excessive Z-axis motion. While acknowledged as a P2 to improve to spiral connect, it might be too rudimentary even for a P0 CNC router application.
- **Hole within a Hole:** Explicitly defers "pocket island detection (holes within holes)" which could lead to tool crashes if a user imports complex STEP files where an inner hole dictates an island that shouldn't be milled.

### Gaps in Risk Analysis
- **Tool Diameter vs. Hole Size:** Does not adequately address the risk of the assigned drill bit being larger than the classified hole diameter, which would result in over-cutting the intended geometry. It briefly mentions warning the user in the mitigation section, but the system should actively prevent generating a plunge move in this scenario.
- **Deep Pockets:** Fails to address the risk of plunging a tool to the full `cut_depth` in a single pass for deep pockets, which can easily break endmills. The intent mentions multi-pass step-down is "Deferred", but this is a significant machine-safety risk.

### Missing Edge Cases
- **Overlapping/Intersecting Pockets:** Does not cover what happens if two inner contours overlap or if an offset ring intersects with another.
- **Open "Pockets":** Assumes all pockets are closed `Polygon.holes`. Doesn't address open-edge pockets (which might not be represented as holes in the current `Polygon` schema, but is a common CAM scenario).

### Definition of Done Completeness
- Very complete. Directly maps to all success criteria in the Intent document.
- Includes specific checks for G80 cancellation and tool radius compensation.

---

## Codex Draft Evaluation

### Strengths
- **Concise and High-Level:** Provides a good high-level overview without getting bogged down in the exact function signatures, allowing for flexibility during implementation.
- **Deduplication:** Uniquely identifies the need to "Deduplicate near-identical centers" during drill planning, which is a great catch for messy CAD imports.
- **Modal Safety:** Explicitly calls out modal safety and transition states when entering/leaving cycle mode in G-code, which is a critical CNC safety factor.
- **Backward Compatibility:** Specifically highlights the need for `serde` defaults and migration-compatible fields to prevent breaking existing user save files.

### Weaknesses
- **Lack of Modularity:** Proposes stuffing hole classification, drill planning, and pocket planning all into the existing `toolpath.rs` file, which will likely lead to a bloated and unmaintainable file.
- **Vague Classification:** Mentions "circularity score (perimeter/area derived)" but doesn't define the formula or threshold mechanism clearly.
- **Less Concrete App Integration:** The app integration and configuration changes are glossed over compared to the Claude draft.

### Gaps in Risk Analysis
- **Missing Tool Size Checks:** Similar to Claude, fails to highlight the critical risk of plunging an endmill or drill bit into a space smaller than its diameter.
- **Polygon Winding:** Does not mention the risk of incorrect polygon winding (CW vs CCW) from the STEP importer, which is crucial for offset algorithms (negative vs positive offset).

### Missing Edge Cases
- **Collinear/Degenerate Vertices in STEP:** Messy STEP imports often have collinear vertices or tiny zero-length segments that can crash offset algorithms.
- **Disabled Operations:** Barely touches on the edge case of a user disabling an operation and how that affects the generation pipeline and tool change logic.

### Definition of Done Completeness
- Adequate, but lacks the specificity of the Claude draft.
- Missing explicit validation of multi-sheet edge cases (e.g., sheet 1 has pockets, sheet 2 has only drills).

---

## Summary and Recommendation

The **Claude draft** is significantly stronger due to its concrete architectural breakdown, specific mathematical formulas, and thorough mapping to the intent document. It should be used as the primary basis for the sprint.

However, the implementation should incorporate the following insights from the **Codex draft**:
1. Implement duplicate hole center deduplication.
2. Ensure strict `serde` backward compatibility for existing config files.
3. Add robust modal safety checks in the G-code emitter tests.

**Critical Addition Needed for Both:**
Before development begins, the design MUST be updated to include a safety check that prevents toolpaths from being generated if the assigned tool diameter is larger than the hole/pocket minimum dimension. Plunging a 6mm tool into a 4mm hole is a severe hardware crash risk that neither draft adequately mitigated.