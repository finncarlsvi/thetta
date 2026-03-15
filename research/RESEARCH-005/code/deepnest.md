# Deepnest -- Open Source Nesting

> **URL**: https://github.com/Jack000/Deepnest
> **Type**: Code
> **Agents**: Claude, Codex, Gemini
> **Relevance**: Medium
> **Confidence**: High -- agents agree on feature set and relevance

## Summary
Deepnest is an open-source nesting application designed for laser cutters, plasma cutters, and CNC machines. Its primary function is packing parts into the most compact arrangement possible to save material and reduce cutting time. While Deepnest does not include tab/bridge placement functionality, it is relevant to this research through its common-line merging feature, which is directly related to the shared-tab problem.

The common-line merging feature automatically identifies where two adjacent parts share a boundary and merges the overlapping cut paths so the cutter does not traverse the same path twice. This reduces heat warping from multiple laser passes and saves cutting time. The concept of shared cut lines is the geometric foundation for shared tabs -- if two parts share a common edge, a tab on that edge inherently holds both parts. The geometric logic required to identify a common line is the exact prerequisite logic needed to place a shared holding tab between two parts.

## Key Findings
- Common-line merging automatically identifies and unifies shared cut paths between adjacent parts (all agents)
- No explicit tab/bridge placement feature (all agents)
- Common-line concept is the geometric foundation for shared-tab placement (all agents)
- C-based engine uses No Fit Polygon (NFP) calculations for performance (Gemini)
- Common-line merging is optimized for cutting, not for holding -- an important distinction (Claude)
- Path approximation may lose precision for tab placement requirements (Claude)

## Technical Details
- **Language**: JavaScript (78.5%), HTML/CSS, C++ (1.5% -- nesting engine)
- **Input formats**: SVG, DXF
- **Core feature**: Part packing/nesting optimization
- **Key feature**: Common-line merging (shared edge detection)
- **Base**: Built on SVGNest library
- **License**: Open source (MIT)

## Strengths
- Common-line detection algorithm directly relevant to identifying shared-tab candidates
- Open source -- algorithm for detecting shared edges available for study
- Active community and known in the maker/CNC space
- Widely used and proven nesting engine

## Weaknesses & Limitations
- No tab placement feature -- only relevant for shared-edge detection
- Project maintenance status unclear
- Path approximation may lose precision for tab placement
- No CNC-specific workholding considerations

## Consensus & Disagreements
All agents agree on the common-line feature and its relevance to shared-tab placement. Claude provides the important distinction that common-line merging is optimized for cutting efficiency, not holding force. Gemini highlights NFP calculations. No factual disagreements.

## Connections
- [RES-002] Common-Edge Nested Cutting -- academic treatment of common-edge optimization
- [RES-018] HOMAG Nesting Features -- commercial implementation combining common-line with tabs
- [RES-024] The Fabricator Common Line -- industry perspective on common-line cutting
- [RES-008] Fusion 360 Adjacent Tabs Forum -- the problem that arises when nesting meets tab placement
