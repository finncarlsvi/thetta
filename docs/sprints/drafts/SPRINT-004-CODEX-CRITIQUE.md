# Sprint 004 Draft Critique: `SPRINT-004-CLAUDE-DRAFT.md`

## 1. Strengths

- **Dual-mode engine via enum dispatch (line 208)** is the right call. The rationale for rejecting trait objects (cloned in GLS, stored by value) is sound and matches the existing `GLSOptimizer` which owns a `NestingEngine` (gls.rs:33). This preserves backward compatibility cleanly.

- **Per-sheet GLS via temporary `SppEngine` (lines 602-642)** is a pragmatic shortcut. Reusing the existing `GLSOptimizer` and `SppEngine` unmodified avoids touching the most complex and fragile code in the nesting crate. The extract-optimize-writeback pattern is easy to test in isolation.

- **Direct `Container::new_strip()` construction (lines 291-297)** correctly bypasses the broken BPP import path. The draft correctly identifies that `Importer.import_container()` does not exist and avoids unnecessary shims.

- **The `BPLayoutType::Closed { bin_id: 0 }` pattern for opening new bins (line 210)** aligns with how `BPProblem::place_item()` actually works (problem.rs:46-53). The draft shows understanding of the real API.

- **Risk table (lines 1162-1173)** identifies the `register_layout` visibility issue and the demand=1 re-placement panic, both of which are real. The piece ID collision risk across STEP files is also correctly flagged.

- **Use cases (lines 11-54)** are concrete and user-visible. UC4 (benchmark backward compat) is especially important and often forgotten.

## 2. Weaknesses

- **`open_sheet()` implementation is broken (lines 366-379).** The draft shows two conflicting approaches in the same code block: first attempting `self.problem.place_item(BPPlacement { ... })` with an incomplete struct (`...`), then calling `self.problem.register_layout(layout)` which is private. The comment "Actually: construct Layout from Container and register it" is handwaving over the core API gap. Neither approach works as written. The draft should commit to one strategy: either (a) make `register_layout` pub (acknowledged in Open Questions line 1212), or (b) always open bins via `place_item` with `BPLayoutType::Closed` on the first piece that needs that sheet (no pre-opening). Option (b) is actually simpler and requires zero jagua-rs changes.

- **The delegating `NestingEngine` enum has a flawed API surface (lines 440-454).** Methods like `collision_cost()`, `place_item()`, `strip_width()`, `fit_strip()`, and `change_strip_width()` are SPP-specific concepts that do not translate to BPP. The draft says these "delegate to the first open sheet (sheet 0) by default" (line 458), but this is semantically wrong for BPP and will cause subtle bugs. `fit_strip()` on a fixed-size bin is a no-op at best, a panic at worst. `change_strip_width()` on BPP bins is meaningless. A cleaner design: the enum exposes only the genuinely shared methods, and BPP-specific/SPP-specific methods are accessed via pattern matching on the enum variant.

- **Task 0.2 (lines 243-256) is dead code.** The draft proposes adding an `import_container()` method that immediately calls `unimplemented!()`, then says "in practice, we bypass the import path entirely." This task should be deleted -- it adds confusion and zero value.

- **GLS per-sheet approach creates N independent sub-problems (lines 602-642) but the time budget is undefined.** The draft's Open Question 3 (line 1216) asks about time allocation but never resolves it. With 5 sheets, does GLS run for 60s total or 300s? This is not a stretch question -- it directly affects whether the optimizer thread blocks the UI for an acceptable duration.

- **Multiple STEP file accumulation (Phase 6, lines 889-970) is significantly underscoped.** The `StepSession` struct (lines 901-907) and per-file thickness selection add substantial state management complexity. The export path (Task 6.3, lines 942-953) is described in 4 lines of pseudocode with "iterates sessions for piece lookup" but the actual mapping from `PieceId` back to `(session_index, solid_index)` is non-trivial, especially after the draft's own PieceId-offset scheme (line 1214). This is a mini-feature deserving its own phase with real code, not a handwave.

- **LOC estimate for `engine.rs` (+300, line 1102) is unrealistic.** The `BppEngine` needs: constructor, `place_item_on_sheet`, `force_place_item_on_sheet`, `collision_cost_on_sheet`, `remove_item`, `placement_of`, `current_result`, `open_sheet`, `pieces_on_sheet`, `sheet_count`, `layout_key`, plus the entire `NestingEngine` enum with delegation for 12+ methods. That is easily 400-500 lines. The `SppEngine` rename alone (identical code, just moved into a new struct) accounts for ~230 lines of the existing engine.

## 3. Gaps in Risk Analysis

- **Missing: `Container::new_strip` creates a strip, not a rectangular bin.** The function name is `new_strip` and it creates a simple rectangular region, which happens to work for fixed bins. But the CDE it constructs uses `HazardEntity::Exterior` with no notion of "this is a closed bin." If jagua-rs ever distinguishes strips from bins internally (e.g., in layout area calculations or solution validation), this could silently produce wrong results. The draft should verify that `Layout::new(container)` behaves identically for strip-constructed containers and "real" bin containers.

- **Missing: `BPProblem::remove_item()` auto-closes empty layouts (problem.rs:69-72).** When a piece is removed and the layout becomes empty, the layout is deregistered and the `LayKey` becomes invalid. The `BppEngine.layout_order` vec will contain stale keys. The draft's GLS writeback loop (lines 628-634) calls `force_place_item_on_sheet` which presumably removes then re-places, which can trigger this auto-close-then-reopen churn. This is not mentioned anywhere.

- **Missing: `item_demand_qtys` underflow panic.** `BPProblem::register_included_item()` (problem.rs:216-218) does `self.item_demand_qtys[item_id] -= 1` with no bounds check. If a piece with demand=1 is placed twice without removal, this panics via arithmetic underflow in debug mode. The draft mentions this risk (line 1172) but underestimates it: the GLS per-sheet approach extracts pieces, creates sub-engines, runs GLS, then writes back. During writeback, `force_place_item_on_sheet` must remove the old placement first, but the piece may have been placed on a different sheet than expected if `pieces_on_sheet()` returns stale data.

- **Missing: `SlotMap` key stability.** `BPProblem.layouts` is a `SlotMap<LayKey, Layout>`. When layouts are removed and re-inserted, old `LayKey` values are not reused (slotmap guarantees this). But the draft's `layout_order: Vec<LayKey>` assumes keys are stable for the lifetime of the engine. If GLS writeback triggers layout removal (empty layout auto-close), the key in `layout_order` becomes dangling.

- **Missing: jagua-rs `bpp` feature may not compile cleanly alongside `spp`.** The `LayKey` type is defined in both `probs::bpp::entities::problem` and `probs::mspp::entities::problem` (see grep results). If both are enabled, there could be name collisions or type confusion. The draft does not verify that `features = ["spp", "bpp"]` compiles cleanly.

- **Missing: `Bin.stock` decrement on layout open.** `BPProblem::open_bin()` (problem.rs:224-225) decrements `bin_stock_qtys[bin_id]`. With stock=100, opening 101 sheets panics via underflow. The draft mentions the 100-sheet cap (line 1182) but does not add an explicit check before opening a new bin in `run_lbf_bpp`. The LBF loop (lines 493-527) has no guard.

## 4. Missing Edge Cases

- **Zero pieces.** What happens when `run_lbf_bpp` is called with an empty piece list? The draft opens a sheet (line 489-491) even with no pieces, producing a `NestingResult` with one empty sheet. Is that the intended behavior? The SPP path does not create empty sheets.

- **Single piece larger than the sheet.** The draft mentions this (line 519: "leave unplaced") but the test (Task 8.1 line 1020) is listed as a unit test stub with no implementation. The `find_lbf_position` grid search will fail, the piece will trigger `open_sheet()`, the grid search will fail again on the new sheet, and the piece lands in `unplaced`. But the newly opened sheet is now empty, and `BPProblem::remove_item` auto-closes empty layouts -- except no item was placed, so there is nothing to remove. We now have a phantom empty layout in `layout_order`. This needs explicit handling.

- **All pieces identical (demand > 1 per item).** The draft uses demand=1 per piece (line 282). But if two pieces have identical polygons, jagua-rs `BPInstance` could potentially deduplicate them into one item with demand=2. The draft's item construction uses per-piece items regardless of shape identity, which is correct, but this should be explicitly noted as a design choice.

- **Sheet config change while optimizer is running.** The draft says "nesting restarts with the new sheet size" (UC6, line 52) but does not describe how the cancel-and-restart interacts with the new BPP engine. If the user changes sheet dimensions mid-optimization, the old `BppEngine` with the old container dimensions is still running on the background thread. The draft should confirm the existing cancel-and-restart mechanism handles this.

- **Loading a benchmark JSON file after loading STEP files (mixed mode).** The draft says benchmarks set `NestingMode::Spp` (line 218). But if STEP pieces are already accumulated in `self.pieces`, switching to SPP mode nests all accumulated pieces on a single strip. Is that intentional? Should loading a benchmark file clear previous pieces?

- **STEP export with unplaced pieces.** If some pieces could not be placed (larger than any sheet), the export iterates `result.sheets` but those pieces have no placement. The draft does not address whether the export should warn, skip, or error.

## 5. Definition of Done Completeness

- **P0 is missing a critical criterion:** "BPP engine correctly tracks piece-to-sheet mapping and produces accurate `NestingResult` with correct piece IDs per sheet." The current P0 items test the plumbing but not the data integrity of the result.

- **P0 is missing:** "No empty sheets in `NestingResult.sheets` (sheets with zero placements are removed before returning the result)."

- **P0 should include:** "GLS does not increase the number of sheets (per-sheet GLS only improves placement quality within existing sheets)."

- **P1 is missing:** "Loading a second STEP file does not invalidate existing piece IDs or placements from the first file."

- **P1 "Integration test: 20+ pieces across 3+ sheets" (line 1146)** should be P0. Multi-sheet placement is the core deliverable.

- **No performance criterion.** The intent says ~1300 LOC per sprint (line 10). At ~1120 estimated LOC, the sprint is within range, but there is no wall-clock time target for the BPP pipeline. For 100 pieces across 5 sheets, is 10 seconds acceptable? 60? This matters for user experience.

- **No criterion for "pieces are placed on the correct sheet in the viewport."** Rendering all sheets is tested, but correctness of which piece appears on which sheet is not explicitly verified.

## 6. Architecture Concerns

- **The `NestingEngine` enum delegation pattern will cause a combinatorial explosion of boilerplate.** With 12+ methods each doing `match self { Spp(e) => e.foo(), Bpp(e) => e.foo() }`, the enum itself becomes ~150 lines of pure delegation. A trait with two impls would be cleaner and is feasible if the trait is not object-safe (use generics instead of `dyn`). The draft rejected trait objects but did not consider generic `fn run_lbf<E: Engine>(engine: &mut E, ...)`.

- **`BppEngine.layout_order: Vec<LayKey>` is a fragile parallel data structure.** It must stay in sync with `BPProblem.layouts` at all times. Any operation that removes a layout (item removal causing auto-close, `remove_layout`, `restore`) will invalidate it. A safer design: derive sheet ordering from `BPProblem.layouts.keys()` on demand, accepting the SlotMap iteration order, or use a `BTreeMap<usize, LayKey>` indexed by sheet number.

- **Per-sheet GLS via temporary `SppEngine` (lines 612-622) silently changes the collision detection geometry.** The sub-engine is created with `SppEngine::new_with_orientations`, which internally calls `pieces_to_sp_instance` and creates a new `SPInstance` with a new strip container. The CDE grid resolution, hazard configuration, and NFP calculations are all recomputed from scratch. Pieces seeded via `force_place_item` may have subtly different collision behavior than in the original BPP layout. This could cause GLS to accept placements that are invalid in the BPP context, or reject valid ones.

- **`pieces_to_bp_instance` applies separation via `ShapeModifyConfig.offset` on the container (line 287-288) AND potentially on items via the `Importer`.** The draft inherits the same `Importer` used for SPP which already applies item-level separation. Double-application of separation (once on items, once on the container) would shrink the effective sheet area. The SPP path in `pieces_to_sp_instance` should be checked to see how separation is handled there, and the BPP path must match.

## 7. Implementation Feasibility

- **Total LOC estimate of ~1120 is optimistic by 30-50%.** The engine refactor alone (rename + BppEngine + enum + delegation) is realistically 500+ LOC. Multi-file STEP accumulation with session tracking, ID offsetting, and per-file thickness UI is at least 150 LOC, not the 120 estimated for all of state.rs changes. The viewport changes need to handle click-to-select across sheets (mapping screen coordinates back to world coordinates on the correct sheet), which is more than the 60 LOC estimated. Realistic total: 1400-1600 LOC.

- **Phase 6 (multi-STEP accumulation) should be cut or moved to a separate sprint.** It is listed as P1 (line 1157: "Cut if Behind Schedule") but the implementation plan interleaves it with core BPP work. The `StepSession` struct introduces new state management that is orthogonal to bin packing. Cutting it cleanly requires that the draft's Phase 6 has no dependencies from other phases, which is true but not explicitly stated.

- **The draft proposes 8 phases.** Previous sprints delivered in ~1300 LOC with simpler phase structures. Eight phases with inter-dependencies (Phase 3 depends on Phase 2 depends on Phase 1) create a long critical path. If Phase 1 (engine refactor) takes longer than expected, everything downstream slips. Consider merging Phases 0+1 and Phases 2+3 to reduce handoff overhead.

- **Task 0.2 proposes modifying jagua-rs (`import_container` shim) and Open Question 1 proposes making `register_layout` public.** Both are changes to the jagua-rs fork. The draft says "no source changes, only feature flag" (line 1202) which contradicts these proposals. The draft should be explicit: jagua-rs fork will be modified (at minimum, `register_layout` visibility change), and this should be in the files summary table.

- **The `find_lbf_position` function (lines 531-561) reimplements the LBF grid search from scratch** instead of reusing the existing `run_lbf` grid logic in lbf.rs. The existing LBF (lbf.rs:26+) has the same grid search pattern but with the actual rotation sampling logic (`rotation_range_to_samples`). Duplicating this logic risks divergence. Better: extract the grid search into a shared helper used by both SPP and BPP LBF paths.
