# Sprint 007 Merge Notes

## Draft Strengths

### Claude Draft
- Best architectural depth: detailed `thetta-cam` crate rationale, data flow diagrams, threading model, and dependency graph update.
- Most complete implementation plan: 7 phases with concrete file paths, Rust code snippets, and algorithm pseudocode for `walk_edge_with_tabs`.
- Strongest test strategy: specific test tables with input/expected pairs for toolpath builder, G-code emitter, and tool DB.
- Multi-pass step-down included as P0 with clear depth-pass computation algorithm.
- Good use case coverage including edge cases (empty sheets, pieces without tabs, corrupt tool DB).
- Intermediate `Toolpath` representation well-justified for testability and future post-processor flexibility.
- Estimated LOC breakdown per file aids sprint planning.

### Codex Draft
- Intermediate toolpath representation explicitly justified as a design decision with clear reasoning.
- Phasing is well-sequenced (types -> planner -> emitter -> persistence -> UI -> E2E).
- Risk table correctly identifies `edge_index/t` mapping as the hardest technical problem.
- Toolpath planner kept in `thetta-geometry` (pure geometric logic, no I/O).
- Atomic file writes (temp-file + rename) for tool table persistence.
- Deterministic output explicitly called out as a design goal and test target.
- `CutDirection` (climb vs conventional) included in `ProfileCutConfig`.

### Gemini Draft
- Explicitly introduces `thetta-cam` crate with clean separation rationale.
- Tool radius compensation addressed in data flow (offset by tool diameter / 2).
- Prioritized DoD (P0/P1/P2 tiers) gives clear scope guidance.
- Z-axis crash risk called out with concrete mitigation (test Z height between piece boundaries).
- G-code float formatting safety mentioned in security section.
- Open questions from intent resolved inline.

## Valid Critiques Accepted

1. **Tool radius compensation is critical (all critiques).** The Codex draft omits it entirely; the Gemini draft hand-waves the mitigation. The interview decision resolves this: software-side polygon offset using `geo-buffer` (already in workspace deps) or proper parallel offset. This is the correct approach over G41/G42 because it keeps the toolpath planner self-contained and avoids LinuxCNC controller quirks.

2. **Tab projection onto offset contour needs a real algorithm (Claude critique of both drafts).** Proportional `t` mapping is invalid because offset changes edge topology. The interview decision specifies nearest-point projection from original tab position onto offset contour, which is a sound approximation for small tool radii.

3. **Spindle speed and feed rates must be P0 (Claude critique of Gemini).** G-code without `S` and `F` values is unusable. Accepted: these are mandatory P0.

4. **Step-down should be P1, not P0 (interview decision).** The Claude draft includes it as P0; the Gemini draft defers to P2. The interview places it at P1 (should ship). P0 uses single full-depth pass to reduce initial complexity. This is a pragmatic scope call.

5. **Tangential arc lead-in is P0 (interview decision).** Both drafts either defer lead-in entirely or make it P1/P2. The interview specifies tangential arc entry as part of the core algorithm. This prevents tool marks at entry points and is standard practice.

6. **`thetta-cam` crate, not `thetta-step` (Claude/Gemini drafts, rejected by Codex).** CAM logic in `thetta-step` would couple G-code text emission with OCCT FFI. All three critiques agree: separate crate is correct.

7. **Float formatting requirements (Claude critique).** Fixed precision (3 decimal places), locale-independent, no NaN/Inf. Both original drafts under-specify this. Accepted.

8. **Coordinate system documentation (Claude critique).** Nesting origin = bottom-left of sheet = G54 work origin. Must be documented in the sprint.

9. **Winding direction validation (Claude critique).** Planner must validate CCW outer winding as a precondition, consistent with existing `tabs.rs` approach.

10. **Hole contours explicitly skipped (Claude critique of Codex).** `Polygon.holes` exist in the type but this sprint is profile-only (outer contour). Must be documented and tested.

## Critiques Rejected

1. **Use G41/G42 instead of software offset (Claude critique suggestion).** The interview explicitly decided on software-side offset. G41/G42 has its own complications (requires lead-in of at least one tool radius, different behavior across LinuxCNC versions, cannot preview the actual cut path). Software offset gives full control and makes the toolpath preview accurate.

2. **Toolpath planner in `thetta-geometry` (Codex draft).** The interview and architecture decision places the planner in `thetta-cam`. The planner depends on CAM concepts (tool radius, feed rates, lead-in arcs) that don't belong in the pure geometry crate. `thetta-geometry` keeps only shared types.

3. **Six phases is over-structured (Claude critique of Codex).** The final sprint uses 4 phases per interview guidance, which is a reasonable middle ground.

4. **Tool table persistence in `thetta-step` (Codex draft).** Rejected by all critiques. Tool table I/O belongs in `thetta-cam` alongside the CAM logic it serves, avoiding coupling with OCCT.

5. **`step_down` in P0 (Claude draft, Codex critique).** Interview decision: P1. Single full-depth pass for P0 simplifies initial implementation while step-down ships as P1 (should ship).

6. **Cutting direction deferred to P2 (Gemini draft).** Interview places it at P1 with a documented default (conventional/CCW for outer contour).

## Interview Refinements Applied

1. **Tool radius compensation**: Software offset via `geo-buffer` crate or proper parallel offset. Offset polygon contour outward by tool radius before generating toolpath. Existing `offset()` in `polygon_ops.rs` is a centroid-radial approximation and insufficient; must use proper method.

2. **Step-down**: Moved to P1 (should ship). P0 uses single full-depth pass. Step-down adds multi-pass with tab handling across depth levels.

3. **Lead-in/lead-out**: Tangential arc entry (G2/G3) is P0. Approach contour from outside via small arc tangent to first contour edge at entry point. This is part of the core algorithm, not deferred.

4. **Entry point selection**: Furthest from part center, on a straight edge, away from tabs. Not just `contour.outer[0]`.

5. **Tab projection**: Nearest-point projection from original tab world-space position onto offset contour. Approximation that works well for small tool radii relative to feature size.

6. **G-code structure**: Explicit per-sheet structure with `%` program delimiters, `G54` coordinate system, `G21`/`G90` modes, `T/M6` tool change, `S/M3` spindle, per-piece rapid/lead-in/contour/lead-out/retract sequence, `M5`/`M2` postamble.

7. **Key types refined**: `ToolpathMove` enum with 6 variants (Rapid, LinearCut, ArcCut, TabLift, Retract, Plunge) replaces simpler `ToolpathSegment`. `ProfileCutConfig` added for lead-in radius and cutting direction. `GcodeConfig` includes `coordinate_system` (G54).
