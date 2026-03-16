# Sprint 008 Draft Critique (Codex)

## Baseline: Intent Alignment Requirements
From `SPRINT-008-INTENT.md`, the non-negotiable targets are:
- Hole drilling and pocket clearing added to complete the 2D CAM pipeline.
- Hole classification from `Polygon.holes` into drill vs pocket.
- Drill cycles via G81/G83.
- Pocket clearing via contour-parallel strategy.
- Operation order: drill -> pocket -> profile.
- Per-operation tool assignment and tool changes.
- Multi-sheet behavior supported.
- UI exposure for drill/pocket configuration.

Both drafts broadly aim at this target, but differ in precision, internal consistency, and delivery readiness.

## Claude Draft Critique

### Strengths
- Strong architectural decomposition: clear separation into `hole_classify.rs`, `drill.rs`, `pocket.rs`, and orchestration/emitter changes.
- Good intent traceability: repeatedly ties choices back to IDEA/intent ordering and existing Sprint 007 infrastructure.
- Better operational depth than Gemini on planner behavior (classification metric, ring generation, ordering phases, G80 lifecycle).
- Includes practical implementation sequencing and file-level impact, which lowers execution ambiguity.
- DoD is more complete and test-oriented than Gemini, with explicit `cargo test --workspace` and multi-sheet/order checks.

### Weaknesses
- Internal inconsistency on tool-change policy:
  - Earlier states tool changes are emitted even when tools are same; later P0 says “when tools differ.”
  - This needs one deterministic rule to avoid behavioral drift and test ambiguity.
- Overloads scope with several design decisions as if finalized (e.g., always peck default recommendation, per-sheet-only ordering recommendation) without clearly marking as pending product decisions.
- Some claims are too confident without validation hooks (e.g., conservative threshold defaults, LinuxCNC compatibility assumptions).
- Adds significant type/config surface (validation, toggles, extra fields) that may exceed strict sprint minimum if schedule is tight.

### Gaps in Risk Analysis
- Missing controller-compatibility nuance beyond LinuxCNC happy path:
  - Canned cycle parameter interpretation differences (R-plane behavior, absolute/incremental interactions, feed modal state).
- Missing machining risk around depth strategy:
  - Treating pocket depth equal to profile through-depth can be unsafe/incorrect for many real pockets.
- Missing transformation/coordinate risks:
  - Hole center and contour orientation after placement transforms (rotation/mirroring) are not explicitly risked/tested.
- Missing geometry robustness risks for near-degenerate contours from STEP tessellation noise (tiny segments, self-intersections).

### Missing Edge Cases
- Holes/pockets smaller than tool diameter are mentioned, but no hard acceptance behavior matrix (skip/warn/error) per operation type.
- Ambiguous geometry cases:
  - Near-circular slots, rounded rectangles, polygons approximating circles at low vertex count.
- Multiple depth scenarios:
  - Blind pockets vs through pockets; mixed depths per feature not represented.
- Shared boundaries / nested islands:
  - Pocket islands are deferred, but this omission should be explicit in DoD exclusions and failure behavior.
- Zero/invalid config edge cases:
  - `step_over <= 0`, `step_over > 1`, negative peck depth, missing tool assignment with enabled operation.

### Definition of Done Completeness
- Overall: **High but not fully “ship-safe.”**
- What is good:
  - Covers core intent requirements and includes testing expectations.
  - Captures ordering and multi-sheet criteria.
- What is missing for completeness:
  - Explicit acceptance tests for disabled operations and missing-tool behavior.
  - Explicit validation outcomes for invalid configs (clamp vs reject).
  - Explicit output expectations for ambiguous classifications (deterministic fallback to pocket is implied but not formalized in DoD).
  - Explicit simulator verification criterion for full drill+pocket+profile program on at least one mixed-feature fixture.

## Gemini Draft Critique

### Strengths
- Concise and readable; easier to consume quickly.
- Captures the core story correctly: classify holes, add drilling/pocketing, enforce drill->pocket->profile ordering.
- Includes a basic phased implementation and an initial risk table.
- Notes practical fallback idea (oversized “drill hole” becomes pocket), which is product-useful.

### Weaknesses
- Architecture detail is too shallow for execution confidence:
  - Reuses `toolpath.rs` for everything conceptually, but lacks clear modular boundaries and interfaces.
- Technical imprecision:
  - Introduces `ArcCut` without grounding whether current pipeline/emitter supports it.
  - “Plunge at pocket center” can be invalid if center lies outside concave regions.
- Weaker consistency with existing code patterns compared to Claude draft.
- DoD mixes “must ship” and exploratory/stretch items without clear dependency boundaries.
- Less explicit about how multi-sheet and per-operation tool assignment behave under missing/disabled configs.

### Gaps in Risk Analysis
- Significant omissions versus intent complexity:
  - No serious risk treatment for classification false positives/negatives beyond one line.
  - No controller-modal risks for G81/G83/G80 state handling.
  - No risk analysis for transform precision, winding normalization, or degenerate offsets.
  - No mention of path efficiency/travel optimization tradeoffs affecting runtime.
- Security section is largely generic and not sprint-specific.

### Missing Edge Cases
- Concave pockets where centroid/center plunge is outside valid material.
- Tiny internal contours, duplicate/collinear vertices, self-intersecting hole polygons.
- Mixed cases on same sheet with one operation disabled and shared tools.
- Handling of no-hole/no-pocket sheets with operation toggles and tool-change suppression.
- Tolerance edge cases for “matches drill diameter” heuristic.

### Definition of Done Completeness
- Overall: **Medium; not release-ready as written.**
- What is good:
  - Includes the core functional milestones and basic ordering/tool-change checks.
- What is incomplete:
  - Lacks robust acceptance criteria for classification determinism and ambiguous contours.
  - Lacks explicit error/validation behavior for invalid configs and missing tools.
  - Lacks explicit multi-sheet verification depth (content parity across sheets, empty-operation sheets).
  - Lacks concrete simulation/fixture validation requirements.

## Comparative Verdict
- **Stronger draft:** Claude.
- **Why:** Better architectural specificity, implementation sequencing, and DoD breadth aligned to intent.
- **Main concern in Claude:** internal policy inconsistencies and a few unformalized acceptance behaviors.
- **Main concern in Gemini:** under-specified engineering details and missing risk/edge-case depth that would likely surface as rework during implementation.

## Recommended Improvements Before Sprint Approval
1. Normalize operation/tool-change policy into one explicit rule set (same tool across ops, disabled ops, missing tools).
2. Add a deterministic classification decision table:
   - circularity threshold, diameter tolerance, ambiguous fallback behavior.
3. Add explicit edge-case acceptance criteria:
   - too-small pockets/holes, invalid polygons, concave pockets, no-feature pieces.
4. Expand DoD with validation behavior:
   - parameter clamp/reject semantics and expected UI/user feedback.
5. Add at least one end-to-end verification fixture:
   - mixed holes+pockets+profiles across multi-sheet output, validated in LinuxCNC/CAMotics.
