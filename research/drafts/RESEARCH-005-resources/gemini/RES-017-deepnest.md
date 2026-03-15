# Deepnest Open Source Nesting
> **URL**: https://github.com/Jack000/Deepnest
> **Type**: Code
> **Agent**: Gemini
> **Relevance**: Medium
> **Credibility**: High

## Summary
Deepnest is one of the most well-known open-source 2D nesting applications, primarily targeting laser and plasma cutters. Built on top of the SVGNest library, it utilizes a C-based nesting engine for performance, wrapped in a JavaScript/HTML/CSS interface.

The most critical feature of Deepnest for this research is its "common line" merging capability. Deepnest is capable of identifying when two parts share an identical, flush edge and merges those paths so the laser only makes a single cut. This significantly reduces cutting time and limits heat warping by avoiding multiple passes over the same area.

While Deepnest does not explicitly document a holding tab or micro-joint feature (as it primarily targets laser/plasma where vacuum hold-down or heavy plate weight is assumed), its handling of common-line geometry is directly applicable to the shared-tab problem in CNC routing.

## Key Findings
- Open-source algorithms exist for detecting and merging common lines between adjacent nested parts.
- Common-line merging is a highly sought-after feature for material efficiency and heat reduction.
- The geometric logic required to identify a common line is the exact prerequisite logic needed to place a shared holding tab between two parts.

## Technical Details
Deepnest relies on path approximation for complex parts, converting splines/arcs to line segments for faster collision detection. The architecture is split between a fast C++ core (for the heavy lifting of the NFP - No Fit Polygon calculations) and a JavaScript frontend.

## Strengths
- Highly proven, widely used open-source nesting engine.
- Successfully implements common-line merging, a complex geometric task.

## Weaknesses & Limitations
- Does not explicitly handle holding tabs or Z-axis retractions.
- Primarily focused on laser/plasma workflows rather than physical CNC routing where tabs are mandatory.

## Connections
- The common-line feature is the exact prerequisite needed to solve the adjacent tab problem discussed in the Autodesk Forum (RES-008).
- The web-tech wrapper shares similarities with Kiri:Moto (RES-016).
