# Sprint 005 – CODEX Critique

This document reviews the two parallel architecture-documentation proposals and highlights, for **each draft**, their strengths, weaknesses, risk-analysis gaps, missing edge cases, and Definition-of-Done (DoD) completeness.

---

## 1. Claude Draft

### Strengths

- **Depth & specificity** – Fully-fleshed outline (12 sections), three ASCII diagrams, concrete file-path anchors; low ambiguity for implementers.
- **Verification checklist** – Phase 3 explicitly validates paths, types, dependency graph, reducing drift risk.
- **Detailed DoD** – Five measurable buckets (D1–D5) covering existence, accuracy, code immutability, and readability.
- **Up-front rationale** – Ordering/content choices justified, aiding future reviewers.
- **Risk table present** – Identifies drift/accuracy risks with mapped mitigations.

### Weaknesses

- **Over-prescriptive size** – 800–1200-line target may intimidate contributors and slow updates; large relative to ~15 KLOC codebase.
- **Dead-code coverage** – Documents unused `jagua-rs` BPP/MSPP modules, adding noise and maintenance cost.
- **Toolchain assumption** – Forces ASCII diagrams while GitHub now supports Mermaid; could age poorly.
- **Manual verification burden** – Grep/`ls` checks every sprint may be unrealistic without automation.
- **No capacity estimate** – Scope vs. team bandwidth not addressed; risk of spill-over.

### Gaps in Risk Analysis

- **Diagram rendering** – Wide ASCII diagrams may overflow on variable-width viewers; unaddressed.
- **Contributor fatigue** – Ongoing maintenance burden & motivation risks missing.
- **Cross-platform encoding** – Windows CRLF or encoding quirks omitted.

### Missing Edge Cases

- **Future crate changes** – No guidance for diagrams when crates are added/renamed.
- **Version skew** – Multi-branch maintenance (release vs. main) not considered.
- **Feature toggles** – OCCT-disabled builds hide functions; doc could mislead.

### DoD Completeness

Strong, but could add:

- **Peer-review gate** – Require maintainer sign-off before merge.
- **CI lint** – Automate path/type existence checks to sustain D2 reliability.

---

## 2. Gemini Draft

### Strengths

- **Concise & approachable** – 300–500-line target lowers maintenance overhead.
- **Clear phased timeline** – Five-day breakdown aids sprint planning.
- **Focus on key diagrams** – Highlights three high-value visuals without excessive detail.
- **Build-env awareness** – Notes macOS ARM64 + OCCT hurdles.

### Weaknesses

- **Shallow DoD** – Only P0/P1 lists; lacks explicit accuracy verification.
- **No glossary/extension-points** – Onboarding may suffer without terminology scaffold.
- **Light risk section** – Drift mitigation limited to “document at high level”.
- **Threading detail lighter** – Channel topology not diagrammed; hampers bug triage.
- **No validation tooling** – Risk of silent staleness.

### Gaps in Risk Analysis

- **Capacity buffer** – Five-day schedule lacks contingency discussion.
- **Diagram readability** – ASCII width/clarity risk ignored.
- **Platform variability** – OCCT Windows/Linux differences not mentioned.

### Missing Edge Cases

- **Feature flags** – `occt` off-path, benchmarking toggles, etc., omitted.
- **Optimisation modes** – Multi-sheet vs. single-sheet runner edge-cases absent.
- **Error-handling flow** – STEP I/O failure propagation through threads/UI not described.

### DoD Completeness

- **Accuracy gate missing** – No `cargo test`/no-code-change requirement.
- **Readability standard vague** – No user-acceptance akin to Claude’s D5.
- **Peer review omitted** – Risks inaccuracies entering `main`.

---

## 3. Comparative Summary & Recommendations

- **Blend rigour and brevity** – Pair Claude’s verification and DoD thoroughness with Gemini’s shorter length for sustainability.
- **Merge risk mitigations** – Combine Claude’s drift checks with Gemini’s build-environment notes.
- **Automate verification** – Add CI job (Markdown link/path linter) to enforce path/type existence.
- **Keep glossary & extension-points** – Claude’s glossary aids onboarding; retain it.
- **Exclude dead code** – Omit unused `jagua-rs` modules; reference only in a footnote if needed.

These measures will produce an architecture document that is accurate, maintainable, and right-sized for Thetta’s velocity.

