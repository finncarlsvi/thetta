# CamBam Holding Tabs Documentation
> **URL**: http://www.cambam.info/doc/plus/cam/HoldingTabs.htm
> **Type**: Product
> **Agent**: Codex
> **Relevance**: High
> **Credibility**: Medium

## Summary
CamBam’s holding-tab documentation is one of the clearest algorithmic descriptions among commercial CAM sources in this set. It gives an explicit tab count rule tied to contour perimeter and target spacing, then bounds that result with minimum/maximum limits.

That transparency makes it especially valuable for implementation benchmarking. Unlike black-box “auto tab” features, CamBam’s rule can be reproduced exactly and used as a deterministic baseline before adding more sophisticated geometry-aware heuristics.

The inclusion of a size-threshold skip condition is also important for production realism. It formalizes a common operator instinct: very small contours may be better left without tabs due to cleanup burden or limited benefit.

## Key Findings
- Automatic tabs are computed as `perimeter / tab_distance`, rounded and clamped between configured minimum and maximum tab counts.
- A perimeter-based size threshold can disable tab insertion for small shapes.

## Technical Details
The documented model is scalar and fast: compute perimeter, derive candidate count, clamp, then place tabs along contour at near-equal spacing. Additional parameters include tab style, width, and depth. This is straightforward to implement and test as a first-pass automatic tab module.

## Strengths
- Explicit, reproducible auto-tab formula with practical guardrails (min/max, threshold).

## Weaknesses & Limitations
- Perimeter-only logic ignores local geometry quality (corners, visibility, neighboring-part interference).

## Connections
- [RES-001-micro-joints-genetic-algorithm.md](/Users/finn.carlsvi/thetta/research/drafts/RESEARCH-005-resources/codex/RES-001-micro-joints-genetic-algorithm.md) — RES-001 provides a more advanced optimization approach that can be compared against CamBam’s deterministic perimeter heuristic.
