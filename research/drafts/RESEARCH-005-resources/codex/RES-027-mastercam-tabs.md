# Creating and Using Tabs in Mastercam — CamInstructor
> **URL**: https://blog.caminstructor.com/creating-and-using-tabs-in-mastercam
> **Type**: Community
> **Agent**: Codex
> **Relevance**: Medium
> **Credibility**: Medium

## Summary
This Mastercam tutorial describes tab creation in 2D contour operations with both automatic and manual placement workflows. It confirms that mature commercial CAM environments typically provide dual control paths.

Automatic placement can be driven by either explicit tab count or maximum spacing, while manual placement uses contour points for exact control. The tutorial also notes dimension and side-profile customization, indicating that tab geometry is not limited to a single default shape.

For research, this source reinforces common UI/behavior expectations that a new tab planner should match: good automatic defaults, easy manual override, and flexible tab profile parameters for different materials/processes.

## Key Findings
- Mastercam supports automatic tabs by count or max distance and manual tabs by explicit point placement.
- Tab thickness, width, and side profile are user-configurable, including non-vertical profiles.

## Technical Details
The workflow appears integrated into contour chain editing, implying tab entities are attached to geometric chain locations with editable attributes. A practical planner can emulate this by storing tab anchors plus profile metadata and exposing bidirectional auto/manual updates.

## Strengths
- Clear demonstration of dual-mode placement and rich geometric parameterization in a mainstream CAM workflow.

## Weaknesses & Limitations
- Community tutorial does not explain internal algorithms behind automatic mode.

## Connections
- [RES-009-vcarve-tab-customization.md](/Users/finn.carlsvi/thetta/research/drafts/RESEARCH-005-resources/codex/RES-009-vcarve-tab-customization.md) — both show hybrid auto/manual tab paradigms with editable placement after initial generation.
