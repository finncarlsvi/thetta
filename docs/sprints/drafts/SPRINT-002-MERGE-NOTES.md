# Sprint 002 Merge Notes

## Claude Draft Strengths (adopted)
- P0/P1/P2 DoD structure with cut list
- LOC estimates per file
- Concrete CLI command examples (`cargo run --release -p thetta-nesting --bin benchmark`)
- ASCII architecture diagrams
- `clap` dependency identified

## Codex Draft Strengths (adopted)
- Detailed orientation canonicalization rules (degree→radian, normalization, dedup, RotationRange mapping)
- Security section with untrusted JSON, path handling, panic containment
- Collision scoring policy detail (monotonic, exterior penalty weighting)
- 5-phase implementation sequencing

## Valid Critiques Accepted
1. **centering_transformation** (both critiques): jagua-rs centers items at origin. Must use `ext_to_int_transformation()` in conversion layer. Added as explicit task.
2. **fit_strip() panic on empty layout** (both critiques): Guard with `placed_items.is_empty()` check before calling.
3. **Item ID consecutive-from-zero** (both critiques): `SPInstance::new()` asserts this. Conversion must renumber items.
4. **HazardCollector extends HazardFilter** (both critiques): Non-trivial impl, made explicit task.
5. **Demand accounting** (Claude critique): `register_included_item` decrements demand. force_place must remove-then-place to avoid underflow.
6. **ordered-float not needed as direct dep** (Codex critique): It's jagua's internal dependency.
7. **SVG parsing for known results is brittle** (Codex critique): Use hardcoded lookup table instead.
8. **spp feature flag** (both critiques): Must enable when depending on jagua-rs.

## Valid Critiques Rejected
1. "nightly/SIMD requirement" stale — Both critiques note this. Correct, removed from sprint doc.

## Interview Refinements Applied
1. **Fork jagua-rs** into workspace instead of git dependency. Full control, offline builds.
2. **CLI + GUI benchmark panel** — both CLI runner and GUI strip length display.
3. **Quality target: all pieces placed** — no strip length target for P0. Strip length comparison is P1.
4. **Respect discrete orientations** from benchmark format, continuous for native format.
