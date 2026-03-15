# Sprint 006 Draft Critique (Codex)

## Scope and Reference
This critique compares both sprint drafts against **RESEARCH-005 Section 4 (Proposed Algorithm for Thetta)**, focusing on:
- strengths
- weaknesses
- gaps in risk analysis
- missing edge cases
- Definition of Done (DoD) completeness

---

## Draft A: `SPRINT-006-CLAUDE-DRAFT.md`

### Strengths
- Strong alignment with Section 4 core flow: perimeter-based count, weighted edge scoring, spacing validation, minimum-count backfill.
- Detailed implementation plan by phase, with concrete files/functions/tests.
- Determinism is explicitly called out and tested.
- DoD is specific and measurable, with good coverage of algorithm and UI behavior.
- Explicitly defers shared/adjacent tabs (consistent with research Phase 1 scope) while acknowledging future need.

### Weaknesses
- **Data model likely wrong for repeated placements**: `TabResult { piece_tabs: HashMap<PieceId, Vec<Tab>> }` cannot distinguish multiple placements of the same `PieceId` on one or more sheets. Later placements can overwrite earlier ones or become ambiguous.
- Material-aware sizing from Section 4 is reduced to a fixed `tab_height` default (6.0) and deferred `MaterialType`; this is a notable deviation from research input/heuristics.
- Assumes polygon-only edge treatment; Section 4 input references line/arc edges. Arc handling strategy is not defined.
- "Force-place if needed" could violate exclusion/spacing constraints without clearly defining precedence rules.

### Gaps in Risk Analysis
- No explicit risk for repeated-part identity collisions (`PieceId` keying issue).
- No explicit risk for winding/orientation mistakes (CW vs CCW) causing convex/concave inversion.
- No explicit risk for invalid geometry classes: self-intersections, duplicate vertices, zero-length edges, open contours.
- No export contract/versioning risk (schema change impact on downstream consumers).
- Shared-tab deferral is stated, but production consequence risk (tab destruction on adjacent cuts) is underweighted.

### Missing Edge Cases
- Multiple placements of the same part ID across one sheet and across multiple sheets.
- Parts with inner holes: confirmation that tabs only target outer contour.
- CW-oriented contours vs CCW-oriented contours.
- Very dense segmented curves (many short edges) where all candidate edges may be excluded.
- Cases where constraints are mutually unsatisfiable (`min_tabs` impossible given width/exclusion/perimeter).
- Degenerate and invalid polygons (zero area, repeated points, self-intersection).

### DoD Completeness
- Overall strong and testable.
- Missing explicit acceptance criteria for:
  - repeated-instance correctness (no overwrite/ambiguity)
  - behavior when constraints cannot all be satisfied (defined fallback)
  - contour orientation normalization/handling
  - outer-vs-inner contour handling
  - export schema compatibility/versioning expectations

### Alignment to RESEARCH-005 Section 4
- **High** on Steps 1-4 and deterministic implementation details.
- **Partial** on Step 5 (force-placement policy is underspecified).
- **Partial/Low** on Section 4.1 and 4.5 material input/sizing (explicitly deferred).

---

## Draft B: `SPRINT-006-GEMINI-DRAFT.md`

### Strengths
- Clear high-level architecture split and concise phased plan.
- Includes a useful risk table and calls out performance/CAM compatibility early.
- Preserves post-processing separation from nesting core, aligned with research intent.
- Includes no-regression criterion in DoD, which is operationally valuable.

### Weaknesses
- Much less concrete than Claude draft: fewer exact algorithms, no detailed scoring/distribution rules, and no test-level specificity.
- Mentions material-aware sizing and `MaterialType`, but does not define concrete formulas/defaults matching Section 4.5.
- Ambiguous STEP scope ("metadata or visual geometry") risks sprint overreach and unclear acceptance.
- Uses broad language like "optimal" without objective criteria.

### Gaps in Risk Analysis
- Missing geometry-robustness risks (orientation, self-intersections, degenerate edges, arc discretization artifacts).
- Missing data-model/identity risks for repeated part instances.
- Missing deterministic-output risk tied to map iteration/order and rounding ties.
- Missing explicit risk for unsatisfiable constraints (`min_tabs` vs spacing/exclusion).
- Missing explicit acknowledgment of shared-tab failure mode consequences while deferring feature.

### Missing Edge Cases
- Repeated placements of the same part ID.
- Tiny-but-complex parts above threshold with no valid candidate edges.
- Parts with holes / multi-contour geometry.
- CW contour orientation and convex/concave misclassification.
- Highly concave/star-shaped parts with centroid/projection ambiguities.
- Numerical boundary cases around threshold/equality (`perimeter == size_threshold`, exact exclusion boundaries).

### DoD Completeness
- DoD is serviceable but mostly high-level.
- Lacks measurable criteria for:
  - exact count formula conformance (`clamp(round(perimeter/tab_distance), min, max)`)
  - spacing and corner exclusion invariants
  - deterministic reproducibility tests
  - multi-sheet + repeated-instance correctness
  - explicit fallback behavior when constraints conflict

### Alignment to RESEARCH-005 Section 4
- **Medium** overall: intent matches, but algorithmic specifics are underdefined.
- **Better than Claude** on acknowledging material-aware direction, but still incomplete due to missing concrete formulas and acceptance checks.

---

## Side-by-Side Verdict
- **Claude draft** is the stronger execution plan for immediate implementation due to specificity, testability, and clearer integration detail.
- **Gemini draft** is a solid product-level outline but under-specified for engineering execution and verification.

## Recommended Synthesis
Use Claude draft as the base, then incorporate these upgrades from research alignment:
1. Fix tab result identity model to be placement-scoped (sheet index + placement index or placement UUID), not just `PieceId`.
2. Add explicit contour normalization (outer ring orientation) before convex/concave scoring.
3. Define conflict policy for unsatisfiable constraints (priority order: safety spacing > corner exclusion > min-tabs, or similar).
4. Add explicit outer-contour-only handling and tests for parts with holes.
5. Add a bounded material-aware tab-height mode consistent with Section 4.5 (even if optional in Phase 1).
6. Make export schema change explicit/versioned and add compatibility acceptance criteria.
