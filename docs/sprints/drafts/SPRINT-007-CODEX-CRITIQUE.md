# Sprint 007 Draft Critique (Codex)

This critique evaluates:
- `/docs/sprints/drafts/SPRINT-007-CLAUDE-DRAFT.md`
- `/docs/sprints/drafts/SPRINT-007-GEMINI-DRAFT.md`
against `/docs/sprints/drafts/SPRINT-007-INTENT.md`.

## Evaluation Criteria
1. Strengths
2. Weaknesses
3. Gaps in risk analysis
4. Missing edge cases
5. Definition of Done (DoD) completeness

## Claude Draft Critique

### Strengths
- Strong architectural separation (`thetta-cam` crate) with clear rationale and good alignment to intent constraints.
- Very complete implementation plan with phased tasks, concrete file-level changes, test strategy, and data flow clarity.
- Good treatment of tab-aware motion logic, multi-pass step-down, and per-sheet generation behavior.
- Includes practical UI/state integration details (threading model, background export flow, status updates), consistent with existing app patterns.
- DoD is comprehensive and mostly testable, with explicit behavioral checks and tooling checks (`cargo test`, `cargo clippy`).

### Weaknesses
- Scope is likely too large for one sprint (estimated ~1300 LOC, multiple new modules, UI CRUD, async export, persistence, extensive tests). This increases delivery risk.
- Some implementation details are over-specified early (e.g., exact modal emission behavior, fixed precision policy), which may reduce flexibility during execution.
- Uses `tool_db.rs` in `thetta-cam` while persistence/UI ownership is in `thetta-app`; ownership boundary could be debated and may cause layering friction.
- A few assumptions need tighter validation against current codebase conventions (exact `PanelAction` flow, current file dialog patterns, where persistence helpers normally live).

### Gaps in Risk Analysis
- Limited treatment of numerical robustness and geometry degeneracy: tiny edges, duplicate vertices, self-touching polygons, near-zero tab widths/heights.
- No explicit mitigation for invalid user inputs beyond simple clamping (e.g., `step_down > cut_depth`, `safe_z` below stock top due to coordinate conventions).
- Limited risk analysis around machine-state safety conventions (units mismatch from machine state, coordinate offsets/work coordinate selection assumptions).
- No explicit risk coverage for concurrent state mutation while background export runs (stale references are handled by cloning, but UX/state race behavior is not discussed).

### Missing Edge Cases
- Tabs that overlap each other or are extremely close, causing interval merge/split ambiguity.
- Tabs located exactly at contour vertices or near contour wrap-around boundaries with floating-point tolerance issues.
- Very small parts where tab width exceeds local edge length.
- Contours with holes/interior loops (even if deferred for machining, should define explicit ignore/reject behavior).
- Duplicate tool numbers or deletion of currently selected tool from UI.
- Existing tool DB schema migration/versioning for future field changes.

### DoD Completeness
- **Coverage:** High. Includes most intent success criteria and extends into strong verification.
- **Traceability to intent:** Strong alignment on LinuxCNC focus, per-sheet output, tab-aware profile behavior, persistent tool DB, and non-blocking UI.
- **Potential overreach:** Includes non-trivial extras and a large matrix of tests that may exceed sprint capacity.
- **Net assessment:** Best technical depth and execution readiness, but needs scope tightening to reduce schedule risk.

---

## Gemini Draft Critique

### Strengths
- Concise and readable; easy for stakeholders to understand quickly.
- Correctly identifies key building blocks (new `thetta-cam` crate, tool DB UI, LinuxCNC output flow).
- Includes a useful high-level risk table and a clear phased implementation narrative.
- DoD has clear P0/P1/P2 prioritization, which is good for sprint planning discussions.

### Weaknesses
- Several P0 choices conflict with the intent and/or increase complexity prematurely:
  - P0 includes outward tool radius compensation, while intent does not require this and Sprint 007 context suggests profile-only, pragmatic first CAM path.
  - Multi-pass step-down is deferred to P2, but intent success criteria explicitly includes configurable step-down.
- Architecture and implementation are comparatively shallow: fewer concrete algorithms, weaker detail on tab mapping fidelity, weaker threading/export lifecycle specifics.
- DoD statements are less testable in places (e.g., “valid G-code” without explicit acceptance checks).
- “Tool database compatible with LinuxCNC tool table format” is mentioned in overview but actual storage is JSON; compatibility strategy is not defined.

### Gaps in Risk Analysis
- Underestimates complexity of offsetting and tab remapping after compensation (listed but mitigation is vague and potentially unstable).
- Missing risks around depth-pass correctness (because step-down is pushed to P2 despite being critical in intent).
- Limited discussion of UI/persistence failure modes (partial writes, disk permission errors, schema evolution).
- No explicit risk around machining safety defaults (unsafe feed/plunge combinations, missing tool selection, safe-Z sanity checks).

### Missing Edge Cases
- Empty/corrupt tool DB handling is only lightly implied; no concrete acceptance behavior.
- No explicit handling for tabs near start/end contour wrap-around and vertex-boundary placement.
- No explicit behavior for sheets with mixed valid/invalid placements (missing piece references).
- No explicit checks for extremely thin features relative to tool diameter if compensation is enabled.
- No explicit output determinism expectations (stable file naming/content ordering across runs).

### DoD Completeness
- **Coverage:** Medium. Captures major themes but misses some intent-critical depth.
- **Traceability to intent:** Partial; strongest miss is step-down placement (intent says configurable step-down, draft moves it to stretch).
- **Testability:** Moderate to low in places due to broad language and fewer concrete acceptance criteria.
- **Net assessment:** Good strategic summary, but under-specified and mis-prioritized for this sprint’s intent.

---

## Comparative Verdict
- **More execution-ready draft:** Claude.
- **More concise stakeholder summary:** Gemini.
- **Most important correction needed:** Make step-down a P0 requirement (as in intent), and avoid forcing full cutter compensation into P0 unless explicitly re-scoped.

## Recommended Consolidation
1. Use Claude’s architecture, algorithm detail, and test rigor as the baseline implementation plan.
2. Apply Gemini’s concise prioritization style to trim Claude’s scope (reduce optional polish in P0).
3. Keep P0 focused on: tab-aware profile centerline cuts, tool DB persistence, LinuxCNC emission, per-sheet export, and step-down.
4. Defer cutter compensation, modal optimization polish, and advanced UX features unless capacity remains.
