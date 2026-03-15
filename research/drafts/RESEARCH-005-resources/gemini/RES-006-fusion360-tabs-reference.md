# Fusion 360 Tabs Reference
> **URL**: https://help.autodesk.com/view/fusion360/ENU/?guid=MFG-REF-2D-CONTOUR-TABS
> **Type**: Commercial
> **Agent**: Gemini
> **Relevance**: High
> **Credibility**: High

## Summary
Autodesk's official reference documentation for Fusion 360 details the configuration and behavior of holding tabs within 2D contour operations. As one of the most widely used CAM packages, Fusion 360 sets a baseline for user expectations regarding tab functionality.

Fusion 360 defines tabs as short, thin pieces of material left behind to prevent the part from moving. Like MecSoft, it offers Rectangular and Triangular shapes. The placement logic is divided into two modes: "By distance" (automatic intervals) and "At points" (manual placement).

The documentation includes practical machining advice, such as setting the bottom of a through-cut to slightly below the stock bottom (-0.05") and compensating by making the tabs higher. It also warns users that automatic placement requires visual inspection, confirming that purely algorithmic placement without collision/feature awareness is often imperfect.

## Key Findings
- Standard automatic placement in Fusion 360 uses a "By distance" metric, placing tabs at set intervals along the toolpath.
- The documentation explicitly recommends visual inspection after automatic placement, implying the algorithm is naive to part features (like placing a tab on a sharp corner).
- Proper tab height calculation must account for over-cutting (cutting below the stock bottom to ensure a clean breakout).

## Technical Details
Tabs in Fusion 360 are configured with explicit width and height parameters. The "Tab Distance" parameter is only active in the automatic "By distance" mode. A known technical quirk is that using "Control Compensation" can cause issues with tab generation, requiring compensation to be handled by the computer instead.

## Strengths
- Represents the industry standard for tab configuration in modern CAD/CAM software.
- Provides practical, actionable advice on tab sizing and depth compensation.

## Weaknesses & Limitations
- The automatic placement algorithm is basic (distance intervals) and seemingly lacks advanced geometric awareness (e.g., avoiding corners).
- No built-in functionality described for handling tabs between nested/adjacent parts.

## Connections
- Corroborates the feature set found in MecSoft (RES-005).
- Provides context for the user frustrations discussed in the Autodesk Forum (RES-008).
