# Sprint 004 Merge Notes

## Draft Comparison

### Claude Draft Strengths
- Dual-mode enum design is well-reasoned (cloned in GLS, stored by value)
- Correctly identified `register_layout` as private and `import_container` as missing
- Per-sheet GLS via temporary `SppEngine` is pragmatic and reuses existing code
- Direct `Container::new_strip()` bypasses broken BPP import path
- LOC estimate closer to historical sprint size (~1120)

### Codex Draft Strengths
- Separate `BinPackEngine` struct avoids SPP-method leakage (fit_strip, change_strip_width are meaningless for BPP)
- Fill-and-spill LBF tries all existing sheets before opening new one — strictly better
- `NestingMode` enum for explicit dispatch (not inferred from sheet dimensions)
- Sheet-elimination compression (redistribute last sheet's pieces) is a good BPP equivalent
- Globally unique piece IDs via `next_piece_id` counter solves cross-file conflicts
- Self-corrected on `SheetConfig::default()` finite-width issue

### Valid Critiques Accepted
1. **jagua-rs BPP is dead code** (both critiques) — `probs/mod.rs` only declares `pub mod spp;`, missing deps (slotmap, itertools, rayon). ACCEPTED: Use multiple independent SPP instances instead.
2. **`open_sheet()` implementation broken** (Codex critique) — two conflicting approaches, neither works. ACCEPTED: Moot since we're not using BPP.
3. **LOC estimates optimistic by 30-50%** (both critiques) — realistic ~1300-1500 LOC. ACCEPTED: Revise upward.
4. **Multi-STEP accumulation underscoped** (both critiques) — orthogonal to bin packing. ACCEPTED: Deferred per user decision.
5. **`layout_order` fragile parallel data structure** (Codex critique) — stale LayKey risk. ACCEPTED: Moot since we're not using BPP.
6. **Pieces-too-large warning missing** (Claude critique) — no user feedback when piece exceeds sheet. ACCEPTED: Add to P1.
7. **LBF grid search duplication** (Codex critique) — extract shared helper. ACCEPTED: Reuse existing `find_lbf_position` logic.
8. **Per-sheet GLS time budget undefined** (Codex critique). ACCEPTED: Define as total_time / sheet_count.

### Valid Critiques Rejected
1. **Cross-sheet GLS should be P0** (Claude critique) — user explicitly chose per-sheet independent optimization. Per-sheet GLS still improves within-sheet utilization. Cross-sheet is a future enhancement.
2. **Trait abstraction between engines** (both critiques) — overhead not justified for two structs. Method duplication is minimal since LBF/GLS reuse existing SPP code paths unchanged.
3. **Tabbed viewport over side-by-side** (Codex draft) — user explicitly chose side-by-side.

## Interview Refinements Applied
1. **Multiple SPP instances** instead of jagua-rs BPP — dramatically simplifies the sprint
2. **Side-by-side viewport** with horizontal layout
3. **Per-sheet independent GLS** — reuses existing optimizer unchanged
4. **Multi-STEP deferred** to Sprint 005

## Architecture Decision: Multiple SPP Instances

Since jagua-rs BPP is non-functional dead code, and the user chose multiple SPP instances:

- Each sheet gets its own `NestingEngine` (existing SPP engine, unchanged)
- `MultiSheetEngine` orchestrates: Vec<NestingEngine>, one per sheet
- LBF fills sheet 1, overflows to sheet 2, etc.
- GLS runs independently on each sheet's engine (existing `GLSOptimizer` unchanged)
- Compression runs independently per sheet (existing `compress_strip` unchanged)
- No jagua-rs fork modifications needed
- `NestingMode` enum for dispatch in runner (SPP vs MultiSheet)

This is dramatically simpler than either draft proposed.
