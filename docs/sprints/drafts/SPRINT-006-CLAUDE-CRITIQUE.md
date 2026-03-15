# Sprint 006 Claude Critique: Codex Draft vs Gemini Draft

Reviewed against: `research/RESEARCH-005.md` Section 4, Sprint 006 Intent document.

---

## 1. Codex Draft

### Strengths

- **Closest alignment with RESEARCH-005 Section 4.** Reproduces the algorithm nearly verbatim: perimeter-based count with `clamp(round(P/tab_distance), min_tabs, max_tabs)`, candidate scoring (straight=1.0, convex=0.8, concave=0.3, too-short=0.0), edge-length-proportional distribution, spacing validation (>= 2*tab_width), and min-count recovery with corner-exclusion relaxation. This is the most faithful translation of the research into a sprint plan.
- **Explicit data flow diagrams.** Two ASCII diagrams (pipeline-level and per-part algorithm) make the architecture immediately understandable. The per-part flowchart mirrors Section 4.4 steps 1-5 directly.
- **Thoughtful storage decision.** Correctly identifies the tension between embedding tabs in `NestingResult` (cross-crate churn) vs. keeping `TabResult` as app-level derived state, and explicitly flags it as a decision point rather than silently choosing one.
- **Comprehensive risk table.** Six risks with concrete mitigations. Highlights floating-point determinism drift (a real concern for reproducibility) and NaN/extreme-config robustness — issues the Gemini draft ignores entirely.
- **Phase granularity.** Five phases with explicit file lists per phase. This makes the sprint plannable and parallelizable (Phase 1-2 can be reviewed independently of Phase 3-5).
- **Security section is substantive.** Goes beyond "no new I/O" to address malformed polygons, NaN coordinates, extreme config values, and panic safety — all relevant for a geometry algorithm processing untrusted input.

### Weaknesses

- **Tab sizing is incomplete.** Lists `tab_height_for_material()` and `default_tab_width_from_height()` as tasks but does not reproduce the per-material-type ratios from Section 4.5 (wood_soft=0.40, wood_hard=0.33, plastic=0.33, composite=0.50) or the `tab_width = clamp(1.5 * tab_height, 5.0, 25.0)` formula. An implementer would need to go back to the research document. This should be explicit in the sprint spec.
- **No `NestingUpdate::Improved` tab handling clarity.** Phase 3 says to trigger tab recompute on `Improved` updates, but this could mean recomputing tabs on every GLS iteration — potentially hundreds of times. The intent document is ambiguous here (Gemini's "strictly on Finished" is arguably better scoped for Sprint 006). At minimum, the draft should specify whether intermediate tab results are displayed or only final.
- **Missing test for circular/arc contours.** Section 4 and RESEARCH-005 Section 7.1 point 4 both flag curved contour handling as poorly understood and a known failure mode (viaconstructor bug). The Codex draft lists "circle approximation" in tests but doesn't address how the algorithm handles true arcs vs. polyline approximations of circles. Since Thetta polygons are `Vec<Point2D>` (polylines), this is fine for Sprint 006 — but the draft should explicitly state this assumption.
- **Open Question 6 is scope creep.** Asking whether Phase 1 should "ship rectangle-only visual markers while reserving future tab-shape variants" mixes rendering concerns with algorithm scope. The research recommends rectangular tabs for Phase 1; this should be a decision, not an open question.
- **No mention of winding order dependency.** The convex/concave classification depends on consistent polygon winding (CW vs CCW). The draft mentions "normalize winding assumptions" in risks but doesn't address it in implementation tasks. If `thetta-geometry` polygons don't guarantee winding order, this is a blocking prerequisite.

### Gaps in Risk Analysis

- **No risk for weighted-distribution rounding errors.** When `round(count * weighted_fraction)` is applied per-edge, rounding can produce a total that differs from `count`. The draft mentions "remainder handling with stable ordering" but doesn't flag this as a risk or describe the resolution strategy.
- **No risk for polygon transformation correctness.** Phase 3 transforms pieces by `Placement` to world space before tab computation. If the transform is wrong (e.g., rotation center mismatch), all tabs will be misplaced. This should be a risk with a test mitigation.

### Missing Edge Cases

- **Parts with all short edges** (e.g., a polygon approximation of a small circle where every edge is shorter than `tab_width + 2*corner_exclusion`). All candidates score 0.0 — the min-count recovery path must handle this, but the draft doesn't describe what happens when *all* edges are invalid.
- **Collinear consecutive edges.** Two adjacent edges that form a straight line should arguably be treated as one edge for scoring and distribution. The draft doesn't address edge merging.
- **Zero-area or self-intersecting polygons.** The security section mentions malformed polygons generically but doesn't specify behavior.

### Definition of Done Completeness

The DoD covers the core algorithm, UI, viewport, export, and testing. **Missing items:**
- No criterion for tab sizing matching Section 4.5 material rules.
- No criterion for deterministic behavior under polygon transformation (world-space correctness).
- No criterion for graceful handling when all edges are too short (the "all-invalid" case).
- "Unit tests cover rectangles, circles, concave geometry, tiny/degenerate parts, and determinism" is good but should explicitly include spacing-conflict resolution and min-count recovery.

---

## 2. Gemini Draft

### Strengths

- **Clean, concise structure.** Easier to read than the Codex draft. Three phases instead of five reduces coordination overhead and is appropriate for a feature of this scope.
- **Correct architectural instinct.** Keeps nesting pipeline untouched, tab algorithm in `thetta-geometry`, UI in `thetta-app`. Module name `tabs.rs` is more idiomatic than `tab_placement.rs` (shorter, matches Rust convention for module naming).
- **Explicit STEP export mention.** Task 3.3 mentions representing tabs "as metadata or visible geometry" in STEP export — a forward-looking consideration the Codex draft defers entirely.
- **Performance risk is well-scoped.** "Keep algorithm O(N) relative to polygon edges" is the right target and a useful constraint for reviewers.

### Weaknesses

- **Significantly less algorithmic detail than the Codex draft.** The implementation section describes *what* to do ("tab generation algorithm following the RESEARCH-005 specification") but not *how*. An implementer would need to independently re-derive the algorithm from the research document. Key omissions:
  - No mention of the specific scoring values (1.0/0.8/0.3/0.0).
  - No mention of the `tab_width + 2*corner_exclusion` minimum-length filter.
  - No mention of the spacing validation threshold (2*tab_width).
  - No mention of the min-count recovery path or corner-exclusion relaxation.
  - No mention of the per-material tab sizing formulas.
  - No mention of the `clamp(1.5 * tab_height, 5.0, 25.0)` width calculation.
- **Triggers only on `NestingUpdate::Finished`.** Task 2.3 says "immediately when `NestingUpdate::Finished` is received." This is conservative and correct for Sprint 006, but doesn't address what happens during long-running nesting jobs where the user wants intermediate visual feedback. The intent document lists `Improved` as a trigger option. This should be acknowledged even if deferred.
- **STEP geometry modification in Sprint 006 is risky.** Task 3.3 proposes modifying OCCT STEP export to include tab geometry. This crosses into OCCT complexity (global state, thread safety, build dependencies) that Sprint 006 should avoid. The Codex draft correctly limits Sprint 006 to JSON metadata only and defers STEP geometry changes.
- **Weaker risk table.** Only four risks vs. Codex's six. Missing entirely:
  - Floating-point determinism (critical for reproducible builds and testing).
  - Degenerate/malformed polygon robustness.
  - Export schema versioning and backward compatibility.
- **Security section is too brief.** "No new external dependencies" and "no new I/O" is accurate but doesn't address robustness concerns (NaN, panics, extreme inputs) that matter for a geometry algorithm.
- **No explicit data model.** Mentions `Tab`, `TabConfig`, `MaterialType`, `TabResult` but doesn't define their fields. The Codex draft provides full struct definitions including `edge_index`, `t` parameter, `piece_id`, and `sheet_index` — all essential for a correct implementation.
- **Missing `TabResult` storage decision.** Doesn't address whether tabs live on `NestingResult` or as separate app-level state. The data flow diagram shows `[NestingResult with Tabs]` which implies modifying `NestingResult` — a higher-churn, higher-risk approach that the Codex draft correctly flags as a decision point.

### Gaps in Risk Analysis

- **No risk for OCCT STEP export complexity.** Task 3.3 proposes STEP geometry modification but the risk table doesn't mention OCCT build issues, thread safety, or the complexity of representing tab volumes as B-rep solids.
- **No risk for polygon winding order.** Same gap as the Codex draft.
- **No risk for floating-point non-determinism.** The intent document explicitly lists determinism as a success criterion.
- **No risk for tab-count rounding accumulation.**

### Missing Edge Cases

All the same edge cases missing from the Codex draft, plus:
- **No mention of the size_threshold parameter** in the algorithm description (only in the DoD). An implementer might miss this.
- **No mention of what happens when `min_tabs` cannot be achieved** even after relaxation. The Codex draft at least asks this as an open question.
- **No mention of corner_exclusion relaxation** as a recovery mechanism.

### Definition of Done Completeness

Reasonable coverage but gaps:
- "Algorithm enforces minimum tab counts and corner exclusion distances" — but doesn't specify what happens when these conflict (recovery behavior).
- No criterion for tab sizing correctness (matching Section 4.5 material rules).
- No criterion for multi-sheet correctness (the Codex draft includes this).
- No criterion for spacing-conflict resolution.
- "No regressions in existing nesting benchmarks" is good and missing from Codex's DoD.
- Missing: deterministic output for identical inputs (mentioned in intent but not in Gemini's DoD — actually present: "updates recalculate tabs deterministically").

---

## 3. Comparative Assessment

| Dimension | Codex | Gemini | Verdict |
|-----------|-------|--------|---------|
| **Algorithmic fidelity to Section 4** | High — reproduces scoring, spacing, recovery | Low — references spec but omits details | Codex |
| **Architecture** | Sound, explicit decision points | Sound, slightly simpler | Tie |
| **Data model completeness** | Full struct definitions | Names only, no fields | Codex |
| **Phase structure** | 5 phases (fine-grained) | 3 phases (simpler) | Gemini slightly better for sprint velocity |
| **Risk analysis** | 6 risks, includes FP determinism | 4 risks, misses critical items | Codex |
| **Edge case coverage** | Good, some gaps | Weaker, more gaps | Codex |
| **DoD completeness** | 11 items, some gaps | 9 items, some gaps | Codex (slightly) |
| **STEP export scope** | Correctly deferred to metadata | Over-scoped (OCCT geometry) | Codex |
| **Readability** | Dense but thorough | Clean and scannable | Gemini |
| **Security** | Substantive | Minimal | Codex |
| **Open questions** | 6, some are scope creep | 3, all well-scoped | Gemini |

### Recommendation

**Use the Codex draft as the primary base**, incorporating the following from the Gemini draft:
1. The simpler 3-phase structure (merge Codex phases 1+2 into "Core Algorithm", phases 3+4 into "Integration + UI").
2. The `tabs.rs` module name.
3. The "no regressions in existing nesting benchmarks" DoD item.
4. The "strictly on `NestingUpdate::Finished`" trigger scoping for Sprint 006.

### Items both drafts should add

1. **Explicit Section 4.5 tab sizing formulas** in the implementation spec (per-material ratios, width clamp bounds).
2. **Winding order prerequisite** — verify or enforce CCW outer ring before convex/concave classification.
3. **Edge case: all edges too short** — define behavior (return empty tabs? warn? force single tab at longest edge midpoint?).
4. **Edge case: collinear consecutive edges** — define whether to merge or treat independently.
5. **Rounding remainder strategy** — explicit rule for distributing leftover tabs after per-edge rounding.
6. **Placement transform test** — verify tabs are correct in world space, not just local polygon space.
7. **DoD item for tab sizing** matching Section 4.5.
8. **DoD item for spacing-conflict resolution** and min-count recovery.
