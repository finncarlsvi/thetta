# Sprint 002 Draft Critique (Codex + Claude)

## Draft A: `SPRINT-002-CODEX-DRAFT.md`

### 1. Strengths
- **`Overview` and `Architecture > 1. Engine replacement with jagua-rs core entities`** clearly align with intent: migrate collision core to jagua-rs while preserving LBF/GLS and GUI behavior.
- **`Implementation (Phased with Tasks)`** has good sequencing (engine/conversion first, then benchmark import, then optimizer/runner/UI), which reduces integration risk.
- **`Risks` and `Security`** include practical concerns (f64->f32 tolerance, resource controls, untrusted JSON input) and are stronger than typical sprint drafts.
- **`Definition of Done`** is concrete about benchmark parsing, runner outputs, and regression expectations.

### 2. Weaknesses
- **`Architecture > 2. Conversion boundary`** references `jagua_rs::io::ext_repr::ExtShape::SimplePolygon` as the conversion target; the real SPP import path is `probs::spp::io::ext_repr::ExtSPInstance` + importer, not base `ExtShape` alone.
- **`Architecture > 4. Optimizer compatibility`** is ambiguous on how `force_place_item` should preserve GLS behavior if placement fails after remove/reinsert; rollback semantics are not specified.
- **`Phase 4: Benchmark runner and reporting`** does not define how best-known records are loaded/normalized, so gap metrics may be inconsistent.
- **`Dependencies`** does not call out required jagua feature flags (`spp`) even though SPP APIs are used.

### 3. Gaps in Risk Analysis
- Missing explicit risk that **`jagua-rs` problem modules are feature-gated** (`probs::spp` behind `feature = "spp"`), which can cause compile failures if dependency features are misconfigured.
- Missing risk that **`SPProblem::fit_strip()` panics on empty layout** (`max_by(...).unwrap()`), relevant when no pieces are placed.
- Missing risk around **ID invariants**: jagua SPP import expects item IDs to be consecutive from 0 in some import paths.
- Missing risk around **unit mismatch policy** (mm vs benchmark units) and its effect on reporting comparability.

### 4. Missing Edge Cases
- Empty or zero-demand benchmark instances (or all-demand-zero after filtering).
- Invalid `strip_height <= 0` (jagua `Strip::new` enforces positive values).
- Non-consecutive/duplicate item IDs in benchmark JSON.
- `allowed_orientations` containing duplicates, negatives, `NaN`, or values equivalent mod 360.
- `fit_strip` called before any successful placement.

### 5. Definition of Done Completeness
- Strong for core migration and benchmark ingestion.
- Incomplete on **comparability acceptance thresholds** (intent asks “reasonable range,” but DoD has no explicit numeric guardrail).
- Incomplete on **failure-mode tests** (malformed benchmark JSON, invalid dimensions, empty placement before strip fitting).
- Incomplete on **feature/config validation** (ensuring `spp` feature is enabled in dependency wiring).

### 6. Technical Accuracy of jagua-rs API Claims
- **Accurate:** `SPProblem`, `Layout`, `CDEngine`, `SPProblem::new`, `place_item`, `remove_item`, `fit_strip`, `change_strip_width`, `Layout::is_feasible`, `detect_poly_collision`, `collect_poly_collisions` all exist with matching high-level semantics.
- **Accurate with caveat:** `detect_poly_collision(shape, filter)` can ignore self-collision by passing a `HazKey` filter, but the draft does not mention obtaining that key via `haz_key_from_pi_key`.
- **Partially accurate:** `Strip::new(...)` and `Item::new(...)` signatures are correct, but both return `Result` and enforce validity constraints.
- **Potentially inaccurate by omission:** SPP APIs are feature-gated (`probs::spp`), not guaranteed available without dependency feature config.
- **Likely stale assumption from intent carried forward:** nightly/SIMD requirement is not reflected in current jagua-rs crate features (no SIMD/nightly feature in current `Cargo.toml`).

## Draft B: `SPRINT-002-CLAUDE-DRAFT.md`

### 1. Strengths
- **`Use Cases`** are concrete and user-visible (CLI, GUI, native format preservation).
- **`Architecture > Engine Architecture Change`** explains state transition from brute-force to jagua-backed engine in practical terms.
- **`Definition of Done`** is well-prioritized (`P0/P1/P2`) and execution-friendly.
- **`Phase 4: Testing and Validation`** includes practical integration/smoke tests across benchmark corpus.

### 2. Weaknesses
- **`Phase 1 > Task 1.1` and `Dependencies`** list `ordered-float` as required by app code for `DTransformation`; this is not required unless directly using that type internals. It is jagua’s internal dependency.
- **`Phase 1 > Task 1.4`** suggests `force_place_item` calling `problem.place_item()` unconditionally, but does not define behavior when an item already has a placement key (stale duplicate hazard risk unless remove-first is guaranteed).
- **`Phase 2 > Task 2.2`** proposes parsing Sparrow SVG records for best-known widths; this is brittle for a sprint unless schema quirks are fully specified.
- **`Risks`** states benchmark coordinate ranges are small and precision risk is low; this is asserted without instance-level evidence.

### 3. Gaps in Risk Analysis
- Missing explicit risk that **`SPProblem::fit_strip()` will panic on empty layouts**.
- Missing risk for **dependency feature mismatch** (`spp` feature not enabled).
- Missing risk that **jagua import paths expect strict item-ID invariants** (consecutive IDs) depending on conversion route.
- Missing risk that **`change_strip_width` rebuilds container/CDE state**, which can affect performance if called too frequently in compression loops.

### 4. Missing Edge Cases
- Calling benchmark mode with no placed items and then fitting strip.
- Input polygons with non-consecutive duplicate vertices or invalid simple polygons.
- `allowed_orientations: []` and `[0]` semantics vs continuous rotation fallback.
- Extremely high `demand` causing expansion memory/time blowups.
- Invalid `strip_height` and other non-positive dimensions.

### 5. Definition of Done Completeness
- Very good operational checklist, especially `P0` gates.
- Needs explicit **negative-path criteria** (bad JSON, invalid geometry, impossible placement cases).
- Needs an explicit **quality target definition** beyond “all pieces placed” (intent asks for competitiveness framing).
- Needs a criterion ensuring **native format parity** is tested, not just GUI launch smoke.

### 6. Technical Accuracy of jagua-rs API Claims
- **Accurate:** core claim set around `SPProblem`, `Layout`, `CDEngine`, `place_item`, `remove_item`, `fit_strip`, `change_strip_width`, `detect_poly_collision`, `collect_poly_collisions`.
- **Accurate:** statement that `Layout::place_item` registers hazard and does not itself enforce feasibility (feasibility checked separately).
- **Accurate with caveat:** collision-cost approach via collector is valid, but should explicitly use a collector implementation (e.g., `BasicHazardCollector`) and self-hazard filtering.
- **Inaccurate:** `ordered-float` is not required as an explicit dependency for Thetta merely to use jagua APIs.
- **Inaccurate/stale:** “nightly required for jagua SIMD features” is not supported by current jagua-rs crate feature definitions.

## Comparative Notes
- Codex draft is stronger on architecture migration sequencing and security framing.
- Claude draft is stronger on delivery management (`P0/P1/P2`) and executable validation steps.
- Both drafts are broadly correct on jagua API surface, but both should be tightened around feature gating (`spp`), empty-layout `fit_strip` behavior, and stale nightly/SIMD assumptions.
