# Sprint 005: Architecture Documentation (ARCHITECTURE.md)

## Overview

After 4 sprints, Thetta spans 5 crates (~15K+ LOC) integrating OpenCASCADE FFI, a forked collision-detection engine, a multi-level nesting optimizer, and a non-blocking egui GUI. The codebase has reached the complexity where an explicit architecture document is needed for onboarding, sprint planning, and bug triage.

This sprint produces a single deliverable: `docs/ARCHITECTURE.md` — a comprehensive reference documenting **what** each part of the system does, **how** data and control flow between them, and **why** the major design decisions were made. No code changes are made.

**Target size:** 600-800 lines of Markdown. Enough depth to be useful; not so much it becomes a maintenance burden.

## Use Cases

1. **New Developer Onboarding**: A Rust developer clones the repo and reads `ARCHITECTURE.md` to understand crate responsibilities, data flow, and where to make changes — without grepping randomly.

2. **Feature Planning Reference**: Before a new sprint, the maintainer consults the doc to identify coupling, extension points, and affected modules for a proposed feature.

3. **Bug Triage Aid**: A rendering glitch or optimizer hang is reported. The threading model section shows which thread owns what state and which channels connect them. The optimization pipeline section clarifies which phase produced a bad result.

4. **Architectural Decision Record**: A contributor asks "why multiple SPP engines instead of BPP?" The decisions table provides the rationale and alternatives considered.

## Architecture

### Document Structure

The `ARCHITECTURE.md` file contains 12 sections, ordered top-down (big picture first, then zoom in) with crate deep dives in dependency order (leaf crates before dependents):

```
docs/ARCHITECTURE.md
 1. Project Purpose & Domain
 2. Workspace Layout
 3. Crate Dependency Graph              [ASCII diagram]
 4. Data Flow Pipeline                  [ASCII diagram]
 5. Threading & Concurrency Model       [ASCII diagram]
 6. Crate Deep Dives
    6.1 thetta-geometry                 (no internal deps — leaf)
    6.2 jagua-rs (forked)               (API surface only — leaf)
    6.3 thetta-nesting                  (depends on geometry + jagua-rs)
    6.4 thetta-step                     (depends on geometry)
    6.5 thetta-app                      (depends on everything)
 7. Optimization Pipeline
 8. Key Architectural Decisions
 9. Build, Configuration & Feature Flags
10. Testing Patterns
11. Extension Points
12. Glossary
```

### Content Principles

1. **Accuracy over aspiration.** Every statement must be verifiable against the current codebase. No "should" or "will" — only "is" and "does."
2. **File paths as anchors.** Every module description includes the actual file path so readers can jump to source.
3. **ASCII diagrams.** Portable, diff-friendly, no rendering tooling needed. Three primary diagrams: crate dependency graph, data flow pipeline, threading model. Keep under 80 columns.
4. **Decisions include rationale.** For each key architectural choice: state the decision, alternatives considered, and why this path was chosen.
5. **Independently updatable sections.** Each section is self-contained so future sprints can update one section without rewriting the whole doc.

### Section Content Specifications

**Section 1 — Project Purpose & Domain**
- One paragraph: Thetta is a desktop app for 2D irregular polygon nesting — packing profiles extracted from 3D STEP files into rectangular sheets to minimize material waste.
- Domain context: Strip Packing Problem (SPP) and its multi-sheet extension. NP-hard. Real-world use: CNC/laser/waterjet cutting.
- Pipeline summary sentence: STEP import → 2D profile extraction → optimization (LBF + GLS + compression) → multi-sheet layout → STEP export.

**Section 2 — Workspace Layout**
- ASCII tree of `crates/` with one-line descriptions per crate.
- Table: Crate | Key responsibility | Internal dependencies.

**Section 3 — Crate Dependency Graph**
- ASCII box-and-arrow diagram showing all 5 crates and dependency edges.
- Annotate `thetta-step` → OCCT with `[feature = "occt"]`.
- Note key external deps on each crate (egui/eframe, opencascade-sys, rand).

**Section 4 — Data Flow Pipeline**
- ASCII diagram showing the full transformation pipeline with types at each stage.
- Three sub-flows: Import (STEP → Vec<Piece>), Optimization (Vec<Piece> → NestingResult), Export (NestingResult → STEP files).
- For each transformation: the function that performs it (with file path), input/output types, one-sentence description.

**Section 5 — Threading & Concurrency Model**
- ASCII diagram showing 4 threads and their communication channels.
- Main thread (egui): owns AppState, renders UI, polls channels. Never blocks.
- Optimizer thread: spawned on Start, cancelled via Arc<AtomicBool>. Sends NestingUpdate via mpsc.
- STEP import thread: one-shot, returns via channel.
- STEP export thread: one-shot, returns via channel.
- Synchronization primitives listed: mpsc channels, Arc<AtomicBool>, Arc<Mutex<StepModel>>.
- State ownership rules.

**Section 6 — Crate Deep Dives**
- For each crate: purpose, module inventory with file paths, key public types (names only, not full field listings), and design notes.
- **6.2 jagua-rs**: API surface only. What it does (trigonometric collision detection), inputs/outputs, why forked (continuous rotation, direct Rust API). Brief mention that BPP/MSPP modules exist but are unused. Reference academic paper for algorithmic internals.

**Section 7 — Optimization Pipeline**
- Three-phase walkthrough: LBF (greedy baseline, ~1s) → GLS (meta-heuristic, seconds-minutes) → Compression (binary search, seconds).
- Conceptual description of each phase (not pseudocode).
- Multi-sheet extension: fill-and-spill LBF, independent per-sheet GLS + compression.
- Channel protocol: NestingUpdate variants.

**Section 8 — Key Architectural Decisions**
- Table: Decision | Alternatives Considered | Rationale.
- Minimum 7 decisions:
  1. Multiple SPP engines for multi-sheet (not BPP)
  2. Forked jagua-rs (vendored in workspace)
  3. Trigonometric collision detection (no NFP)
  4. Continuous rotation support
  5. Non-blocking UI with background threads + mpsc
  6. Feature-gated OCCT with JSON fallback
  7. Fill-and-spill multi-sheet overflow
  8. Separation via polygon inflation

**Section 9 — Build, Configuration & Feature Flags**
- Workspace Cargo.toml: resolver v2, release profile (LTO, codegen-units=1, strip).
- `.cargo/config.toml`: CMAKE_POLICY_VERSION_MINIMUM=3.5 for OCCT compatibility.
- Feature flag: `thetta-step/occt`. Default enabled. Fallback: JSON.
- Platform: macOS ARM64 (primary target).
- OCCT system requirement: `brew install opencascade`.
- How to run: `cargo run -p thetta-app`.

**Section 10 — Testing Patterns**
- `cargo test --workspace` runs all tests. OCCT tests need `--test-threads=1`.
- Test data convention: STEP files use `.json` extension to avoid OCCT reader.
- Test categories: unit tests in modules, integration tests in `tests/`, benchmark tests (ESICUP), convergence tests.

**Section 11 — Extension Points**
- Table: Extension | Where to add | Interface to implement.
- Cover: new optimization heuristic, new file format import, new UI panel, new nesting mode, per-piece rotation constraints.

**Section 12 — Glossary**
- Domain terms: SPP, BPP, LBF, GLS, NFP, CDE, surrogate, quadtree, piece, placement, sheet, NestingResult, fill-and-spill, compression, STEP, OCCT.

## Implementation

### Phase 1: Audit & Outline (~20% of effort)

Systematically verify facts against the codebase before writing prose.

**Tasks:**
- [ ] Inventory all crates: pub types, module trees, Cargo.toml dependencies, feature flags
- [ ] Verify dependency graph edges against actual Cargo.toml declarations
- [ ] Trace the full data flow through code: import → conversion → LBF → GLS → compression → result → export
- [ ] Identify all thread::spawn calls, mpsc channels, Arc<AtomicBool>/Mutex usage
- [ ] Review sprint docs (001-004) for architectural decision rationale
- [ ] Create document skeleton with all 12 section headers

### Phase 2: Write Core Sections (~50% of effort)

**Files:**
- `docs/ARCHITECTURE.md` — Create with all content

**Tasks:**
- [ ] Write Section 1 (Purpose) and Section 2 (Workspace Layout)
- [ ] Draw ASCII diagrams: crate dependency graph (Section 3), data flow (Section 4), threading model (Section 5)
- [ ] Write crate deep dives (Section 6) in dependency order
- [ ] Write optimization pipeline (Section 7) with conceptual descriptions
- [ ] Write architectural decisions table (Section 8) with 7+ entries
- [ ] Write build/config (Section 9), testing (Section 10), extension points (Section 11), glossary (Section 12)

### Phase 3: Verification (~20% of effort)

**Tasks:**
- [ ] File path audit: every path in the doc exists in the repo
- [ ] Type accuracy: every type name matches the actual source
- [ ] Dependency graph: matches actual Cargo.toml declarations
- [ ] Diagram check: all arrows correspond to real dependencies or data flows
- [ ] Cross-reference: architectural decisions reference the correct sprint
- [ ] Run `cargo test --workspace` to confirm no accidental code changes
- [ ] Readability pass: read top-to-bottom as a new developer, flag missing prerequisites

### Phase 4: Polish & Ship (~10% of effort)

**Tasks:**
- [ ] Ensure ASCII diagrams are under 80 columns and render in GitHub preview
- [ ] Add link to `ARCHITECTURE.md` in `README.md` (minor tracked-file change)
- [ ] Final review of all section completeness

## Files Summary

| File | Action | Purpose |
|------|--------|---------|
| `docs/ARCHITECTURE.md` | Create | Sole deliverable — comprehensive architecture reference (~600-800 lines) |
| `README.md` | Modify (minor) | Add link to ARCHITECTURE.md for discoverability |

## Definition of Done

### P0 — Must ship

- [ ] `docs/ARCHITECTURE.md` exists with all 12 sections populated
- [ ] Three ASCII diagrams included: crate dependency graph, data flow pipeline, threading model
- [ ] All 5 crates documented with responsibilities, key modules, and file paths
- [ ] Optimization pipeline (LBF → GLS → Compression) clearly explained
- [ ] At least 7 key architectural decisions documented with alternatives and rationale
- [ ] Glossary defines all domain-specific terms
- [ ] Extension points table identifies where new features plug in
- [ ] Every file path in the document verified to exist in the current repo
- [ ] Document accurately reflects code on `main` (no aspirational content)
- [ ] `cargo test --workspace` passes (no code changes)
- [ ] `git diff` (excluding `docs/` and `README.md`) shows no changes

### P1 — Should ship

- [ ] README.md updated with link to ARCHITECTURE.md
- [ ] ASCII diagrams render correctly in GitHub Markdown preview
- [ ] A Rust developer unfamiliar with the project can read top-to-bottom without needing to reference external material
- [ ] No section requires understanding a concept introduced later in the document

### Cut if behind schedule

- Extension points table (Section 11) — can be inferred from crate deep dives
- Glossary (Section 12) — useful but not critical for architecture understanding
- README.md update — can be done post-sprint

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Document drifts from code over time | High | Medium | Structure sections independently so future sprints update individual sections. Include file paths for easy verification. |
| Type or module names documented inaccurately | Medium | Medium | Phase 3 verification step checks every reference against source. |
| Excessive detail makes document hard to maintain | Medium | Low | Target 600-800 lines. Keep at module level, not individual function level. jagua-rs as API surface only. |
| ASCII diagrams break in narrow terminals | Low | Low | Keep under 80 columns. Test in GitHub preview. |

## Security Considerations

No security implications. Documentation-only sprint with no code changes, no new dependencies, no user input handling, no network access. Ensure the document does not inadvertently include sensitive file paths or credentials (negligible risk for a local desktop app).

## Dependencies

### Internal
- **Sprints 001-004 complete**: The document describes the cumulative architecture. All features must be stable.

### External
- None. Markdown authoring requires no additional tooling.

## Open Questions

All open questions from the intent document have been resolved:

| Question | Resolution | Source |
|----------|------------|--------|
| Depth of jagua-rs coverage | API surface only. Reference academic paper for internals. | Interview |
| Diagram format | ASCII art. Portable, diff-friendly, no tooling. | Interview + all three drafts converged |
| Historical context | Omit sprint history. Architecture doc covers current state only. Link to sprint docs for history. | Gemini draft, validated in critique |
| Optimization algorithm detail | Conceptual overview. Order-of-magnitude timings. No pseudocode. | Claude draft, validated by Codex critique |
