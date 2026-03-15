# Sprint 005: Architecture Documentation — Codex Draft

## Overview

Produce a comprehensive `docs/ARCHITECTURE.md` that maps the current state of Thetta’s code-base and major design decisions. The document serves onboarding developers and future reviewers by answering **what exists**, **why it looks that way**, and **how information flows** across crates, threads, and files. No production code changes are allowed; the sprint is pure documentation with optional helper scripts.

Key objectives:

1. Explain the overall purpose and domain scope of Thetta (2D irregular nesting of STEP-derived profiles with a desktop GUI).
2. Enumerate every workspace crate, its public surface, and responsibilities.
3. Visualise relationships (crate graph, data-flow, threading) using portable ASCII (or Mermaid if agreed).
4. Capture critical architectural decisions (forked `jagua-rs`, multi-sheet via multiple SPP engines, OCCT FFI isolation, etc.).
5. Describe verification strategies so the document remains truthful over time.

## Use Cases

UC1: **New contributor ramp-up**  
  A Rust developer joins the project. Reading `ARCHITECTURE.md` they can open the right crate, understand data types, follow the optimizer loop, and run the app in <30 minutes.

UC2: **Design review reference**  
  Before adding new features (e.g., constraint-based nesting) the maintainer consults the architecture doc to assess coupling and extension points.

UC3: **Bug triage aid**  
  When a threading dead-lock is reported, the doc’s threading model diagram quickly shows which channels and locks may be involved.

UC4: **External stakeholder overview**  
  Potential collaborators skim the doc to judge integration effort without reading the code.

## Architecture (of the Sprint Delivery)

The sprint outputs a single Markdown file:

```
docs/
└── ARCHITECTURE.md   <— new, authored this sprint
```

### Required Sections in `ARCHITECTURE.md`

1. **Project Purpose & Scope** — mission statement and problem space.
2. **Crate Overview** — table listing every crate with responsibilities, key modules, and public types.
3. **Dependency Graph** — ASCII or Mermaid diagram of crate-level dependencies; annotate feature flags.
4. **Data Flow Pipeline** — STEP import → nesting → STEP export; refine existing diagrams.
5. **Threading & Concurrency Model** — channels, background threads, UI thread; rationale for non-blocking UI.
6. **Optimization Pipeline** — LBF → GLS → Compression → Multi-Sheet orchestrator; explain choice of multiple SPP engines instead of BPP.
7. **Key Architectural Decisions** — bullet list linking to commits/sprints for each decision.
8. **Testing & CI Strategy** — workspace tests, benchmark fixtures, STEP round-trip tests.
9. **Build & Tooling Notes** — workspace resolver, system libs, feature flags.
10. **Extensibility Hot-spots** — where new algorithms, file formats, or UI panels plug in.
11. **Glossary** — domain term definitions.

### Verification Strategy for Accuracy

- **Crate enumeration script**: small script lists workspace members; diff against doc table in CI.
- **File reference check**: grep every module path in doc to ensure it exists.
- **Diagram lints**: ensure Mermaid or ASCII width < 100 chars for readability.

## Implementation

- Draft outline directly in `docs/ARCHITECTURE.md` skeleton.
- Use `rg` helpers to auto-extract crate names and top-level modules.
- Capture diagrams using Mermaid fenced blocks (fallback ASCII).
- Self-review by opening each referenced file/type.
- Optional `scripts/verify_arch_doc.rs` sanity checker.
- Update top-level `README.md` with doc link.

## Files Summary

- **NEW** `docs/ARCHITECTURE.md` — 800-1200 lines covering all sections.
- **MOD** `README.md` or `docs/README.md` — add link in “Documentation” list.
- **NEW (opt)** `scripts/verify_arch_doc.rs` — automated checks.

## Definition of Done

1. `docs/ARCHITECTURE.md` committed with all required sections populated.
2. Manual walkthrough confirms every path, type, and diagram matches `main` branch.
3. `cargo test --workspace` passes (no code changes).
4. Markdown renders without errors on GitHub.
5. PR approved and merged without TODO comments.

## Risks

- **Doc drift**: mitigated via update checklist in CONTRIBUTING.
- **Over-detail**: balance depth with maintenance cost.
- **Mermaid rendering**: some terminals lack support; provide ASCII fallback.

## Security

Pure documentation; no runtime impact. Avoid leaking confidential paths.

## Dependencies

- Coordination near sprint end to freeze APIs so doc stays accurate.
- OpenCASCADE installation only required if screenshots/scripts introspect OCCT (optional).

## Open Questions

1. Use **Mermaid** or ASCII diagrams?
2. Include historical sprint timeline?
3. Where should accuracy checks live—CI or pre-commit?
4. Depth of `jagua-rs` internals to expose?

