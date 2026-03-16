# Sprint 008 Merge Notes

## Draft Strengths

### Claude Draft (primary basis)
- Strongest architectural decomposition: three new modules (`hole_classify.rs`, `drill.rs`, `pocket.rs`) prevent `toolpath.rs` bloat
- Explicit circularity formula: `4*pi*area / perimeter^2`
- Detailed data flow for `generate.rs` multi-operation orchestration
- Comprehensive DoD with `cargo test --workspace` and multi-sheet checks
- Clear design decisions tied back to IDEA.md and Sprint 007 patterns

### Codex Draft
- Best intermediate representation: `SheetProgram -> Vec<OperationPlan>` with `op_type` discriminant
- Three-variant drill model (`DrillCycleStart`/`DrillHole`/`DrillCycleEnd`) maps directly to LinuxCNC canned cycle semantics
- Identified serde backward compatibility risk for config migration
- Modal safety emphasis in G-code emitter tests
- Drill point deduplication for messy CAD imports

### Gemini Draft
- Concrete use cases (UC1-UC6) serve as acceptance criteria
- Clear architecture diagram with three parallel planners feeding one emitter
- Drill-to-pocket fallback for oversized holes (adopted as P1)
- Explicitly identified `geo-buffer` degenerate polygon risk
- Addressed pocket plunge strategy (center plunge P0, helical ramp P1)

## Accepted Critiques

### From Claude's critique of Codex + Gemini
- Codex `PocketLink` variant underspecified -- dropped in favor of standard `Rapid`/`Plunge`/`LinearCut` for pocket transitions
- Gemini's single `Drill` variant conflates cycle semantics -- adopted Codex's three-variant model instead
- Both drafts missing concrete circularity formula -- Claude's `4*pi*area/perimeter^2` adopted with 0.95 threshold
- Neither draft adequately addressed world-space transformation risk -- added explicit validation requirement

### From Codex's critique of Claude + Gemini
- Claude internal inconsistency on tool-change policy (same-tool suppression) -- resolved: skip redundant T/M6 when consecutive operations use same tool number
- Claude overconfident on threshold defaults -- made circularity threshold user-configurable
- Gemini architecture too shallow for execution -- Claude's modular breakdown adopted
- Both missing explicit acceptance behavior for disabled operations and missing tools

### From Gemini's critique of Claude + Codex
- Critical safety gap in both: no tool diameter vs hole/pocket size validation -- added as P0 requirement
- Claude's pocket entry (retract-rapid-plunge) upgraded to helical ramp entry per interview decision
- Codex stuffing everything into `toolpath.rs` rejected in favor of Claude's three-module split
- Deep pocket single-pass risk noted -- multi-pass step-down added as P1

## Interview Refinements

1. **Hole classification**: dual criteria (circularity > 0.95 AND effective_diameter <= drill_tool_diameter)
2. **Pocket entry**: helical ramp-in at pocket center (not vertical plunge) -- safer for tools
3. **Peck drilling**: G83 as default with configurable peck_depth (5mm default)
4. **Module layout**: Claude's three new modules confirmed
5. **ToolpathMove model**: Codex's three-variant drill model (`DrillCycleStart`/`DrillHole`/`DrillCycleEnd`) plus new `HelicalRamp` variant
6. **Step-over**: 50% tool diameter default
7. **Operation ordering**: per-sheet global (drill ALL -> pocket ALL -> profile ALL)
8. **Pocket finish pass**: P1 item
9. **Tool diameter validation**: P0 safety check (reject drill if tool > hole, reject pocket if tool > minimum pocket dimension)
10. **Same-tool optimization**: skip redundant T/M6 if consecutive operations share tool number

## Rejected Ideas

- Codex's `stepdown` in PocketConfig without multi-depth implementation -- moved to P1 as explicit feature
- Codex's `dwell_ms` in DrillConfig -- unnecessary for wood/composite CNC router workflows
- Gemini's zigzag pocketing as P1 -- moved to P2 stretch goal
- Claude's "always emit tool changes even for same tool" -- replaced with same-tool optimization per interview
- Gemini's `ArcCut` in pocket transitions -- not grounded in current emitter; use `LinearCut` for pocket rings
- Codex's per-material preset defaults -- out of scope for this sprint
