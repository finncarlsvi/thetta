# FreeCAD CAM DressupTag
> **URL**: https://wiki.freecad.org/Path_DressupTag/en
> **Type**: Code
> **Agent**: Gemini
> **Relevance**: High
> **Credibility**: High

## Summary
FreeCAD's Path workbench handles holding tabs through a feature called "DressupTag." Instead of modifying the base geometry or the initial toolpath generation, DressupTag acts as a post-processing modifier (a "dress-up") applied to an existing 2D contour path.

The documentation explains that these tags are intended to be broken off by hand or chisel and filed flat later. A unique piece of guidance provided is that tags should ideally be placed on convex sections of the part, as these areas are significantly easier to file down smoothly compared to concave inner corners.

The automatic placement algorithm in FreeCAD generates tags evenly spaced along the contour, notably beginning the distribution from the longest edge. Users retain the ability to delete or reposition these automatically generated tags, reflecting the industry consensus that manual override is necessary.

## Key Findings
- FreeCAD implements tabs as a post-processing "dress-up" applied over an existing, complete contour path.
- The automatic placement algorithm prioritizes placing the first tabs on the longest available edge.
- A key physical heuristic is noted: tabs are much easier to post-process (file/sand) when placed on convex curves or straight lines rather than concave corners.

## Technical Details
The DressupTag feature includes parameters for Angle (ramp angle for plunge/ascent), Height (measured from the bottom of the profile cut), Radius (for filleting the tab base), and Width. It relies heavily on segmenting the existing G-code/path objects rather than the original CAD geometry.

## Strengths
- Open-source implementation provides transparency into how a major CAD/CAM package handles the feature.
- Treating tabs as a "dress-up" separates the core pathing logic from the fixturing logic.

## Weaknesses & Limitations
- Applying tabs to the final path rather than the base geometry can cause issues if the path has already been optimized for something else.
- As noted in the source code analysis (RES-012), this implementation is known to be fragile.

## Connections
- The source code implementing this documentation is analyzed in RES-012.
- The preference for convex edges aligns with the straight-edge recommendations in RES-021.
