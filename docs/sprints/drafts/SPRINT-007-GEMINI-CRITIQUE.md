# Sprint 007 Draft Critique: G-code Profile Cuts with Tool Database

## Overview
This document provides a comparative critique of the Claude and Codex drafts for Sprint 007, evaluating them against the stated intent of implementing G-code profile cuts and a LinuxCNC-compatible tool database.

---

## Claude Draft (`SPRINT-007-CLAUDE-DRAFT.md`)

### Strengths
* **Architectural Clarity:** The proposal to create a dedicated `thetta-cam` crate is excellent. It correctly identifies that CAM concerns (G-code text emission) are fundamentally different from 3D solid geometry (STEP I/O), avoiding pollution of the `thetta-step` crate.
* **Algorithmic Depth:** Provides a detailed, step-by-step algorithm for toolpath generation (`walk_edge_with_tabs`), explicitly handling the complexity of mapping tab arc-distances to edge segments and multi-pass step-downs.
* **Intermediate Representation:** Strongly justifies the `Toolpath` intermediate representation for testability and future post-processor flexibility.
* **Testing Strategy:** Includes specific, well-thought-out unit and integration tests (e.g., `test_tab_z_clamped_above_zero`, `test_rectangle_with_one_tab`).

### Weaknesses
* **Complexity:** Introduces a new workspace crate, which slightly increases build configuration complexity, though the trade-off is well worth it.
* **Ramp Implementation Detail:** The assumption of ramping Z linearly over a "short distance (2mm)" is slightly arbitrary and might cause tool marks or breakage depending on the plunge feed rate and material.

### Gaps in Risk Analysis
* **UI State Sync:** Fails to deeply analyze the risk of UI state becoming desynchronized if the background tool database save operation fails silently.
* **Tool Diameter Validation:** Doesn't consider the risk of a selected tool having a diameter larger than the cut geometry itself or wider than the space between two parts.

### Missing Edge Cases
* **Start Point on Tab:** What happens if the profile cut's starting vertex (`contour.outer[0]`) falls exactly within a tab's interval? The plunge move might incorrectly plunge into the tab.
* **Overlapping Tabs:** Behavior when tabs are placed too close together or overlap (e.g., resulting in a continuous tabbed section).

### Definition of Done Completeness
* **Very Complete:** The DoD is highly comprehensive, covering explicit type fields, specific testing criteria, UI panel states, file output requirements, and edge cases (empty sheets).

---

## Codex Draft (`SPRINT-007-CODEX-DRAFT.md`)

### Strengths
* **Safety in Persistence:** Specifically calls out an atomic temp-file write + rename pattern for saving the tool database, which is a robust way to prevent file corruption.
* **Directionality Awareness:** Acknowledges the need for `CutDirection` (`climb` vs `conventional`) as part of the `ProfileCutConfig`.
* **Minimalist Approach:** Attempts to minimize structural overhead by reusing existing crates.

### Weaknesses
* **Architectural Flaw (SRP Violation):** Forcing G-code emission and tool table persistence into `thetta-step` is a significant architectural misstep. `thetta-step` is bound to OpenCASCADE FFI; tying plain-text CAM generation to it conflates domains and harms testability.
* **Vague Implementation:** The algorithm for planning tab-aware lifts is described at a very high level ("Segment contour arcs/edges into alternating full-depth and tab-lift intervals") without concrete data structures or logic.
* **Lack of Concrete Tests:** The testing section lacks specific scenarios, relying on general statements like "planner correctness + determinism tests."

### Gaps in Risk Analysis
* **Build Coupling:** Fails to recognize the risk of coupling CAM logic with the `occt` feature flag in `thetta-step`, which could make testing CAM logic unnecessarily heavy.
* **Floating-Point Accumulation:** Misses the risk of floating-point inaccuracies when calculating Z depths across many step-down passes.

### Missing Edge Cases
* **Wrap-around Tabs:** Does not explicitly address how to handle a tab interval that straddles the start/end point of a closed contour.
* **Safe-Z clear height:** Doesn't consider cases where the material itself might have variations, and `safe_z` might need to be relative to the stock surface rather than absolute 0.

### Definition of Done Completeness
* **Incomplete:** The DoD lacks specificity. It covers the high-level features but misses granular requirements for the G-code output format (e.g., precision, modality), specific UI feedback mechanisms (e.g., status messages), and concrete test coverage targets.

---

## Conclusion and Recommendation
The **Claude Draft** is significantly stronger. Its architectural decision to create `thetta-cam` is correct, its algorithmic design is much more concrete, and its DoD is highly actionable. The Codex draft's suggestion to put CAM in `thetta-step` should be rejected. However, the final implementation should incorporate Codex's suggestion for atomic file writes for the tool database.