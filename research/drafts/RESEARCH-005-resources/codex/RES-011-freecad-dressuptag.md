# FreeCAD CAM DressupTag (Holding Tags)
> **URL**: https://wiki.freecad.org/Path_DressupTag/en
> **Type**: Code
> **Agent**: Codex
> **Relevance**: High
> **Credibility**: High

## Summary
FreeCAD’s DressupTag feature is an open implementation reference for adding holding tags to existing contour paths. It is relevant because it demonstrates a production-grade, scriptable approach in an open-source CAM ecosystem.

The documentation describes both geometric parameters (height, width, radius, angle, segmentation factor) and placement behavior (automatic even spacing that starts from the largest edge). This blend of shape and placement controls closely matches what users expect in closed-source CAM tools.

For this research task, DressupTag is valuable because it gives both UI-level semantics and a path to inspect source behavior. It also surfaces practical guidance such as preferring convex regions for easier finishing, which can inform heuristic penalties.

## Key Findings
- FreeCAD auto-generates tags with even spacing along the contour and prioritizes placement from the longest edge.
- Parameters include transition geometry (angle/radius) in addition to width/height, implying trajectory-aware tag shaping.

## Technical Details
DressupTag operates as a post-process “dressup” layer on a base toolpath. The tag generator must map contour geometry to tag anchors, then alter motion segments to leave material bridges. User edits (delete/reposition) indicate mutable tag metadata linked to specific path intervals.

## Strengths
- Open-source feature with documented parameters and behavior suitable for direct adaptation.

## Weaknesses & Limitations
- Documentation-level overview does not fully expose edge-case handling or failure conditions.

## Connections
- [RES-012-freecad-dressuptag-source.md](/Users/finn.carlsvi/thetta/research/drafts/RESEARCH-005-resources/codex/RES-012-freecad-dressuptag-source.md) — source-level analysis clarifies internals behind the documented placement behavior.
