# Sprint 007 Draft Critique

Reviewer: Claude Opus 4.6
Date: 2025-03-15
Inputs: SPRINT-007-INTENT.md, SPRINT-007-CODEX-DRAFT.md, SPRINT-007-GEMINI-DRAFT.md

---

## 1. Codex Draft Critique

### Strengths

1. **Intermediate toolpath representation is the right call.** The explicit decision to separate geometric planning (`thetta-geometry::toolpath`) from LinuxCNC string emission (`thetta-step::gcode`) is well-motivated and produces a testable, extensible architecture. The reasoning is stated clearly in Design Decisions.

2. **Phasing is well-sequenced.** Types first, then planner, then emitter, then persistence, then UI integration, then E2E validation. Each phase builds on the previous one with clear boundaries. This enables incremental testing and reduces the risk of late-stage integration surprises.

3. **Risk table is concrete and actionable.** The `edge_index/t` mapping risk is the single hardest technical problem in this sprint, and the draft correctly identifies it as medium-likelihood/high-impact. The mitigation (focused planner tests) is appropriate.

4. **Keeps toolpath planner in `thetta-geometry`.** This is architecturally correct — the planner is pure geometric logic with no I/O dependencies. It aligns with the existing crate boundary where `thetta-geometry` owns domain types and `thetta-step` owns file format concerns.

5. **Atomic file writes for tool table.** Calling out temp-file + rename is a small but important detail for data integrity.

6. **Deterministic output is explicitly called out** as both a design goal and a test target. This matters for reproducibility and debugging.

7. **Comprehensive file summary table** makes it easy to audit scope.

### Weaknesses

1. **No tool radius compensation.** The draft describes profile cuts following "placed outer contours" but never mentions offsetting the toolpath outward by `tool.diameter / 2`. A profile cut on the contour line itself would leave the part undersized by the tool radius on every edge. This is a critical omission — the generated G-code would produce parts with wrong dimensions. The planner must either offset the contour or emit G41/G42 cutter compensation codes.

2. **Tab mapping after offset is unaddressed.** Even if offset is added, the `Tab.edge_index` and `Tab.t` parameters reference the *original* polygon, not the offset polygon. The offset changes edge count (corners become arcs or extra segments), so naive index mapping breaks. This is the hardest technical problem in the sprint and the draft doesn't discuss a solution.

3. **Lead-in/lead-out mentioned in types but not in implementation.** `ProfileCutConfig` includes `lead_in_mm` and `lead_out_mm` fields, but Phase 2 and Phase 3 never describe how these are used. Either remove them from the type (defer to a later sprint) or describe the implementation. Half-specified features create confusion during implementation.

4. **`step_down` included in P0 scope but risky.** Multi-pass depth adds significant complexity to tab-lift interactions (each pass must lift differently). The draft acknowledges this risk but still includes it as a DoD item. This should be P1 or P2 — single-pass at full depth is sufficient for a first working G-code export.

5. **Six phases is arguably over-structured** for what is fundamentally three units of work (types+planner, emitter+persistence, UI+integration). The phases are fine as mental checkpoints but may slow a single implementer who could work more fluidly across boundaries.

6. **No mention of coordinate system assumptions.** LinuxCNC G-code needs a defined work coordinate system (G54 default). The draft emits `G90` but doesn't discuss how the nesting coordinate origin maps to the CNC work origin. If the nesting origin is bottom-left of the sheet, that needs to be documented and validated.

7. **Missing edge case: pieces with holes.** The intent says "profile cuts only" (outer contour), but the `Polygon` type has `holes: Vec<Vec<Point2D>>`. The draft should explicitly state that hole contours are skipped in this sprint and add a test confirming this.

### Gaps in Risk Analysis

- **Tool radius compensation correctness** is not listed as a risk, yet it's arguably the highest-impact technical challenge (or its absence is the biggest gap — see Weakness #1).
- **Polygon winding direction** is assumed correct but never validated. If a polygon has CW outer winding, the climb/conventional direction logic inverts. A winding check should be a precondition.
- **Float formatting in G-code** is not discussed. LinuxCNC is sensitive to locale-dependent decimal separators and excessively long decimal strings. The emitter must use fixed-precision, locale-independent formatting.
- **Large sheet / many-piece performance** is not considered. For a sheet with 50+ placements, is the single-threaded planner fast enough? Probably yes for profile-only, but worth noting.

### Missing Edge Cases

- Piece contour with very short edges (< tool diameter) — offset may collapse or self-intersect
- Tab width exceeding remaining edge length after corner exclusion
- Two tabs on the same edge whose lift regions overlap (draft mentions "adjacent tabs" in tests but not the resolution strategy)
- Zero-thickness material (cut_depth = 0) — should be a validation error
- Tool diameter = 0 — should be a validation error
- Sheet with placements but all placements have empty polygons (degenerate input)

### Definition of Done Completeness

The DoD is solid but missing:
- No criterion for tool radius compensation correctness (parts must be dimensionally correct)
- No criterion for G-code being loadable in a LinuxCNC simulator without errors
- No criterion for float formatting validity
- "Optional multi-pass depth" as a DoD item is ambiguous — is it required or not?

---

## 2. Gemini Draft Critique

### Strengths

1. **Explicitly introduces a `thetta-cam` crate.** This is a defensible architectural choice. G-code generation is conceptually distinct from STEP I/O, and a dedicated crate prevents `thetta-step` from growing into a catch-all I/O module. The crate name clearly communicates purpose.

2. **Tool radius compensation is addressed.** The data flow explicitly includes "Apply tool radius compensation (offset by tool diameter / 2)" as a pipeline step. This is the most critical technical requirement that the Codex draft omits entirely.

3. **Prioritized DoD (P0/P1/P2).** Breaking the Definition of Done into must-ship, should-ship, and stretch tiers is pragmatic. It gives the implementer clear guidance on what to cut if time runs short. Putting step-down in P2 is the right call.

4. **Z-axis crash risk is called out.** The risk of a rapid move at cut depth between pieces is a real CNC safety concern. The mitigation (test Z height between piece boundaries) is concrete and testable.

5. **G-code float formatting safety mentioned.** The security section calls out NaN and unparseable floats, which is a real concern the Codex draft ignores.

6. **Open questions are resolved inline.** Each open question from the intent has a stated resolution, which reduces ambiguity for the implementer.

### Weaknesses

1. **Tab position mapping after offset is hand-waved.** The risk table says "Map tab parameter `t` from the original contour to the offset contour proportionally," but this is not a valid approach. Polygon offsetting changes edge topology — corners become arcs, short edges may vanish, edges may merge or split. Proportional `t` mapping would place tabs at wrong positions. This needs a real algorithm: project original tab world-space positions onto the offset contour, or compute arc-length correspondence.

2. **Tool radius compensation is listed as "High likelihood / High impact" risk but the mitigation is weak.** "Use `polygon_ops`, fallback to zero offset and warn" means the happy path may silently produce incorrect G-code (zero offset = undersized parts). A better mitigation: for V1, emit G41/G42 cutter compensation codes and let LinuxCNC handle the offset, avoiding polygon offsetting entirely. Or: require the user to acknowledge the limitation.

3. **Implementation is underspecified.** Only three phases, each described in 2-3 sentences. No detail on:
   - How the toolpath planner segments the contour at tab positions
   - How multi-sheet iteration works
   - How the background export job is structured
   - What the `Toolpath` type actually contains (moves? segments? waypoints?)
   - How validation errors surface in the UI

4. **Spindle speed and feed rates are P1, not P0.** This is wrong — G-code without `S` (spindle speed) and `F` (feed rate) values is not usable on any real CNC machine. LinuxCNC will either error or use whatever modal values happen to be set from a previous program. These must be P0.

5. **No test plan.** The DoD mentions "unit tests for G-code formatting and toolpath tab lifting" but there's no test file listed, no test strategy described, and no edge cases enumerated. Compare to the Codex draft which lists specific test scenarios (no-tabs, single-tab, multiple tabs on same edge, adjacent tabs, tiny edges, deterministic output).

6. **Ramp lead-in/lead-out for tabs is P1.** Without ramp transitions, the tool will plunge vertically into the material at each tab exit. Vertical plunges at full cut depth can break tools, especially in hard materials. At minimum, the P0 should use a gradual ramp-down rather than a vertical plunge, even if lead-in/lead-out arcs are deferred.

7. **No mention of cutting direction.** Climb vs. conventional milling affects surface finish, tool life, and cut forces. Deferring to P2 is acceptable, but the draft should state the default (e.g., conventional) and ensure the contour traversal direction implements it correctly.

8. **Tool table persistence is mixed into app state.** The draft puts load/save logic in `thetta-app/src/state.rs` rather than in `thetta-cam` or `thetta-step`. This couples persistence logic to the UI crate, making it harder to test and reuse.

### Gaps in Risk Analysis

- **No risk for incorrect contour winding / traversal direction.** If the polygon outer ring is not consistently CCW, the tool may cut on the wrong side.
- **No risk for coordinate system mismatch.** Nesting coordinates vs. CNC work coordinates — if the origin convention differs, all cuts are shifted.
- **No risk for empty tab list.** If a piece has no tabs, does the full-depth profile cut work correctly? (It should, but it's not discussed.)
- **No risk for UI blocking.** The Codex draft explicitly calls out background threading; the Gemini draft doesn't describe how export avoids blocking the egui main loop. The existing app uses a channel-based pattern that should be followed.

### Missing Edge Cases

- Same as Codex critique, plus:
- What happens if the user has no nesting result yet and clicks "Generate G-code"?
- What if the tool table file on disk is corrupted or from a future version?
- What if two pieces on the same sheet overlap (nesting bug) — does the G-code generator detect this?
- Output directory doesn't exist or is read-only
- Filename collisions with existing `.ngc` files

### Definition of Done Completeness

The P0 DoD is mostly correct but:
- Missing: validation that G-code is parseable by a LinuxCNC simulator
- Missing: deterministic output for identical inputs
- **Critical**: Spindle speed and feed rate must be P0, not P1
- Missing: explicit criterion for coordinate correctness (world-space contours, not piece-local)
- The P1/P2 split is mostly good, except for the feed/spindle issue above

---

## 3. Comparative Assessment

| Dimension | Codex Draft | Gemini Draft |
|-----------|-------------|--------------|
| **Architecture** | Keeps planner in `thetta-geometry`, emitter in `thetta-step`. Conservative, follows existing boundaries. | Introduces new `thetta-cam` crate. Cleaner separation but adds workspace complexity. |
| **Tool radius compensation** | Not addressed (critical gap) | Addressed but with weak mitigation |
| **Tab-to-offset mapping** | Not addressed | Mentioned but hand-waved |
| **Implementation detail** | High — 6 phases with specific files, tasks, and test scenarios | Low — 3 phases with brief descriptions |
| **Test strategy** | Specific test cases enumerated | Generic "unit tests" mentioned |
| **DoD structure** | Flat checklist | Prioritized P0/P1/P2 tiers |
| **Risk analysis** | Good coverage, misses cutter comp | Good coverage, misses winding/coordinates |
| **Scope control** | step_down included (risky) | step_down deferred to P2 (better) |
| **Feed/spindle handling** | Included in P0 | Incorrectly deferred to P1 |
| **Practical usability** | Ready to implement as-is with additions | Needs significant fleshing out before implementation |

### Recommendation

**Use the Codex draft as the base**, with these modifications drawn from the Gemini draft and this critique:

1. **Add tool radius compensation** — either polygon offset in the planner or G41/G42 emission in the emitter. For V1, G41/G42 is simpler and delegates the hard geometry to LinuxCNC's built-in compensation.
2. **Adopt P0/P1/P2 tiering** from the Gemini draft. Move `step_down` and lead-in/lead-out arcs to P1/P2.
3. **Consider a `thetta-cam` crate** rather than putting CAM logic in `thetta-step`. The Gemini draft's argument is valid — G-code is not STEP I/O, and the crate will grow with future operations (drilling, pocketing).
4. **Add explicit tab-to-offset mapping strategy.** If using G41/G42, this problem goes away (tabs reference original contour, cutter comp is applied by the controller). If doing software offset, define the projection algorithm.
5. **Add float formatting requirements** — fixed precision (3-4 decimal places), locale-independent, no NaN/Inf.
6. **Add coordinate system documentation** — define how nesting origin maps to G54 work origin.
7. **Add winding direction validation** as a precondition in the planner.
8. **Ensure feed rates and spindle speed are in P0** — they are not optional for usable G-code.

### Key Technical Risk Both Drafts Underweight

The hardest problem in this sprint is the interaction between **tool radius compensation** and **tab position accuracy**. Tabs are defined on the original polygon contour. The actual cutting path is offset outward by the tool radius. The correspondence between these two paths is non-trivial because offsetting changes edge geometry. Both drafts either ignore this (Codex) or hand-wave it (Gemini).

The simplest V1 solution: **emit G41/G42 cutter radius compensation codes** and let LinuxCNC handle the offset. The toolpath planner works on the original contour, tabs stay accurate, and the controller offsets the physical tool position. This defers the hard polygon-offset math to a future sprint while producing dimensionally correct parts today. The tradeoff is that G41/G42 has its own quirks (requires a lead-in move of at least one tool radius), but these are well-documented in the LinuxCNC manual.
