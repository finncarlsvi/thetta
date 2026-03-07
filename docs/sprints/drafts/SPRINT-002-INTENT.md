# Sprint 002 Intent: Benchmark-Grade Nesting

## Seed Prompt

Import sparrow benchmark JSON format, integrate jagua-rs collision detection, run ESICUP benchmarks, add benchmark comparison mode, and improve nesting quality to be competitive with published results.

## Orientation Summary

- Sprint 001 delivered a working nesting app with custom O(n²) collision engine, LBF + GLS optimizer, egui GUI (~1600 LOC)
- The custom collision engine in `engine.rs` uses brute-force polygon overlap checks — needs replacing with jagua-rs for performance and correctness
- Sparrow's JSON format (`{name, items, strip_height}`) differs from ours (`{id, quantity, polygon}`) — need a converter or dual-format support
- 34 benchmark instances available in `research/repos/sparrow/data/input/` with published best results
- GLS `force_place_item` was just added to fix a critical bug where GLS couldn't place overlapping pieces

## Relevant Codebase Areas

| Area | Files | Relevance |
|------|-------|-----------|
| Collision engine | `thetta-nesting/src/engine.rs` | Replace with jagua-rs |
| Conversion layer | `thetta-nesting/src/conversion.rs` | Currently stub types, needs real jagua-rs conversion |
| GLS optimizer | `thetta-nesting/src/gls.rs` | Must work with new engine API |
| LBF heuristic | `thetta-nesting/src/lbf.rs` | Must work with new engine API |
| Runner | `thetta-nesting/src/runner.rs` | Orchestrates LBF → GLS → compression |
| Geometry types | `thetta-geometry/src/types.rs` | May need additions for benchmark format |
| App state | `thetta-app/src/state.rs` | Loading benchmark files, showing results |
| Benchmark data | `research/repos/sparrow/data/input/*.json` | 34 ESICUP instances |
| Benchmark results | `research/repos/sparrow/data/records/` | Best known strip lengths |

## Sparrow JSON Format (reference)

```json
{
  "name": "jakobs1",
  "strip_height": 40.004,
  "items": [
    {
      "id": 0,
      "demand": 1,
      "allowed_orientations": [0.0, 90.0, 180.0, 270.0],
      "shape": {
        "type": "simple_polygon",
        "data": [[0.0, 0.0], [2.0, 0.0], [0.0, 2.0], [0.0, 0.0]]
      }
    }
  ]
}
```

## Our Current JSON Format

```json
[
  {
    "id": 1,
    "quantity": 1,
    "polygon": {
      "outer": [{"x": 0.0, "y": 0.0}, {"x": 400.0, "y": 0.0}, ...],
      "holes": []
    }
  }
]
```

## Constraints

- Must preserve existing workspace crate structure (thetta-geometry, thetta-nesting, thetta-app, thetta-step)
- Must preserve background threading model (mpsc channel, AtomicBool cancel)
- egui GUI must continue working — benchmark mode is additive
- jagua-rs is a git dependency (requires network at build time)
- jagua-rs requires nightly Rust for SIMD features (optional but recommended)
- Our current `NestingEngine` API (`place_item`, `force_place_item`, `collision_cost`, `remove_item`, `current_result`) should be preserved or adapted minimally

## Success Criteria

1. Can load any of the 34 sparrow benchmark JSON files
2. jagua-rs collision detection replaces custom engine
3. Can run a benchmark and report strip length + utilization
4. Results are within reasonable range of published SOTA (not necessarily matching sparrow's results, but not orders of magnitude worse)
5. Benchmark mode: CLI or UI panel that runs a benchmark instance and reports metrics

## Verification Strategy

- Run all 34 benchmark instances through our engine
- Compare strip lengths against sparrow's published records
- Automated integration tests for format conversion
- Build + test on macOS ARM64

## Uncertainty Assessment

| Factor | Level | Reason |
|--------|-------|--------|
| Correctness | Medium | jagua-rs API is well-documented but we need to map our abstractions correctly |
| Scope | Medium | Clear targets (34 benchmarks) but jagua-rs integration depth is uncertain |
| Architecture | High | Replacing the collision engine is a fundamental change; jagua-rs may impose different patterns |

## Open Questions

1. Should we use jagua-rs as a library dependency or fork it into our workspace?
2. Strip packing mode (SPP) vs bin packing mode (BPP) — which jagua-rs problem type to use?
3. Should benchmark mode be CLI-only, GUI-only, or both?
4. How much of sparrow's two-phase approach (explore + compress) should we replicate vs keeping our own GLS?
5. Should we support `allowed_orientations` from the benchmark format or keep continuous rotation?
6. What's an acceptable quality target? Within 10% of sparrow? Within 20%?
