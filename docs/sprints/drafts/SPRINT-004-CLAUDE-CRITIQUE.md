# Sprint 004 Codex Draft Critique

This document reviews the Codex draft for Sprint 004 against the intent document, the actual jagua-rs BPP source code, and the current thetta-nesting codebase.

---

## 1. Strengths

1. **Separate `BinPackEngine` struct is the right call.** The draft correctly identifies (Design Decision 1, line ~288) that the SPP engine's `strip_width`/`fit_strip`/`change_strip_width` methods are semantically incompatible with BPP. Two clean structs sharing `NestingResult` as output is the right architecture. The rationale is well-articulated.

2. **Fill-and-spill LBF strategy is sound.** Trying all existing sheets before opening a new one (Design Decision 2, line ~290) is strictly better than always appending to the last sheet. The draft correctly identifies that this fills gaps in earlier sheets.

3. **`NestingMode` enum recovery is good engineering.** The draft initially proposes dispatch based on `sheet.width.is_finite()`, then catches the bug that `SheetConfig::default()` already has finite width (line ~1026), and self-corrects to an explicit `NestingMode` enum. This is the correct approach and shows careful thinking about backward compatibility.

4. **`BPProblem` API usage is mostly accurate.** The `place_item` call with `BPLayoutType::Closed { bin_id: 0 }` to open new sheets (line ~472) and `BPLayoutType::Open(sheet_key)` for existing sheets (line ~495) correctly matches the actual API at `crates/jagua-rs/src/probs/bpp/entities/problem.rs:45-63`.

5. **`pk_map` storing `(LayKey, PItemKey)` tuples is correct.** The draft recognizes that `BPProblem::remove_item` requires both keys (Risk table, line ~1412), and the `pk_map: Vec<Option<(LayKey, PItemKey)>>` design handles this properly.

6. **Tab-based viewport over side-by-side is the right UX choice.** Design Decision 5 (line ~296) correctly argues that side-by-side becomes unusable with 5+ sheets. Hiding tabs when only 1 sheet is a good detail.

7. **Cross-sheet GLS sampling distribution is well thought out.** Dividing `samples_per_move` across sheets (line ~761) avoids the O(N*M) blowup that naive cross-sheet sampling would cause.

## 2. Weaknesses

1. **The BPP module in jagua-rs is dead code and will not compile.** This is the single biggest problem with the draft. `crates/jagua-rs/src/probs/mod.rs` contains only:
   ```rust
   #[cfg(feature = "spp")]
   pub mod spp;
   ```
   There is no `pub mod bpp;` declaration -- conditional or otherwise. The BPP source files exist on disk (`crates/jagua-rs/src/probs/bpp/`) but are never compiled into the crate. Furthermore, the BPP import code at `probs/bpp/io/import.rs:6` uses `rayon::prelude::*` and `itertools::Itertools`, neither of which are in `crates/jagua-rs/Cargo.toml` (which only lists `anyhow`, `ordered-float`, and `serde`). The `BPProblem` at `probs/bpp/entities/problem.rs:10` uses `slotmap`, also absent from dependencies. **The entire BPP module is non-functional code that has never been compiled.** The draft treats these types as ready-to-use ("jagua-rs BPP types exist but are unused" -- line ~9). In reality, enabling BPP requires: (a) adding `pub mod bpp;` to `probs/mod.rs` with appropriate feature gating, (b) adding `slotmap`, `itertools`, and `rayon` to `jagua-rs/Cargo.toml`, (c) adding `import_container` to the `Importer` struct (it doesn't exist -- only `import_item` does), and (d) fixing any compilation errors in code that has never been through `rustc`. This is easily 100-200 LOC of jagua-rs modifications and a significant risk the draft does not account for.

2. **`Importer::import_container` does not exist.** The draft's "primary path" for `pieces_to_bp_instance` (lines ~369-398) calls `import_bp_instance(&importer, &ext_instance)` which in turn calls `importer.import_container(&ext_bin.base)` (at `bpp/io/import.rs:44`). But `Importer` only has `import_item` -- there is no `import_container` method. The "fallback" path using `Container::new_strip` (lines ~405-432) is actually the only viable path, but the draft presents it as a backup rather than the primary approach.

3. **`Container::new_strip` creates containers with `id` from the parameter, but `Bin::new` sets `id` from `container.id`.** At `entities/bin.rs:18-24`, `Bin::new` does `Self { id: container.id, ... }`. The draft's fallback code passes `0` as the container ID (line ~422), which is correct for a single bin type. But the draft never discusses what happens if `Container::new_strip` returns `Err` -- the `?` operator is used but `pieces_to_bp_instance` returns `Result<BPInstance>`, so this is fine. However, the `ShapeModifyConfig` construction (line ~416-420) applies separation as `offset: Some(separation / 2.0)` to the container, when separation should be applied to the items (as the SPP path does via `Importer`). Container offset would shrink the usable bin area, not add spacing between pieces.

4. **The `to_internal_bp_transformation` / `to_external_bp_transformation` functions are unnecessary duplication.** The existing `to_internal_transformation` and `to_external_transformation` functions take `&SPInstance` but only use the `Instance` trait method `instance.item(item_id)`. Since `BPInstance` also implements `Instance` (at `bpp/entities/instance.rs:46`), these functions could be made generic over `impl Instance` rather than duplicated. The draft proposes four functions doing the same thing when two generic ones would suffice.

5. **`compress_sheets` rebuilds the engine from scratch each iteration, losing GLS improvements.** The compression function (lines ~853-924) creates a fresh `BinPackEngine` and re-runs LBF. This discards any GLS refinements to piece positions on earlier sheets. It would be better to operate on the existing engine state: remove pieces from the victim sheet, then try to place them into existing layouts.

6. **`find_lbf_position_new_sheet` is described but never implemented.** Line ~693 says "we can just place at (0, 0, 0) as a fallback," which is not robust. A large piece that doesn't fit at the origin of a new sheet will silently fail. The implementation needs to actually search the grid on the new sheet, but the code for opening a temporary layout, probing, and potentially rolling back is non-trivial and the draft hand-waves it.

7. **GLS `total_overlap` method is referenced but not defined.** The `GLSBinPackOptimizer` calls `self.total_overlap()` (line ~737) but this method is never shown. For BPP, computing total overlap requires iterating all layouts and all placed items per layout -- more complex than the SPP version at `gls.rs:198-208` which only has one layout. The draft omits this.

8. **Multi-file STEP loading replaces pieces vs. accumulates -- unclear interaction with re-nesting.** The draft says pieces "accumulate" (UC5, line ~47), but the current `apply_step_import` replaces pieces. The draft shows ID remapping (lines ~1150-1161) but doesn't address what happens when the user re-nests after loading a second file: are pieces from file 1 still present? What if the user changes thickness on file 1 -- does it discard file 2's pieces? The interaction model between `StepFileSession` vec and the global `pieces` vec needs more detailed specification.

## 3. Gaps in Risk Analysis

1. **Missing risk: jagua-rs BPP code has never been compiled.** As detailed above, the BPP module is dead code. The risk table mentions "BPP types behind a feature gate" (line ~1404) as Medium likelihood, but the actual situation is worse: the module isn't even declared, and its dependencies are missing. This should be High impact / High likelihood.

2. **Missing risk: `rayon` dependency for BPP import.** `bpp/io/import.rs:6` uses `rayon::prelude::*` for `par_iter`. This introduces a new heavy dependency. Even if we bypass the import path and construct `BPInstance` directly, any future use of the BPP IO module will require rayon.

3. **Missing risk: `BPProblem::place_item` panics on stock exhaustion.** The risk table mentions this (line ~1406) and suggests generous stock. But the stock calculation `ceil(total_area / sheet_area) + 2` can be insufficient when pieces are large and irregular (poor packing efficiency). A piece wider than the sheet will consume one sheet per piece. If 10 such pieces exist with a stock of 5, it will panic. There is no stock check before calling `place_item` with `BPLayoutType::Closed`.

4. **Missing risk: `BPProblem::remove_item` auto-removes empty layouts.** At `problem.rs:66-77`, `remove_item` checks if the layout is empty after removal and automatically deregisters it, returning `BPLayoutType::Closed`. This means `pk_map` entries for other pieces on that layout become stale if the layout was removed. The draft's `BinPackEngine::remove_item` (line ~224) doesn't account for this cascading layout removal.

5. **Missing risk: SlotMap key stability.** `LayKey` is a SlotMap key. When a layout is removed and a new one inserted, the old key is invalidated. The draft's `sheet_keys()` returns a snapshot `Vec<LayKey>`, but if `remove_empty_sheets` or `remove_item` triggers layout removal during a GLS iteration, the keys held by `pk_map` entries may point to removed layouts. Accessing `self.problem.layouts[stale_key]` will panic.

6. **Missing risk: no `itertools` or `slotmap` in jagua-rs `Cargo.toml`.** The `BPProblem` type uses `SlotMap<LayKey, Layout>` and `slotmap::new_key_type!`, but `slotmap` is not a dependency. `BPProblem::new` uses `itertools::Itertools` (via `collect_vec()`). These must be added to `Cargo.toml` to compile the BPP module.

## 4. Missing Edge Cases

1. **Piece larger than the sheet.** If a single piece's bounding box exceeds sheet dimensions (e.g., a 2500mm piece on a 2440mm sheet), no rotation will make it fit. The LBF will fail to place it on any sheet, and `place_item_new_sheet` will also fail. The piece ends up in `unplaced`, but there is no user feedback explaining *why* it was unplaced. The draft should address this with a warning or error message.

2. **Zero pieces.** The `pieces_to_bp_instance` function checks `ensure!(!pieces.is_empty())`, but what if the user loads a STEP file with no extractable faces? The draft doesn't discuss this path through `BinPackEngine::new`.

3. **Single piece.** With one piece, fill-and-spill opens one sheet and places it. Cross-sheet GLS has nothing to optimize (one sheet, one piece). Sheet elimination has nothing to eliminate. This is a degenerate case that works but wastes compute. The draft should short-circuit for single-piece inputs.

4. **All pieces identical (high demand).** The BPP instance uses `demand: 1` per piece (line ~359). If the user loads a STEP file with 50 identical parts, they become 50 separate items with demand 1 each, not 1 item with demand 50. This is correct for the current piece model but loses the BPP optimization opportunity of treating identical items as a single item type with quantity > 1.

5. **Sheet with zero area.** If `sheet.width` or `sheet.height` is zero (but finite and positive fails the ensure), the stock calculation `ceil(total_area / sheet_area) + 2` divides by zero. The ensure check at line ~338 catches this, but the draft should note this explicitly.

6. **GLS never achieves zero overlap.** If pieces are complex and the grid is too coarse, GLS may never find a feasible solution for BPP. The SPP path handles this by falling back to LBF baseline. The BPP path (line ~980) also falls back, but the LBF baseline for BPP may have unplaced pieces if `find_lbf_position_new_sheet` was hand-waved to "just place at (0,0,0)." An infeasible baseline combined with an infeasible GLS means the user sees partially placed results with no explanation.

7. **Active sheet index after optimizer restart.** If the user starts a new optimization while viewing sheet 3, and the new result only has 2 sheets, the clamping logic (line ~1121) resets to sheet 1. But if the optimizer sends intermediate `Improved` results with varying sheet counts during the run, the active tab will jump around unpredictably.

## 5. Definition of Done Completeness

1. **Missing P0 criterion: jagua-rs BPP module compiles.** Given the dead-code situation, "BPP types are compilable and tested" should be an explicit P0 item.

2. **Missing P0 criterion: stock exhaustion guard.** "Opening a new bin never panics due to stock exhaustion" should be explicit, since the fallback is a panic deep in jagua-rs.

3. **Missing P1 criterion: pieces-too-large warning.** The user should be told when a piece cannot fit on any sheet.

4. **Missing criterion: round-trip correctness.** STEP import -> nest on multiple sheets -> STEP export -> re-import should produce the same piece count and no data loss. This is not listed anywhere.

5. **"Benchmark regression" (P0, line ~1372) is vague.** It should specify: "jakobs1 benchmark produces identical strip_length (within tolerance) to pre-Sprint-004 results," not just "valid SPP results."

6. **Cross-sheet GLS (P1, line ~1376) is too important to be P1.** Without it, the optimizer is just LBF + compression. GLS is the core optimization loop. If cross-sheet GLS doesn't ship, the quality of multi-sheet results will be poor. This should be P0 or the sprint scope should be reduced.

## 6. Architecture Concerns

1. **jagua-rs fork modification breaks the "unchanged" assumption.** The intent document (line ~36) and the draft (line ~90) both state "jagua-rs unchanged." This is false. At minimum, `probs/mod.rs` must add `pub mod bpp;`, and `Cargo.toml` must add `slotmap`, `itertools`, and likely `rayon`. The draft's risk table acknowledges "potentially add `bpp` feature flag" (line ~1432) but dramatically underestimates the scope. This should be called out as a prerequisite Phase -1 with its own LOC estimate.

2. **`Container::new_strip` semantics for BPP bins.** The function name `new_strip` implies infinite-width strip semantics, but the implementation (at `container.rs:19-43`) creates a bounded rectangle. This is fine mechanically, but conceptually confusing. The draft's Open Question 1 (line ~1447) correctly flags this but doesn't resolve it. For correctness, a placement test should verify that items placed partially outside the container boundaries register collisions against the `HazardEntity::Exterior` hazard. This should be a Phase 0 verification task, not an open question.

3. **No trait abstraction between `NestingEngine` and `BinPackEngine`.** Both engines share methods like `piece_count`, `piece_area`, `sheet_config`, `rotation_range`, `collision_cost`, `place_item`, `remove_item`, `placement_of`, and `current_result`. Without a shared trait, the LBF and GLS modules must be fully duplicated (`run_lbf` vs `run_lbf_bp`, `GLSOptimizer` vs `GLSBinPackOptimizer`). This creates a maintenance burden. A `NestingBackend` trait with associated types would allow shared LBF/GLS logic with generic dispatch. The draft dismisses this implicitly by choosing separate structs, but doesn't discuss the code duplication cost.

4. **`BPProblem::save()`/`restore()` not used.** The BPP `save`/`restore` API (at `problem.rs:80-158`) enables snapshotting and rolling back state, which is essential for robust GLS (save best feasible, restore if regression). The draft's `GLSBinPackOptimizer` tracks `best_feasible` as a `NestingResult` (line ~710), but never uses `save`/`restore` on the underlying `BPProblem`. This means the engine state drifts from the best known solution and cannot be recovered.

## 7. Implementation Feasibility

1. **LOC estimates are significantly underestimated.** The draft estimates ~993 LOC total. But:
   - jagua-rs BPP enablement (mod declaration, Cargo.toml deps, potential `import_container` implementation, compilation fixes): ~50-150 LOC, not counted.
   - `BinPackEngine` at 280 LOC seems low given the `NestingEngine` SPP equivalent is 230 LOC for simpler single-layout logic. Multi-layout operations (iterating sheets for `current_result`, handling layout lifecycle in `remove_item`, `remove_empty_sheets`) add complexity. 350-400 LOC is more realistic.
   - `GLSBinPackOptimizer` at 120 LOC delta is optimistic. The SPP `GLSOptimizer.run()` is 130 lines alone. The BPP version adds cross-sheet sampling, `total_overlap` across layouts, and empty-sheet cleanup. 180-200 LOC delta is more realistic.
   - Realistic total: ~1300-1500 LOC, consistent with prior sprints (~1300 LOC each).

2. **Phase ordering has a hidden dependency.** Phase 0 (BinPackEngine) depends on the BPP module compiling, which requires jagua-rs modifications. This prerequisite is not listed as a phase and has no LOC estimate.

3. **The compression phase (Phase 3) rebuilds entire engines.** This is expensive: constructing `BPInstance` involves building collision detection engines for each container. For N sheets, each compression attempt creates a new engine, imports all items, and runs LBF. With 5+ sheets this could take 5-10x the initial LBF time. The draft estimates 60 LOC for compression, which covers the logic but not the performance implications.

4. **No mention of `Container::new_strip` returning `Result`.** The `new_strip` function returns `Result<Self>` (at `container.rs:25`), meaning it can fail (e.g., if width or height is zero/negative, `Rect::try_new` fails). The draft's fallback code uses `?` but doesn't discuss error recovery or propagation to the user.

5. **`optimizer.rs` is currently a trivial 18-line wrapper.** The draft proposes adding `NestingMode` dispatch there (Phase 4.2), which is straightforward. But the file also needs new imports for `run_nesting_bp` and the `NestingMode` enum. The +15 LOC estimate is reasonable for this file.

## Summary

The draft demonstrates solid architectural thinking, particularly around the separate-engine design, fill-and-spill LBF, and the self-correcting `NestingMode` dispatch. However, it is built on a fatally incorrect assumption: that jagua-rs BPP types are functional and ready to use. They are not. The BPP module is undeclared dead code with missing dependencies (`slotmap`, `itertools`, `rayon`) and a missing `import_container` method. Addressing this changes the sprint's risk profile from "straightforward integration" to "partial upstream enablement + integration," adding 100-200 LOC and an unknown amount of debugging time for code that has never been compiled.

The draft should be revised to:
1. Add a Phase -1 for jagua-rs BPP enablement with explicit tasks and LOC estimates.
2. Remove the ext_repr import path and commit to direct `Container::new_strip` + `BPInstance::new` construction.
3. Add a trait abstraction between engine types, or explicitly accept the duplication cost in LOC estimates.
4. Promote cross-sheet GLS to P0.
5. Add stock exhaustion guards and oversized-piece user feedback.
6. Revise LOC estimates upward to ~1300-1500.
