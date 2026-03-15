# Sprint 005 Merge Notes

## Draft Strengths Adopted

### From Claude Draft
- **12-section document structure** with dependency-order crate deep dives (geometry first, app last)
- **Three ASCII diagrams**: crate dependency graph, data flow pipeline, threading model
- **Architectural decisions table** with alternatives and rationale (7+ decisions)
- **Glossary** with domain-specific terms (SPP, BPP, LBF, GLS, NFP, CDE, etc.)
- **Extension points table** mapping features to entry points
- **Content principles**: accuracy over aspiration, file paths as anchors, independently updatable sections
- **Phase 3 verification** with concrete checks (file paths, type accuracy, dependency graph)
- **D1-D5 structured DoD** with measurable criteria

### From Codex Draft
- **README update** for discoverability (flagged as minor tracked-file change)
- **Verification automation idea** — adopted as future improvement, not in-sprint scope
- **UC4: External stakeholder overview** — broader audience consideration

### From Gemini Draft
- **P0/P1 priority tiers** in Definition of Done (matches Sprint 004 convention)
- **Phased implementation** with clear sequencing
- **Build environment notes** (macOS ARM64, OCCT constraints)
- **Concise scope control** — avoid over-documentation

## Valid Critiques Accepted

| Critique | Source | Action |
|----------|--------|--------|
| Codex draft doesn't resolve open questions | Claude critique | Resolved in interview: ASCII diagrams, jagua-rs as API surface, no historical section |
| Gemini underestimates doc size (300-500 lines) | Claude critique, Codex critique | Target 600-800 lines per interview |
| Gemini missing glossary & extension points | Claude critique, Codex critique | Added as P0 per interview |
| Claude draft too prescriptive on type-level detail | Codex critique | Reduced to module-level; keep key types but not all fields |
| Codex verification script is scope creep | Claude critique, Gemini critique | Dropped from sprint scope |
| Missing `cargo test --workspace` in DoD | Claude critique | Added |
| Architectural decisions should be P0, not P1 | Claude critique | Promoted to P0 |
| jagua-rs dead code (BPP/MSPP) should be mentioned but not documented | Codex critique | Brief footnote only |

## Critiques Rejected

| Critique | Source | Reasoning |
|----------|--------|-----------|
| Should consider Mermaid diagrams | Codex critique | ASCII is more portable, diffs cleanly, renders everywhere. Interview confirmed. |
| Cross-platform (Windows/Linux) docs needed | Codex critique, Gemini critique | Project targets macOS ARM64 only. Mention this, but no need for multi-platform docs. |
| CI automation for doc verification | Codex critique | Over-engineering for solo project. Manual verification sufficient. Revisit if team grows. |
| README modification violates "no code changes" | Claude critique | README is documentation, not code. Include as minor change. |

## Interview Refinements

1. **jagua-rs depth**: API surface only — document inputs, outputs, and why forked. Reference academic paper for internals.
2. **Extra sections**: Include both glossary and extension points as P0.
3. **Target length**: 600-800 lines. Balanced depth without maintenance burden.
