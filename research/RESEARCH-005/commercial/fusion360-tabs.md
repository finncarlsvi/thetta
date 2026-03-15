# Fusion 360 Tabs Reference -- Autodesk

> **URL**: https://help.autodesk.com/view/fusion360/ENU/?guid=MFG-REF-2D-CONTOUR-TABS
> **Type**: Commercial
> **Agents**: Claude, Codex, Gemini
> **Relevance**: High
> **Confidence**: High -- official vendor documentation

## Summary
Autodesk's official reference documentation for the tab feature in Fusion 360's 2D Contour CAM operation. Fusion 360 is one of the most widely used CAM tools in the hobbyist and small-shop CNC market, making its tab implementation a de facto standard for user expectations.

Tabs are defined as short, thin pieces of material left between the part and stock to prevent part movement during cutting. The system supports two tab shapes: Rectangular and Triangular. Tab positioning can be set to "By distance" (automatic intervals along the contour) or "At points" (manual placement). The documentation includes practical machining advice: recommended tab height of 0.1125 inches for through cuts, with guidance to set the cut bottom 0.05 inches below stock bottom and increase tab height to compensate. Visual inspection after automatic placement is explicitly recommended, acknowledging that automatic algorithms do not always produce optimal results.

## Key Findings
- Tab shapes: Rectangular and Triangular (all agents)
- Positioning: "By distance" (automatic) or "At points" (manual) (all agents)
- Recommended tab height for through cuts: 0.1125 inches (Claude)
- Best practice: cut bottom 0.05 inches below stock bottom, compensate with taller tabs (Claude, Gemini)
- Known issue: Control Compensation must be set to "Computer" for correct tab toolpaths (Claude, Codex)
- Visual inspection of automatic placement recommended, implying algorithmic naivety (Gemini)

## Technical Details
- **Software**: Fusion 360 (Autodesk), 2D Contour operation
- **Tab Shapes**: Rectangular, Triangular
- **Positioning Modes**: By Distance (automatic), At Points (manual)
- **Key Parameters**: Tab Width, Tab Height, Tab Distance
- **Recommended Height**: 0.1125 inches for through-cuts

## Strengths
- Industry-standard reference from the most popular hobbyist/prosumer CAM tool
- Clear parameter definitions that set user expectations
- Practical tips reflect real-world usage
- Two positioning modes is the standard UX pattern

## Weaknesses & Limitations
- No details on the automatic distance-based placement algorithm
- Does not address tab placement for adjacent/nested parts
- "At points" mode is fully manual -- no smart suggestions
- Known toolpath issues with certain compensation settings

## Consensus & Disagreements
All agents agree on shapes, modes, and parameters. Claude provides the most specific dimensional recommendations. Gemini highlights the vendor's own recommendation for visual inspection as implicit admission of algorithmic naivety. No factual disagreements.

## Connections
- [RES-007] Fusion 360 Tabs Tutorial -- step-by-step walkthrough
- [RES-008] Fusion 360 Adjacent Tabs Forum -- community struggles with lack of shared-tab support
- [RES-005] MecSoft Bridges & Tabs -- same parameter model
- [RES-027] Mastercam Tabs -- equivalent feature set
