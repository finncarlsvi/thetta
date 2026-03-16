# Sprint 008 Critique: Codex Draft vs Gemini Draft

## Intent Alignment

Both drafts faithfully address the Sprint 008 intent: add hole drilling (G81/G83) and pocket clearing (contour-parallel offset) to the existing `thetta-cam` pipeline, with drill → pocket → profile ordering and per-operation tool assignment. Neither draft invents scope beyond IDEA.md. Both correctly identify `Polygon.holes` as the geometry source and propose extending the existing two-stage (toolpath planning → G-code emission) architecture.

---

## Codex Draft

### Strengths

1. **Operation model is explicit and well-structured.** The `SheetProgram → Vec<OperationPlan>` with `op_type: Drill | Pocket | Profile` gives a clean intermediate representation that decouples planning from emission. This is the strongest architectural contribution across both drafts.
2. **ToolpathMove extension is concrete.** New variants (`DrillCycleStart`, `DrillHole`, `DrillCycleEnd`, `PocketLink`) map directly to G-code semantics — cycle-start sets parameters, each hole is a point, cycle-end emits G80. This is closer to how LinuxCNC canned cycles actually work than Gemini's single `Drill` variant.
3. **Risk table is thorough.** Covers backward config compatibility (serde defaults), performance on dense pockets, and modal bugs — all realistic risks. The "excessive tool changes" row and its mitigation (group by operation, change only when number differs) shows practical CNC awareness.
4. **Security section is substantive.** ASCII sanitization, non-finite rejection, deterministic output, and no shelling out are all relevant to a G-code generation pipeline.
5. **Phase structure is logical.** Types → Planning → Emission → Integration is the correct dependency order, and each phase lists specific files.
6. **Config design is detailed.** `DrillConfig` includes `retract_z`, `peck_depth`, `dwell_ms`, `hole_tolerance`, `min_diameter`, `max_diameter`. `PocketConfig` includes `stepdown`, `finish_pass`, `min_area`. These are practical CNC parameters.

### Weaknesses

1. **Puts CAM config types in `thetta-geometry/src/types.rs`.** The intent says "no new crates" and the existing `GcodeConfig` is already in `types.rs`, but `DrillConfig` and `PocketConfig` are CAM-specific concerns. The geometry crate shouldn't accumulate CAM policy. This is a minor layering issue but sets a precedent.
2. **Missing pocket plunge strategy.** Codex describes "repeated inward offset by stepover until exhausted" but never specifies how the tool enters the pocket. No mention of ramping, helical entry, or even a direct center-plunge. This is a significant gap — plunging an endmill straight down is aggressive for many materials.
3. **"PocketLink" variant is underspecified.** It's listed in the ToolpathMove extension but never defined with fields. What does a pocket link move contain? Rapid? Linear at safe-Z? This matters for collision avoidance between offset rings.
4. **No circularity formula given.** Says "circularity score (perimeter/area derived)" and "diameter consistency check from centroid radii" but never commits to a specific formula or threshold. The open questions repeat this ambiguity.
5. **P2 items are unlikely to matter.** "Per-material default presets for drill peck depth/stepover" is speculative scope that doesn't belong in this sprint's DoD at any priority.
6. **`stepdown` in PocketConfig implies multi-depth pocketing** but nothing in the implementation phases addresses Z-level iteration. Either remove it or spec it out.

### Gaps in Risk Analysis

- **No risk for `geo-buffer` failing on complex concave pockets.** Gemini identifies this explicitly; Codex only mentions "pocket offset degenerates on narrow features" which is a subset of the problem. `geo-buffer` can produce self-intersecting results or empty output on concave polygons — this needs explicit handling.
- **No risk for coordinate-space bugs.** Holes from `Polygon.holes` are in local piece space but need world-space transformation before drill points or pocket offsets are meaningful. The data flow diagram mentions "transform piece polygons to world space" but no risk is identified for getting this wrong (e.g., forgetting rotation, mirroring).
- **No risk for drill-into-pocket overlap.** If a circular hole sits inside a larger pocket, should it be drilled AND pocketed? Or should the drill be suppressed? Neither draft addresses this convincingly.

### Missing Edge Cases

- Pieces with zero holes (covered in Phase 4 fixtures — good).
- Holes that are exactly on the circularity threshold — which way do they fall?
- Very small pockets that collapse to nothing after one offset — `min_area` helps but the behavior when all pockets are filtered out needs to be specified (empty pocket group → skip tool change).
- Arc segments in hole contours. `Polygon.holes` stores `Vec<Point2D>` (polylines), so arcs from STEP are already tessellated. But the circularity check should account for tessellation artifacts.

### Definition of Done Completeness

Good priority stratification (P0/P1/P2). P0 items cover the core requirements. However:
- No P0 item for "pieces with no holes produce valid G-code unchanged" — this is the most important regression guard.
- P1 "UI exposes drill/pocket parameters and validates invalid combinations" should be P0 since the intent explicitly lists UI configuration as a success criterion.
- No DoD item for multi-sheet correctness — intent success criterion #6.

---

## Gemini Draft

### Strengths

1. **Use cases are concrete and user-facing.** UC1–UC6 each describe a specific user-visible behavior, making them good acceptance criteria. "Peck drilling for thick materials" (UC5) and "contour-parallel pocketing" (UC6) are specific enough to test against.
2. **Architecture diagram is clear.** The component box diagram showing the three parallel planners feeding into a single `gcode.rs` emitter is immediately understandable.
3. **Pocket plunge strategy is addressed.** "Add an initial plunge move in the center of the innermost remaining area" — this is a concrete decision, even if it's aggressive. It also identifies ramped/helical entry as a P1 improvement, which is pragmatic.
4. **Drill-to-pocket fallback (P1).** "If a drill hole is larger than the drill tool diameter, automatically convert it to a pocket operation" — this is an important real-world case that Codex misses entirely.
5. **Fewer open questions, more decisions made.** Gemini's open questions are narrower (peck depth defaults, tolerance values) while Codex leaves more fundamental questions open (classification policy, grouping policy).
6. **`geo-buffer` risk is explicit.** "Pocket offsetting fails on complex concave inner holes" with mitigation "discard degenerate sliver geometries" — directly addresses a real library limitation.

### Weaknesses

1. **Single `ToolpathMove::Drill` variant conflates cycle semantics.** `Drill { to, depth, peck, feed }` puts all cycle parameters on every hole move. In LinuxCNC, `G81`/`G83` set parameters once, then each hole is just `X Y`. Codex's `DrillCycleStart` / `DrillHole` / `DrillCycleEnd` is closer to the machine model and will produce more idiomatic G-code.
2. **No `OperationPlan` abstraction.** Goes straight from classification to emission without an intermediate operation grouping type. This will make orchestration in `generate.rs` ad-hoc — collecting three separate `Vec<Toolpath>` rather than a structured operation plan.
3. **Config types are thin.** `DrillConfig` has only "cycle type, peck depth, tool number" and `PocketConfig` has only "step-over percentage, tool number". Missing: retract height, dwell, min/max diameter thresholds, min pocket area, finish pass toggle. These will need to be added during implementation.
4. **Phase ordering puts UI last (Phase 4).** This means the first three phases have no way to receive configuration from the user — hardcoded defaults or function arguments only. This isn't blocking but means integration testing is deferred.
5. **Risk table rates "drill classification false positives" as Low likelihood.** With tessellated polyline input and no specified circularity formula, false positives are more likely than "Low." A rounded rectangle tessellated to many vertices could score as circular.
6. **Security section is perfunctory.** "No new external system dependencies" and "CNC machines can be damaged" are true but don't add actionable guidance. No mention of input sanitization, non-finite coordinate rejection, or deterministic output.
7. **No backward compatibility consideration.** Adding `DrillConfig` and `PocketConfig` to `AppState` will break deserialization of existing saved state. Codex identifies this risk; Gemini doesn't mention it.

### Gaps in Risk Analysis

- **No risk for world-space transformation errors** (same gap as Codex).
- **No risk for tool change emission bugs.** What if drill and pocket use the same tool number? Should the tool change be suppressed? The orchestrator says "at most 3 tool changes per sheet" but doesn't address the same-tool case.
- **No risk for empty geometry.** If a sheet has pieces but none have holes, the drill and pocket sections should be entirely absent from G-code — not just empty tool change blocks.
- **Plunge risk is identified but underweighted.** Rated "High likelihood, Medium impact" — direct endmill plunging into material should be "High impact" as it can break tools, especially in metal or hard plastic.

### Missing Edge Cases

- Nested holes (hole within a hole) — the intent mentions this explicitly but Gemini doesn't address it.
- Holes that straddle piece boundaries after nesting transformation (shouldn't happen with correct transforms, but worth a validation check).
- Pocket with a single remaining island after offset — what happens to the link move?
- Multiple drill holes at nearly the same location (duplicate detection).
- Sheet with all operations disabled — should produce no file or an empty file?

### Definition of Done Completeness

P0 is comprehensive for core functionality. However:
- P1 items are ambitious for "should ship" — smooth pocket transitions AND drill-to-pocket fallback AND zigzag pocketing alternative. Pick one; the others are P2.
- No DoD item for regression against Sprint 007 profile/tab behavior.
- No DoD item for multi-sheet correctness.
- No DoD item for "no holes" producing valid output.
- "UI panels allow tool assignment for all three operations" is P0 — good, this matches the intent.

---

## Comparative Summary

| Dimension | Codex | Gemini |
|-----------|-------|--------|
| **Architecture** | Stronger — explicit `OperationPlan` model, cycle-aware ToolpathMove variants | Weaker — no operation abstraction, conflated drill variant |
| **Config depth** | More thorough — retract Z, dwell, diameter bounds, min area | Thin — will need expansion during implementation |
| **Practical CNC awareness** | Good (modal safety, tool change optimization) | Good (plunge strategy, drill-to-pocket fallback) |
| **Risk coverage** | Broader (6 risks including serde compat, perf) | Narrower (4 risks) but includes geo-buffer explicitly |
| **Edge case handling** | Better fixture coverage in Phase 4 | Misses nested holes, empty sheets |
| **Use cases** | Implicit (no UC section) | Explicit UC1–UC6, easier to validate |
| **Pocket plunge** | Missing | Addressed (center plunge + P1 ramping) |
| **Backward compat** | Addressed | Missing |
| **DoD** | Good stratification, missing regression/multi-sheet items | Good P0 coverage, P1 too ambitious |

## Recommendation

A merged plan should take:
- **From Codex:** `OperationPlan` abstraction, cycle-start/hole/cycle-end ToolpathMove model, detailed config structs, backward compatibility handling, security guidance.
- **From Gemini:** Explicit use cases as acceptance criteria, pocket plunge strategy (center plunge for P0, ramp for P1), drill-to-pocket fallback (P1), `geo-buffer` risk mitigation.
- **From neither (must add):**
  - Concrete circularity formula and threshold (e.g., `4π·area / perimeter² > 0.85`).
  - World-space transformation risk and validation.
  - Same-tool-number optimization (suppress redundant tool changes).
  - Nested hole handling policy (treat as single pocket? error?).
  - DoD items for: regression against S007, multi-sheet output, no-holes pieces, empty operation groups.
  - Specify `stepdown` behavior or remove it — no half-specified features.
