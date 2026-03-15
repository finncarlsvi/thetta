# How to position tabs for cutting out adjacent pieces — Autodesk Forum
> **URL**: https://forums.autodesk.com/t5/fusion-manufacture-forum/how-to-position-tabs-for-cutting-out-adjacent-pieces/td-p/12566903
> **Type**: Community
> **Agent**: Codex
> **Relevance**: High
> **Credibility**: Medium

## Summary
This forum thread is directly aligned with the core research problem: tabs placed on one part can be removed unintentionally when cutting neighboring parts in tight nests. It captures a failure mode that is often absent from official documentation.

The discussion suggests that default automatic tab placement may not coordinate between adjacent contours. Users report needing manual alignment or increased spacing to avoid destroying tabs, which confirms the absence (or weakness) of shared-tab aware logic in typical workflows.

For implementation planning, this resource is critical despite lower academic rigor. It provides a concrete user pain point and candidate acceptance criteria: tab locations should be co-planned across neighboring pieces and remain intact after all relevant toolpaths execute.

## Key Findings
- Adjacent-part machining can invalidate tabs if locations are not aligned across neighboring contours.
- Practitioner workarounds are manual tab drawing/edge selection or larger part spacing, both of which reduce automation efficiency.

## Technical Details
The thread indicates a need for cross-part conflict detection: given two close contours, candidate tab arcs should be checked against future cuts from both operations. A shared-edge/tab graph model could enforce synchronized hold regions and delayed release ordering.

## Strengths
- High relevance to real user-reported behavior around shared-tab failure in dense layouts.

## Weaknesses & Limitations
- Forum evidence is anecdotal and access-limited in cache, so details are incomplete and not systematically validated.

## Connections
- [RES-002-common-edge-nested-cutting.md](/Users/finn.carlsvi/thetta/research/drafts/RESEARCH-005-resources/codex/RES-002-common-edge-nested-cutting.md) — both center on adjacent/shared-edge cutting where independent per-part planning breaks down.
