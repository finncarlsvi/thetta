# Fusion 360 Tabs Reference — Autodesk
> **URL**: https://help.autodesk.com/view/fusion360/ENU/?guid=MFG-REF-2D-CONTOUR-TABS
> **Type**: Product
> **Agent**: Codex
> **Relevance**: High
> **Credibility**: High

## Summary
Autodesk’s Fusion 360 tabs reference documents tabs as a native 2D contour feature and frames them as a core part-retention mechanism for through-cuts. As official product documentation, it is one of the most authoritative implementation references in this set.

The documented controls include rectangular/triangular shape selection and two placement modes: interval-based automatic placement (“By distance”) and user-directed placement (“At points”). This combination is important because it sets user expectations for both automation and deterministic override.

The page also includes practical process notes such as through-cut compensation, visual verification after auto placement, and a compensation-mode caveat. These details are valuable for translating algorithm output into robust CAM behavior that survives post-processing and machine execution.

## Key Findings
- Fusion 360 supports rectangular/triangular tabs and both distance-based automatic placement and point-based manual placement.
- Autodesk guidance includes process-specific recommendations (through-cut depth offsets and compensation settings) that affect tab reliability.

## Technical Details
The tab subsystem is embedded in 2D contour operation parameters. In automatic mode, tab distance controls spacing; in manual mode, user points define exact locations. Implementation-wise, this implies a path annotation layer tied to contour segments with operation-aware validation for compensation and cut depth.

## Strengths
- High-authority vendor documentation with practical machine-level cautions.

## Weaknesses & Limitations
- Documentation explains controls but not the internal scoring/heuristics used by auto placement.

## Connections
- [RES-008-fusion360-adjacent-tabs-forum.md](/Users/finn.carlsvi/thetta/research/drafts/RESEARCH-005-resources/codex/RES-008-fusion360-adjacent-tabs-forum.md) — the forum thread highlights real-world failure cases when adjacent-part tab interactions are not resolved by default auto behavior.
