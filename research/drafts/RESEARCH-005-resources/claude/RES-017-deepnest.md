# Deepnest — Open Source Nesting
> **URL**: https://github.com/Jack000/Deepnest
> **Type**: Code
> **Agent**: Claude
> **Relevance**: Medium
> **Credibility**: Medium

## Summary
Deepnest is an open-source nesting application designed for laser cutters, plasma cutters, and CNC machines. Its primary function is packing parts into the most compact arrangement possible to save material and reduce cutting time. While Deepnest does not include tab/bridge placement functionality, it is relevant to this research through its common-line merging feature, which is directly related to the shared-tab problem.

The common-line merging feature automatically identifies where two adjacent parts share a boundary and merges the overlapping cut paths so the laser/cutter doesn't traverse the same path twice. This reduces heat warping from multiple laser passes and saves cutting time. The concept of shared cut lines is the geometric foundation for shared tabs — if two parts share a common edge, a tab on that edge inherently holds both parts.

Deepnest is built on SVGNest (a JavaScript nesting library) with a C-based nesting engine for performance. The architecture (78.5% JavaScript, with C++ for the compute-intensive nesting) demonstrates the common pattern of using a higher-level language for UI/orchestration with a compiled language for the geometric engine. The tool supports SVG and DXF formats and includes path approximation for complex curved parts.

## Key Findings
- Common-line merging automatically identifies and unifies shared cut paths between adjacent parts
- No explicit tab/bridge placement feature
- Common-line concept is the geometric foundation for shared-tab placement
- Reduces heat warping and cutting time by avoiding duplicate cuts
- C-based engine for nesting performance, JS for UI

## Technical Details
- **Language**: JavaScript (78.5%), HTML/CSS, C++ (1.5% — nesting engine)
- **Input formats**: SVG, DXF
- **Core feature**: Part packing/nesting optimization
- **Key feature**: Common-line merging (shared edge detection)
- **Base**: Built on SVGNest library
- **License**: Open source (MIT)

## Strengths
- Common-line detection algorithm directly relevant to identifying shared-tab candidates
- Open source — algorithm for detecting shared edges available for study
- Active community and known in the maker/CNC space
- Demonstrates hybrid architecture (JS + C) for nesting geometry

## Weaknesses & Limitations
- No tab placement feature — only relevant for shared-edge detection
- Project maintenance status unclear (last major commits may be dated)
- Path approximation may lose precision for tab placement requirements
- No CNC-specific workholding considerations
- Common-line merging optimized for cutting, not for holding

## Connections
- [RES-002 Common-Edge Nested Cutting] — academic treatment of the common-edge optimization Deepnest implements
- [RES-018 HOMAG Nesting Features] — commercial implementation combining common-line with tabs
- [RES-024 The Fabricator Common Line] — industry perspective on common-line cutting
- [RES-008 Fusion 360 Adjacent Tabs Forum] — the user problem that arises when nesting (like Deepnest) meets tab placement
