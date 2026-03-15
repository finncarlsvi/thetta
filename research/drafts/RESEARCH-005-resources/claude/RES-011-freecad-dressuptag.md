# FreeCAD CAM DressupTag (Holding Tags)
> **URL**: https://wiki.freecad.org/Path_DressupTag/en
> **Type**: Code
> **Agent**: Claude
> **Relevance**: High
> **Credibility**: High

## Summary
FreeCAD's Path DressupTag is an open-source implementation of automatic tab (holding tag) placement within a full-featured CAM system. The feature works as a "dressup" — a post-processing modification applied to an existing toolpath (typically a 2D contour). Rather than generating tabs during initial toolpath creation, the dressup modifies a completed path to add material-retaining tags at specified locations.

The automatic placement algorithm generates tags evenly spaced along the contour, beginning with the longest edge. This edge-length-weighted distribution ensures that longer edges get tabs first, providing better coverage. Tags are intended for convex sections of the profile where they're easier to file down after removal. Users can adjust the automatic placement by deleting unwanted tags or repositioning them.

The tag parameters include: Angle (the plunge/ascent angle as the tool approaches and leaves the tag), Height (measured from the bottom of the profile cut), Radius (fillet at the tag transition), Width (along the cutting direction), and a Segmentation Factor. The angle and radius parameters control the shape of the tag transition, allowing smooth or abrupt approaches. This parametric flexibility goes beyond the simple rectangular/triangular dichotomy offered by most commercial tools.

FreeCAD's implementation is particularly relevant to Thetta because it's open source (LGPL), written in Python, and demonstrates a complete tag placement pipeline from automatic generation through toolpath modification.

## Key Findings
- Tags placed evenly along contour, starting from longest edge
- Dressup pattern: modify existing toolpath rather than generate from scratch
- Parameters: Angle, Height, Radius, Width, Segmentation Factor
- Placement preference for convex sections (easier removal)
- Open source (LGPL) — code available for study and adaptation

## Technical Details
- **Software**: FreeCAD (open source, LGPL)
- **Module**: Path/CAM → DressupTag
- **Architecture**: Dressup applied to existing 2D contour path
- **Placement algorithm**: Even distribution along contour, weighted by edge length, starting at longest edge
- **Parameters**: Angle (plunge/ascent), Height (from cut bottom), Radius (fillet), Width, Segmentation Factor
- **Tag geometry**: Parametric shape defined by angle + radius, more flexible than rectangular/triangular
- **Language**: Python

## Strengths
- Open source — full algorithm available for study
- Edge-length-weighted distribution is more sophisticated than simple even spacing
- Parametric tag shape (angle + radius) more flexible than discrete shape choices
- Convex-section preference is a good heuristic for post-processing ease
- Mature codebase integrated into a full CAM system

## Weaknesses & Limitations
- Described as "the touchiest of all the dressups and in desperate need of a rewrite" by original author
- Automatic generation is acknowledged as problematic (author suggests it shouldn't have been added)
- No support for shared tabs between adjacent parts
- Complexity from trying to work on imported G-code (geometry reconstruction)
- Single-contour focus — doesn't consider nesting context

## Connections
- [RES-012 FreeCAD DressupTag Source] — the actual Python source code implementing this feature
- [RES-010 CamBam Holding Tabs] — CamBam's perimeter formula vs. FreeCAD's edge-length weighting
- [RES-013 bCNC Tabs] — another open-source Python tab implementation, simpler approach
- [RES-015 viaconstructor] — Python CAM tool with tab support, known circular path issues
