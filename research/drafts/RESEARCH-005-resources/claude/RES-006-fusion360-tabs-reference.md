# Fusion 360 Tabs Reference — Autodesk
> **URL**: https://help.autodesk.com/view/fusion360/ENU/?guid=MFG-REF-2D-CONTOUR-TABS
> **Type**: Commercial
> **Agent**: Claude
> **Relevance**: High
> **Credibility**: High

## Summary
This is Autodesk's official reference documentation for the tab feature in Fusion 360's 2D Contour CAM operation. Fusion 360 is one of the most widely used CAM tools in the hobbyist and small-shop CNC market, making its tab implementation a de facto standard for user expectations.

Tabs in Fusion 360 are defined as short, thin pieces of material left between the part and stock to prevent part movement during cutting. The system supports two tab shapes: Rectangular (straight-sided, consistent cross-section) and Triangular (tapered, easier to remove but less holding force). Tab positioning can be set to "By distance" (automatic — Fusion places tabs at regular intervals along the contour) or "At points" (manual — user specifies exact locations).

A notable practical detail is the recommended tab height of 0.1125" for through cuts, with the guidance to set the cut bottom 0.05" below stock bottom and increase tab height to compensate. This ensures the cutter fully separates the part except at tab locations. The documentation also notes a known issue with Control Compensation — users must set it to "Computer" for correct toolpath generation around tabs. Visual inspection after automatic placement is recommended, acknowledging that automatic algorithms don't always produce optimal results.

## Key Findings
- Tab shapes: Rectangular and Triangular
- Positioning: "By distance" (automatic spacing) or "At points" (manual)
- Recommended tab height for through cuts: 0.1125"
- Best practice: cut bottom 0.05" below stock bottom, compensate with taller tabs
- Known issue: Control Compensation must be set to "Computer" for correct tab toolpaths
- Visual inspection of automatic placement recommended

## Technical Details
- **Software**: Fusion 360 (Autodesk), 2D Contour operation
- **Tab Shapes**: Rectangular, Triangular
- **Positioning Modes**: By Distance (automatic), At Points (manual)
- **Key Parameters**: Tab Width, Tab Height, Tab Distance
- **Recommended Height**: 0.1125" for through-cuts
- **Workaround**: Set bottom to -0.05" below stock, increase tab height accordingly

## Strengths
- Industry-standard reference from the most popular hobbyist/prosumer CAM tool
- Clear parameter definitions that set user expectations
- Practical tips (bottom offset, tab height compensation) reflect real-world usage
- Two positioning modes (automatic/manual) is the standard UX pattern

## Weaknesses & Limitations
- No details on the automatic distance-based placement algorithm
- Does not address tab placement for adjacent/nested parts
- "At points" mode is fully manual — no smart suggestions
- Known toolpath issues with certain compensation settings suggest implementation complexity

## Connections
- [RES-007 Fusion 360 Tabs Tutorial] — step-by-step walkthrough of the same features
- [RES-008 Fusion 360 Adjacent Tabs Forum] — community struggles with Fusion's lack of shared-tab support
- [RES-005 MecSoft Bridges & Tabs] — same parameter model (shape, height, width, distance/count)
- [RES-027 Mastercam Tabs] — Mastercam's equivalent feature set
