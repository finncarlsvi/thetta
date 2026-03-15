# Sprint 005 Draft Critique

Reviewer: Claude | Date: 2025-03-14

Evaluated against: `SPRINT-005-INTENT.md` and project conventions established in Sprints 001-004.

---

## Codex Draft

### Strengths

1. **Comprehensive section outline.** The 11-section structure for ARCHITECTURE.md goes beyond the intent's requirements — "Extensibility Hot-spots" and "Glossary" are genuine value-adds that the intent didn't explicitly request but that serve the stated audience (onboarding developers).

2. **Use cases are well-scoped.** UC1-UC4 cover a broader range of readers (new contributor, maintainer doing design review, bug triage, external stakeholder) than the Gemini draft. UC3 (bug triage via threading diagram) is a particularly practical scenario.

3. **Verification strategy.** The crate-enumeration script and file-reference check ideas are concrete and actionable. The suggestion to run these in CI addresses long-term doc drift, which is a real risk for architecture documents.

4. **README update.** Small but practical — a new developer needs to discover the doc exists.

### Weaknesses

1. **Thin implementation plan.** The "Implementation" section is six bullet points. There's no phasing, no sequencing, no indication of what to write first or how to verify accuracy incrementally. Compare this to Sprint 004's detailed phase-by-phase breakdown. For a doc sprint this matters less than for a code sprint, but the lack of structure means there's no way to track progress or cut scope if time runs short.

2. **Flat Definition of Done.** Five undifferentiated items with no priority tiers. If time is constrained, which items are P0? The intent's success criteria list 5 points — the DoD should map to these explicitly and add verifiable acceptance criteria. "Manual walkthrough confirms every path, type, and diagram matches `main` branch" is vague — who does this, against what checklist?

3. **Doesn't resolve open questions.** The draft restates 4 open questions from the intent almost verbatim without taking a position. A sprint plan should make decisions (with rationale), not defer them. Mermaid vs ASCII, jagua-rs depth, historical context, and algorithm detail level all need answers before execution starts.

4. **Estimated size is vague.** "800-1200 lines" is a wide range. At 1200 lines the document risks becoming a maintenance burden (contradicting the intent's maintainability constraint). At 800 it's tight for 11 sections with diagrams. No breakdown of expected size per section.

5. **Optional verification script is scope creep.** `scripts/verify_arch_doc.rs` is a nice idea but writing and maintaining a Rust script to validate a Markdown file is engineering work that could consume real time in a documentation-only sprint. It should be explicitly marked as P2/stretch or cut.

6. **README modification blurs "no code changes" constraint.** The intent says "No code changes in this sprint." Modifying `README.md` is borderline — it's not production code, but it is a tracked file outside the sprint deliverable. Should be flagged and approved explicitly.

### Gaps in Risk Analysis

- **No risk identified for accuracy.** The hardest part of an architecture doc is getting the details right. What if the document describes a data flow that doesn't match the code? The only mitigation is "manual walkthrough" with no process.
- **No risk for scope creep.** 11 sections + optional script + README update + CI integration is ambitious for a doc sprint. What gets cut first?
- **Missing: reviewer availability.** DoD item 5 says "PR approved" — by whom? Solo developer project, so self-review, but the intent mentions "manual review alongside code" which takes real time.

### Missing Edge Cases

- What happens when a crate is added in a future sprint? The doc needs a clear "how to update this doc" section or convention (the draft mentions "CONTRIBUTING" but the project doesn't have one).
- Diagram width constraint (100 chars) is good but doesn't address what to do when the crate graph grows beyond that.
- No mention of how to handle feature-gated code paths (e.g., `thetta-step`'s `occt` feature) in the documentation.

### Definition of Done Completeness

Partially complete. Missing:
- Specific section-by-section acceptance criteria
- Priority tiers
- Mapping to intent's 5 success criteria
- Verification procedure (who runs what)

---

## Gemini Draft

### Strengths

1. **Resolves open questions.** Unlike the Codex draft, Gemini takes clear positions: ASCII diagrams for portability, jagua-rs as black box with API surface documented, no historical context (stays in sprint docs). These are reasonable decisions with stated rationale. This is ready to execute.

2. **Prioritized Definition of Done.** P0/P1 separation is good practice and matches the convention from Sprint 004. The P0 items are concrete and verifiable: "Document includes a Crate Dependency Graph (ASCII)" is binary — it's there or it isn't.

3. **Phased implementation.** Four phases with day-by-day breakdown (Skeleton → Core → Deep Dives → Review) gives a clear execution order and natural checkpoints. Phase 4 dedicated to verification is smart for a documentation sprint.

4. **Risk table with severity ratings.** Three risks with severity levels and mitigations. The "document becomes immediately outdated" risk has a good mitigation (architectural level, not function level). The "inaccurate jagua-rs fork" risk correctly identifies the highest-risk section.

5. **Section structure mirrors data flow.** The proposed ARCHITECTURE.md outline (Introduction → Crate Graph → Data Flow → Threading → Deep Dives → Build Notes) follows the natural reading order for someone understanding the system top-down. This is better pedagogy than a flat list.

### Weaknesses

1. **Underestimates document size.** 300-500 lines for a comprehensive architecture document covering 5 crates, 3 diagrams, algorithm deep dives, and build notes is tight. Sprint 004's plan alone is 820 lines. The intent asks for coverage of "project purpose, crate structure, dependency graph, data flow, threading model, optimization pipeline, UI architecture, testing patterns, and build/configuration notes" — that's 9 topics minimum. At 300 lines, each topic gets ~33 lines including diagrams, which forces superficiality.

2. **Missing sections the intent requires.** The intent's success criteria #4 asks for "Key architectural decisions explained (why SPP not BPP, why forked jagua-rs, why trigonometric collision detection, etc.)." The Gemini outline pushes this to P1 ("Explanation of key architectural decisions"). This should be P0 — architectural decisions are arguably the most valuable part of the document, since code structure can be read from the source but decision rationale cannot.

3. **No extensibility or glossary sections.** The Codex draft includes these; Gemini doesn't. For a project where sprint planning regularly involves deciding where new features plug in, an extensibility map is high-value. A glossary is also useful given the domain-specific terminology (NFP, SPP, BPP, GLS, LBF, trigonometric collision detection).

4. **Testing/CI strategy is P1 instead of P0.** The intent's success criteria explicitly lists "testing patterns" as required coverage. Demoting this to P1 risks shipping an incomplete document.

5. **No mention of README update or discoverability.** A developer needs to find the document. If it's not linked from the project root, it may go undiscovered.

### Gaps in Risk Analysis

- **No risk for insufficient depth.** The 300-500 line estimate combined with broad coverage requirements creates a real risk of the document being too shallow to be useful. A new developer reading a surface-level overview might still not understand how the pieces connect.
- **No risk for day-by-day schedule slippage.** If Phase 2 takes longer than a day (likely, given it covers both introduction and the full data pipeline), Phases 3-4 get compressed. No contingency plan.
- **Missing: accuracy verification process.** Phase 4 says "manually trace documentation against codebase" but doesn't specify how. Check every `pub struct`? Every module path? Every diagram arrow? This needs a concrete checklist.

### Missing Edge Cases

- Feature-gated code paths (`thetta-step` with/without `occt`) are not mentioned in the outline.
- The `jagua-rs` fork's dead BPP code — should the doc mention it exists but is unused? This is relevant architectural context (Sprint 004 explicitly decided against it).
- Multi-sheet vs strip nesting mode dispatch — this is a key architectural boundary that should appear in the data flow section.
- How the document handles cross-crate type sharing (everything flows through `thetta-geometry`) deserves explicit treatment.

### Definition of Done Completeness

Good structure (P0/P1 tiers) but:
- Missing P2/stretch items (the Codex draft's verification script, glossary, extensibility map would fit here)
- No "cut if behind schedule" list (Sprint 004 had this — useful convention)
- P0 item "The document accurately reflects the code currently in `main`" is not verifiable as stated — needs a concrete procedure
- Missing: `cargo test --workspace` passes (the intent requires this as a sanity check that no files were accidentally modified)

---

## Comparative Summary

| Dimension | Codex | Gemini | Verdict |
|-----------|-------|--------|---------|
| **Resolves open questions** | No — restates them | Yes — takes positions with rationale | Gemini |
| **Section coverage** | 11 sections (superset of intent) | 6 sections (subset of intent) | Codex |
| **Implementation structure** | Bullet list, unsequenced | 4 phases, day-by-day | Gemini |
| **DoD quality** | Flat, 5 items, no tiers | P0/P1 tiers, 9 items | Gemini |
| **Risk analysis depth** | 3 risks, no severity | 3 risks, severity + mitigation | Gemini |
| **Size estimate realism** | 800-1200 lines (reasonable) | 300-500 lines (too low) | Codex |
| **Scope discipline** | Creeps (script, README, CI) | Tight, maybe too tight | Gemini |
| **Intent alignment** | Good but defers decisions | Good but drops some requirements | Tie |
| **Sprint convention adherence** | Weak (no phases, no P0/P1) | Strong (matches Sprint 004 style) | Gemini |

## Recommendation for Final Sprint Plan

Merge the best of both drafts:

1. **Use Gemini's structure** — phased implementation, P0/P1/P2 DoD tiers, resolved open questions. This matches project conventions.
2. **Use Codex's section list** — adopt the 11-section outline, but make "Key Architectural Decisions", "Testing & CI Strategy", and "Glossary" all P0. Move "Extensibility Hot-spots" to P1.
3. **Target 600-800 lines** — split the difference. Enough depth to be useful, not so much it becomes unmaintainable.
4. **Add a "cut if behind schedule" list** — following Sprint 004's convention. Candidates: Glossary, Extensibility Hot-spots, verification script.
5. **Add `cargo test --workspace` to DoD** — the intent requires it as a no-regression check.
6. **Specify the verification procedure concretely** — e.g., "every module path referenced in the doc is checked with `rg` against the codebase; every diagram arrow corresponds to a real `use` or dependency in `Cargo.toml`."
7. **Keep the README update but flag it** — useful for discoverability, but note it's a minor tracked-file change outside the core deliverable.
8. **Drop the verification script** — over-engineering for a solo project. Manual verification with a checklist is sufficient. Revisit if the team grows.
