# Deepnest — Open Source Nesting
> **URL**: https://github.com/Jack000/Deepnest
> **Type**: Code
> **Agent**: Codex
> **Relevance**: Medium
> **Credibility**: High

## Summary
Deepnest is an open-source nesting optimizer designed for laser/plasma/CNC workflows, focused on maximizing material utilization. It is not a tab-placement tool, but it is highly relevant to the shared-edge context where tab alignment becomes challenging.

Its key feature for this research is automatic common-line merging, where adjacent parts share cut lines to reduce duplicated cuts and heat distortion. This is structurally adjacent to shared-tab planning because both require inter-part reasoning rather than independent contour treatment.

For implementation in a nesting-oriented system, Deepnest illustrates that global geometry operations across multiple parts are practical and beneficial. A tab module could build on similar adjacency graphs to coordinate hold points across neighbors.

## Key Findings
- Deepnest performs common-line merging to avoid cutting identical adjacent boundaries twice.
- The tool focuses on material/time efficiency and thermal improvement, but does not document explicit tab algorithms.

## Technical Details
Architecture includes JavaScript-heavy application logic with a C-based performance component and supports SVG/DXF workflows. The common-line machinery likely computes proximity/overlap relationships that could be repurposed for shared-tab candidate extraction and conflict checking.

## Strengths
- Strong open-source reference for inter-part geometric optimization in nesting workflows.

## Weaknesses & Limitations
- No native documented tab placement, so adaptation to hold-feature planning requires new logic.

## Connections
- [RES-002-common-edge-nested-cutting.md](/Users/finn.carlsvi/thetta/research/drafts/RESEARCH-005-resources/codex/RES-002-common-edge-nested-cutting.md) — both center on common-edge/shared-line optimization for adjacent parts.
