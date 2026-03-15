# Sprint 004 Intent: Multi-Sheet Bin Packing + Multiple STEP Files

## Seed

Implement multi-sheet bin packing so pieces overflow onto additional fixed-size sheets, and support loading multiple STEP files with pieces accumulating across files. These are the last two gaps between the current implementation and the IDEA.md product vision.

## Orientation Summary

- **Current state**: 3 sprints complete (~14.7K LOC). Full pipeline works: JSON/STEP import → 2D nesting (jagua-rs LBF+GLS) → STEP export. Currently uses SPP (strip packing) — single infinitely-wide strip, not fixed-size sheets.
- **Recent direction**: Sprint 001 (engine+GUI) → Sprint 002 (jagua-rs+benchmarks) → Sprint 003 (STEP import/export via OpenCASCADE). Each sprint ~1300 LOC.
- **Key architectural finding**: jagua-rs already has full BPP (bin packing) support: `BPInstance`, `BPProblem`, `BPPlacement` with multiple layouts/bins, stock quantities, and cost tracking. `NestingResult` already has `Vec<Sheet>`. The type system is ready.
- **Constraints**: OCCT single-threaded and feature-gated. macOS ARM64 target. Sprint-based development with consensus planning.

## Relevant Codebase Areas

### Must Change
- `crates/thetta-nesting/src/engine.rs` — `NestingEngine` wraps `SPProblem`. Must add BPP mode using `BPProblem` with multiple layouts.
- `crates/thetta-nesting/src/conversion.rs` — `pieces_to_sp_instance()` creates `SPInstance`. Need `pieces_to_bp_instance()` creating `BPInstance` with `Bin` definitions.
- `crates/thetta-nesting/src/lbf.rs` — LBF places into single strip. Must handle multiple bins: fill one sheet, overflow to next.
- `crates/thetta-nesting/src/gls.rs` — GLS optimizes single layout. Must handle multi-layout optimization (move pieces between sheets).
- `crates/thetta-nesting/src/compression.rs` — Strip compression. May need bin-packing equivalent (minimize sheet count).
- `crates/thetta-nesting/src/runner.rs` — Pipeline orchestration. Must pass BPP vs SPP mode through.
- `crates/thetta-app/src/viewport.rs` — Renders first sheet only (`.first()`). Must render all sheets.
- `crates/thetta-app/src/state.rs` — Stats read first sheet only. Must aggregate across sheets. STEP loading should accumulate files.
- `crates/thetta-app/src/panels.rs` — Need sheet navigation/count display.

### Reference (jagua-rs BPP)
- `crates/jagua-rs/src/probs/bpp/instance.rs` — `BPInstance`, `Bin`
- `crates/jagua-rs/src/probs/bpp/problem.rs` — `BPProblem` with `SlotMap<LayKey, Layout>`
- `crates/jagua-rs/src/probs/bpp/solution.rs` — `BPSolution`
- `crates/jagua-rs/src/probs/bpp/bp_placement.rs` — `BPPlacement`, `BPLayoutType`

### Unchanged
- `crates/thetta-geometry/` — `NestingResult` already supports `Vec<Sheet>`, no changes needed.
- `crates/thetta-step/` — Import/export pipeline. Export already iterates sheets.
- `crates/jagua-rs/` — BPP support exists, no modifications expected.

## Constraints

1. **Backward compatibility**: Benchmark mode (sparrow JSON) must continue using SPP (strip packing). BPP mode is for STEP/native workflows with fixed sheet sizes.
2. **Existing test suite**: All existing tests must pass. Benchmarks remain SPP.
3. **OCCT thread safety**: No changes to OCCT threading model.
4. **Sheet config**: Default 2440mm × 1220mm per IDEA.md. Single sheet template replicated as needed (not per-sheet configs).
5. **jagua-rs BPP API**: Use the existing BPP types without modifying the jagua-rs fork.

## Success Criteria

1. When pieces don't fit on one 2440×1220mm sheet, they overflow to additional sheets automatically.
2. Viewport renders all sheets (side-by-side or with navigation).
3. Stats show per-sheet and aggregate utilization.
4. User can load multiple STEP files; pieces accumulate.
5. STEP export produces one `.step` file per sheet (already works if `NestingResult.sheets` is correct).
6. Benchmark mode still uses SPP with strip packing semantics.
7. GUI shows sheet count and allows navigation between sheets.

## Verification Strategy

- **Unit tests**: BPP conversion, multi-bin LBF placement, piece overflow logic.
- **Integration tests**: Load pieces exceeding one sheet → verify multiple sheets in result, all pieces placed, no overlaps.
- **GUI verification**: Launch app, load STEP file with many parts, verify multiple sheets render, navigate between them.
- **Regression**: All existing tests pass (`cargo test --workspace`).
- **Benchmark regression**: `jakobs1` and other benchmark instances still produce valid SPP results.

## Uncertainty Assessment

| Factor | Level | Reasoning |
|--------|-------|-----------|
| **Correctness** | Medium | jagua-rs BPP API exists but we haven't used it yet. Integration details may surprise us. |
| **Scope** | Low | Well-defined: switch engine to BPP for fixed-sheet mode, update UI to show multiple sheets. |
| **Architecture** | Medium | Dual-mode engine (SPP for benchmarks, BPP for sheets) adds complexity. LBF/GLS adaptation for multi-bin is non-trivial. |

## Open Questions

1. **Dual-mode engine or separate engines?** One `NestingEngine` with SPP/BPP enum, or two separate types? Leaning toward enum variant to keep runner/optimizer code unified.
2. **GLS across sheets**: Should GLS move pieces between sheets to balance utilization, or optimize each sheet independently? Moving between sheets is more powerful but more complex.
3. **Viewport layout**: Render sheets side-by-side horizontally, vertically stacked, or use tabs/pagination? Side-by-side is simplest and most visual.
4. **Sheet overflow strategy in LBF**: When a piece doesn't fit on current sheet, open a new bin via BPP. Should we try all existing sheets before opening new ones?
5. **Compression equivalent for BPP**: SPP has strip compression. BPP equivalent is minimizing sheet count. Should we try removing the last sheet and redistributing pieces?
