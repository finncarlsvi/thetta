# Sprint 003 Merge Notes

## Drafts Compared

- **SPRINT-003-CLAUDE-DRAFT.md** (1383 lines, opus-4.6)
- **SPRINT-003-CODEX-DRAFT.md** (490 lines, gpt-5.2)

## Claude Draft Strengths

- Extremely detailed FFI wrapper design (`occt.rs` module with RAII patterns, Drop impls)
- Concrete `ExtractionConfig` with configurable mesh tolerance, face heuristic
- Thorough test plan with 5 specific test STEP files and actual test code
- Detailed `build.rs` and system dependency install/verify instructions
- Risk table with 8 specific risks including thread safety (`Send`/`Sync`) concerns
- `StepSession` design that cleanly separates OCCT ownership from Rust data

## Codex Draft Strengths

- Cleaner scope boundaries with explicit non-goals section
- UC3 (mixed input sources) explicitly addresses JSON + STEP coexistence policy
- Feature-gated OCCT compilation (`--features occt`) so other crates can build without OCCT
- `StepPieceRef` as a clean parallel mapping between PieceId and OCCT solids
- More practical phasing: Phase 0 (bring-up) is isolated and testable before any real work
- Option B for test fixtures (generate at test-time using OCCT primitives) avoids binary STEP files in repo

## Interview Refinements Applied

- Sprint 2 is just started -> Sprint 3 prerequisites clearly stated
- User confirmed `opencascade-sys` preference -> both drafts aligned on this
- Neither OCCT nor test files set up -> Phase 0 (bring-up) is critical and must include setup docs
- macOS ARM64 only -> system dependency section includes Homebrew instructions

## Merge Decisions

1. **Architecture**: Use Claude's `occt.rs` wrapper module approach (better safety) but with Codex's `StepModel` + `StepPieceRef` naming (cleaner API)
2. **Feature gating**: Adopt Codex's `--features occt` approach so workspace builds without OCCT installed
3. **Mixed sources**: Include Codex's UC3 — important for real-world usage
4. **Test fixtures**: Primary approach is generating fixtures at test-time using OCCT (Codex Option B), with a secondary recommendation to also check in small files for CI
5. **Phasing**: Use Codex's Phase 0 (pure bring-up) + Claude's detailed implementation phases
6. **Thread safety**: Use Claude's recommendation (c) — extract Rust data during import, keep OCCT solids behind Arc<Mutex<>> only for export
7. **Risk table**: Use Claude's detailed format with Codex's additional risk about mixed-source export policy
8. **Files**: Use Claude's file breakdown (more realistic LOC estimates) with Codex's feature flag additions
