# Sprint 002 Draft Critique

This document reviews both the Codex and Claude drafts for Sprint 002 against the intent document and verified jagua-rs source code.

---

## Codex Draft Critique

### Strengths

1. **Phased implementation is well-structured.** The five phases (engine skeleton, benchmark import, optimizer migration, benchmark runner, UI stability) form a logical dependency chain with clear progression. Each phase has concrete, enumerated tasks.

2. **Orientation handling is thorough.** Phase 2, Task 2 (lines 137-144) provides explicit canonicalization rules: degree-to-radian conversion, normalization to `[0, 2pi)`, deduplication, and correct mapping to `RotationRange` variants. This matches the actual jagua-rs `Importer::import_item()` logic at `io/import.rs:80-88`.

3. **Security section is substantive.** Lines 287-301 cover untrusted JSON, path traversal, resource controls, and panic containment -- more depth than most sprint plans provide. The `dxf` field warning (line 294) shows awareness of the actual benchmark format.

4. **Collision scoring design is well-considered.** Lines 126-129 describe a collector-based score with monotonicity for GLS compatibility, plus differential penalties for exterior vs. item hazards. This aligns with how `HazardCollector` works (it tracks `HazardEntity` types including `HazardEntity::Exterior`).

5. **Open questions are actionable.** Lines 319-337 raise six well-scoped questions, each with clear decision boundaries.

### Weaknesses

1. **`Importer` is underspecified.** Line 71 mentions `Importer::new(..., min_item_separation = Some(sheet.separation), ...)` but the actual `Importer::new()` signature (at `io/import.rs:31-45`) takes `(CDEConfig, Option<f32>, Option<f32>, Option<(f32, f32)>)` -- four positional parameters for `cde_config`, `simplify_tolerance`, `min_item_separation`, and `narrow_concavity_cutoff`. The draft does not mention `simplify_tolerance` or `narrow_concavity_cutoff`, and the named-parameter syntax shown does not match Rust calling conventions. These configuration choices (especially `simplify_tolerance`) can materially affect polygon quality.

2. **Missing `pre_transform` / centering transformation awareness.** jagua-rs applies a centering transformation to each item (see `io/import.rs:71` -- `pre_transform: centering_transformation(&shape)`), which translates the centroid to the origin. This means all `DTransformation` values used during placement are relative to the centered shape, not the original coordinates. The draft never mentions this critical coordinate-space concern, which will cause placement bugs if ignored.

3. **`SPProblem::place_item` return value not discussed.** The draft (line 55) lists `SPProblem::place_item(SPPlacement)` but does not mention that it returns a `PItemKey`, which is essential for later removal. While the engine section (line 119) mentions `piece_idx -> Option<PItemKey>` mapping, the connection between the place call's return value and this mapping is implicit.

4. **No discussion of `SPProblem::save()`/`restore()`.** These are essential for GLS (saving best solutions, restoring on regression). The draft's GLS section (lines 160-163) does not mention snapshot/restore semantics.

5. **`ExtSPInstance` vs `ExtItem` confusion.** Line 69 mentions `ExtItem` + `ExtSPInstance` input models, but jagua-rs has two different `ExtItem` types: `crate::io::ext_repr::ExtItem` (base) and `crate::probs::spp::io::ext_repr::ExtItem` (wraps base + demand). The draft does not disambiguate, which will cause import confusion.

6. **LOC estimates absent.** Unlike the Claude draft, there are no effort estimates, making it harder to assess feasibility within the sprint.

### Gaps in Risk Analysis

1. **Missing risk: `rayon` parallelism dependency.** The SPP import module (`probs/spp/io/import.rs:9`) uses `rayon::prelude::*` for parallel item import. If Thetta does not already depend on rayon, this introduces a new transitive dependency. Not mentioned anywhere.

2. **Missing risk: `centering_transformation` coordinate mismatch.** As noted above, placement coordinates must account for jagua-rs's centering pre-transform. Getting this wrong produces silently incorrect placements. This is a high-likelihood integration bug.

3. **Missing risk: `fit_strip()` panics on empty layout.** The implementation at `problem.rs:43-49` calls `.unwrap()` on the max of placed item x-coordinates. If no items are placed, this panics. The draft does not note this invariant.

4. **Missing risk: item demand tracking.** `SPProblem::place_item()` calls `register_included_item()` which decrements `item_demand_qtys[item_id]`. If the same `item_id` is placed more times than its demand, this will underflow (panic in debug, wrap in release). The draft's `force_place_item` semantics (line 89) could trigger this if not carefully managed.

### Missing Edge Cases

1. **Zero-demand items.** The SPP import (`probs/spp/io/import.rs:25`) calls `items.retain(|(_, demand)| *demand > 0)`, filtering out zero-demand items. If any benchmark instance has `demand: 0` items, ID renumbering could break assumptions. Not discussed.

2. **Single-vertex or degenerate polygons.** jagua-rs's `import_simple_polygon` eliminates degenerate vertices but will fail on polygons with fewer than 3 non-degenerate vertices. No validation mentioned.

3. **`strip_height` as f32.** `ExtSPInstance::strip_height` is `f32`, but the draft mentions f64 values from JSON. The Sparrow JSON format uses f64 (`"strip_height": 40.004`), but jagua-rs's `ExtSPInstance` deserializes it as `f32`. Precision is generally fine for these values, but the draft's conversion section (line 66-68) does not acknowledge that jagua-rs's own serde handles this.

### Definition of Done Completeness

The DoD (lines 239-262) covers the main areas but lacks:
- No measurable performance criterion (e.g., "jakobs1 LBF completes in under 5 seconds")
- No specific test count or coverage target
- "Automated tests cover conversion, orientations, and engine placement invariants" (line 261) is vague -- does not specify which invariants

### Technical Accuracy of jagua-rs API Claims

| Claim (line) | Verdict | Notes |
|---|---|---|
| `Item::new(id, OriginalShape, RotationRange, min_quality, surrogate_config)` (line 52) | **Correct** | Matches `entities/item.rs:27-33` |
| `Strip::new(fixed_height, cde_config, shape_modify_config, width)` (line 53) | **Correct** | Matches `spp/entities/strip.rs:18-32`, returns `Result<Self>` |
| `SPProblem::new(instance)` (line 54) | **Correct** | Matches `spp/entities/problem.rs:19` |
| `SPProblem::place_item(SPPlacement { item_id, d_transf })` (line 55) | **Correct** | Returns `PItemKey`, matches `problem.rs:60-65` |
| `SPProblem::remove_item(pkey)` (line 56) | **Partially correct** | Returns `SPPlacement`, not `void`. Matches `problem.rs:68-76` |
| `Layout::is_feasible()` (line 57) | **Correct** | Matches `layout.rs:128-136` |
| `Layout::cde().detect_poly_collision(shape, filter)` (line 58) | **Correct** | `cde()` returns `&CDEngine`, method at `cd_engine.rs:160`. Returns `bool`. |
| `Layout::cde().collect_poly_collisions(shape, collector)` (line 59) | **Correct** | Method at `cd_engine.rs:261`. Collector must impl `HazardCollector`. |
| `SPProblem::fit_strip()` (line 60) | **Correct** | Matches `problem.rs:39` |
| `SPProblem::change_strip_width(...)` (line 60) | **Correct** | Takes `f32`, matches `problem.rs:33` |

---

## Claude Draft Critique

### Strengths

1. **Excellent LOC estimates and file-level planning.** The Files Summary (lines 356-385) provides per-file LOC estimates for both new and modified files, totaling ~1140 LOC. This makes sprint capacity planning concrete.

2. **Prioritized Definition of Done.** Lines 388-418 use P0/P1/P2 tiers with a "Cut if Behind Schedule" section (lines 416-418). This is significantly more actionable than the Codex draft's flat checklist, giving the team clear triage guidance.

3. **Concrete CLI interface design.** Lines 276-297 specify exact `cargo run` commands, CLI flags (`--input`, `--time`, `--all`, `--csv`), and output table format. This eliminates ambiguity about the benchmark runner's UX.

4. **Threading model documentation.** The ASCII diagram at lines 114-122 makes the threading model explicit and confirms it is unchanged. Good for onboarding and review.

5. **Risk table format with Impact/Likelihood/Mitigation columns.** Lines 422-430 are structured and scannable, with specific mitigations rather than generic statements.

6. **Dependency on `clap` identified.** Line 451 correctly notes the need for `clap` for CLI argument parsing, which the Codex draft omits.

7. **`ordered-float` dependency identified.** Line 133 and line 450 correctly note this is required by jagua-rs's `DTransformation` (which uses `NotNan<f32>`). The Codex draft does not mention this.

### Weaknesses

1. **`collision_cost()` implementation description is imprecise.** Line 237 says "Temporarily transform the item's `SPolygon`" but does not explain how to obtain the transformed polygon. In jagua-rs, `Item` stores `shape_cd` (the collision detection shape), which must be transformed via `DTransformation` -> `Transformation` -> `Transformable::transform_clone()`. The draft does not reference this transformation pipeline.

2. **`force_place_item` semantics are misleadingly simple.** Line 239 says "Call `problem.place_item()` unconditionally. The CDE will register the hazard even if it overlaps." This is correct that jagua-rs does not prevent overlapping placements, but it fails to note that every `place_item` call decrements the item's demand counter (`register_included_item`). If the same item_id is placed beyond its demand, this will underflow. For GLS where pieces are force-placed and moved, this demand tracking is a critical integration concern.

3. **Missing `pre_transform` / `ext_to_int_transformation` awareness.** Same issue as the Codex draft. jagua-rs centers items at the origin via `centering_transformation`. The `DTransformation` used in `SPPlacement` must be in the internal coordinate space (post-centering). jagua-rs provides `ext_to_int_transformation()` (`io/import.rs:223-234`) for this conversion. Neither draft mentions this function or the coordinate space issue. This is likely the single highest-risk integration bug.

4. **Task 2.2 (benchmark result comparison) is fragile.** Lines 301-304 propose parsing SVG solution files to extract strip widths. This is brittle -- SVG parsing for metric extraction is error-prone and not what SVGs are designed for. The sparrow repo may have structured result data (JSON/CSV) that would be more reliable. If not, hardcoding known-best values in a lookup table is simpler and more maintainable.

5. **`detect_poly_collision` return type confusion.** Line 430 risk entry says "does not return a numeric cost, only bool" with likelihood "Low." This is not really a risk -- it is a known fact: `detect_poly_collision` returns `bool` (verified at `cd_engine.rs:160`). Framing a known API characteristic as a "risk" with "Low" likelihood is misleading. It should be stated as a design constraint, not a risk.

6. **UC4 (Run Benchmark via GUI) scope creep.** Lines 31-35 describe a "Load Benchmark" button as a use case, but the intent document (line 68) says "benchmark mode is additive" and asks whether benchmark mode should be "CLI-only, GUI-only, or both?" (line 101). The Claude draft assumes both without flagging this as a decision point. In the DoD, GUI benchmark features are correctly placed at P1, but presenting it as a core use case creates mixed signals.

7. **`HazardCollector` trait complexity understated.** Line 472 recommends "use a counting `HazardCollector` implementation" but `HazardCollector` extends `HazardFilter` (verified at `collector.rs:7`: `pub trait HazardCollector: HazardFilter`). Implementing it requires implementing both `HazardFilter::is_irrelevant()` and `HazardCollector::{contains_key, insert, remove_by_key, iter}`. This is non-trivial and should be called out as implementation work.

### Gaps in Risk Analysis

1. **Missing risk: `fit_strip()` panic on empty layout.** Same as for Codex. `problem.rs:49` calls `.unwrap()` on an iterator that is empty when no items are placed.

2. **Missing risk: `rayon` dependency.** Same as for Codex.

3. **Missing risk: coordinate space mismatch (centering transform).** The most likely source of integration bugs. Neither draft addresses this.

4. **Missing risk: `SPInstance` requires consecutive item IDs starting from 0.** `SPInstance::new()` asserts this (`instance.rs:17-19`). If Thetta assigns piece IDs non-consecutively (e.g., from benchmark item IDs with gaps or from demand expansion), this assertion will panic. The draft's conversion layer must renumber items, but this is not discussed.

5. **Missing risk: winding order.** Line 191 says "jagua-rs expects CCW, same as ours" but this claim is not verified against Thetta's geometry types. If Thetta uses CW winding, all polygons will be inverted. The actual winding order expectation in jagua-rs is determined by `SPolygon::new()` which calls `ensure_ccw()` -- so jagua-rs does normalize, but the draft should note this rather than making an unverified assumption.

### Missing Edge Cases

1. **Benchmark items with `demand: 0`.** The SPP importer filters these out and renumbers IDs. If the conversion layer does not match this behavior, ID mismatches will occur.

2. **Items with holes.** `Importer::import_item()` warns and ignores holes (`io/import.rs:63-64`). If any benchmark instance has items with holes, this is silently lossy. Not discussed.

3. **`strip_height` precision.** `ExtSPInstance::strip_height` is `f32`. Sparrow JSON may have `strip_height: 40.004` which loses precision in f32. The draft's conversion (line 190) proposes f64-to-f32 coordinate conversion but does not note that jagua-rs's own deserialization already handles this, so double-conversion should be avoided.

4. **Empty `allowed_orientations` array.** jagua-rs treats `Some([])` as `RotationRange::None` (no rotation), not continuous. The Codex draft (line 141) gets this right; the Claude draft (line 193) says `RotationRange::Discrete` or `RotationRange::Continuous` without specifying the empty-array case.

### Definition of Done Completeness

The P0/P1/P2 structure is good. Gaps:
- P0 item "LBF places all pieces from jakobs1.json with zero collision" (line 394) -- this is a strong correctness check, but it should also verify that the *number* of placed pieces equals the total demand (25 pieces for jakobs1).
- No performance/latency criterion at any tier.
- P2 item "Results within 20% of sparrow's published best on 5+ instances" (line 412) contradicts the open question at line 476 which suggests "within 50% of sparrow's best" as P1. These should be reconciled.

### Technical Accuracy of jagua-rs API Claims

| Claim (line) | Verdict | Notes |
|---|---|---|
| "quadtree-backed collision detection" (line 5) | **Correct** | `CDEngine` uses `QTNode` quadtree. |
| `SPInstance`, `SPProblem` (line 104-108) | **Correct** | Module paths and field structure verified. |
| `CDEngine` uses quadtree with surrogates for O(log n) (line 110) | **Approximately correct** | CDEngine does use quadtree + surrogates. Complexity depends on tree depth and item count; "O(log n)" is a simplification but directionally accurate. |
| `Layout` manages hazard registration automatically (line 110) | **Correct** | `Layout::place_item()` calls `cde.register_hazard()`, `remove_item()` calls `deregister_hazard_by_entity()`. |
| `detect_poly_collision` takes `HazardFilter` (line 237, 424) | **Correct** | Signature at `cd_engine.rs:160`: `fn detect_poly_collision(&self, shape: &SPolygon, filter: &impl HazardFilter) -> bool`. |
| `collect_poly_collisions` with counting collector (line 237, 472) | **Correct concept, implementation understated** | `HazardCollector` extends `HazardFilter`. Need custom impl. |
| `SPProblem::fit_strip()` (line 241, 249) | **Correct** | At `problem.rs:39`. |
| `place_item` does not check collisions (line 426) | **Correct** | `Layout::place_item()` at `layout.rs:73-84` just registers the hazard. |
| f32 throughout (line 191, 425) | **Correct** | All jagua-rs geometry uses `f32`. `ExtSPInstance::strip_height` is `f32`. |
| `ShapeModifyConfig.offset = Some(separation / 2.0)` (line 194) | **Correct** | Matches `Importer::new()` at `io/import.rs:39`. |
| `DTransformation` uses `NotNan<f32>` (line 133) | **Needs verification** | `ordered-float` is listed as a dependency, but the `DTransformation` struct definition was not checked in this review. Likely correct given the dependency exists. |

---

## Cross-Draft Comparison

### What Codex Does Better
- More detailed orientation canonicalization rules (angles, deduplication, normalization)
- Security section with concrete defensive measures
- `collision_cost` scoring policy is more detailed (monotonicity, exterior penalty weighting)

### What Claude Does Better
- LOC estimates enable capacity planning
- P0/P1/P2 DoD tiers with explicit cut list
- Concrete CLI command examples
- Identifies `ordered-float` and `clap` dependencies
- Risk table with structured columns
- ASCII architecture diagrams for quick comprehension

### Shared Blind Spots (Both Drafts)
1. **`centering_transformation` / `ext_to_int_transformation`:** Neither draft addresses jagua-rs's internal coordinate centering. This is the most dangerous shared gap -- placements will be wrong if the coordinate transform is not applied.
2. **`fit_strip()` panic on empty layout:** Neither notes the `.unwrap()` on an empty iterator.
3. **`rayon` transitive dependency:** Not mentioned by either.
4. **Item ID consecutive-from-zero requirement:** `SPInstance::new()` asserts consecutive IDs. Neither draft discusses renumbering strategy.
5. **`HazardCollector` extends `HazardFilter`:** Both mention using a collector but neither notes it requires implementing two traits.
6. **Demand accounting in `SPProblem`:** Neither fully addresses how `register_included_item` / `deregister_included_item` interacts with GLS's force-place/remove patterns.

---

## Recommendations

1. **Address the centering transform immediately.** Add a task in Phase 1 to implement `ext_to_int_transformation` usage in the conversion layer. This is the highest-risk integration concern.

2. **Guard `fit_strip()` calls.** Always check `layout.placed_items.is_empty()` before calling `fit_strip()`.

3. **Adopt the Claude draft's P0/P1/P2 structure** with the Codex draft's detail level for orientation handling and collision scoring.

4. **Add a performance criterion to P0.** At minimum: "LBF on jakobs1 completes in under 10 seconds in release mode."

5. **Resolve the quality target discrepancy.** The Claude draft says "within 50% of sparrow" as P1 but "within 20%" as P2. Pick one and be explicit about measurement methodology.

6. **Add `rayon` to the dependency list** or verify it is already in the workspace dependency tree.

7. **Design the `HazardCollector` implementation** as a named task with its own acceptance criteria, since it requires implementing multiple trait methods and is central to GLS cost computation.
