# Sprint 001 Merge Notes

## Draft Comparison

### Claude Draft Strengths
- Comprehensive 4-phase implementation plan with all tasks detailed
- Includes STEP import as Phase 3 (ambitious but complete)
- Good risk matrix with specific mitigations
- Detailed threading model

### Gemini Draft Strengths
- Key insight: **defer STEP to Sprint 2** to de-risk
- Better phasing: geometry → jagua-rs + LBF → GLS → GUI (parallelizable)
- More realistic LOC estimates (~1960 total)
- Clearer jagua-rs API integration plan (BPP module, specific types)
- Open questions about BPP vs SPP mode are important

### Key Differences
| Aspect | Claude | Gemini |
|--------|--------|--------|
| STEP import | Included as Phase 3 | Deferred to Sprint 2 |
| Baseline heuristic | Mentioned briefly | LBF as explicit phase |
| jagua-rs detail | High-level | Specific types and API calls |
| LOC estimates | None | ~1960 |

## Interview Refinements

1. **STEP deferred to Sprint 2** — JSON polygon input for Sprint 1
2. **Both LBF and GLS in Sprint 1** — LBF as baseline, GLS as the real optimizer
3. **Polished GUI** — Full panels, piece interaction, stats, sheet tabs
4. **Cut priority if behind: multi-sheet** — Single sheet first, multi-sheet in Sprint 2

## Merge Decisions

- Adopt Gemini's scoping: no STEP in Sprint 1, JSON polygons instead
- Adopt Claude's comprehensive task breakdown but reorder per Gemini's phasing
- Include both LBF baseline AND GLS optimizer per interview
- Include full GUI with piece interaction per interview
- Multi-sheet is cut target if behind schedule
- Keep thetta-step crate as stub (Sprint 2 target)
