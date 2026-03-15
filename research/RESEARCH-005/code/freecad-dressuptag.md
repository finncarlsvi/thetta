# FreeCAD CAM DressupTag (Holding Tags)

> **URL**: https://wiki.freecad.org/Path_DressupTag/en
> **Type**: Code
> **Agents**: Claude, Codex, Gemini
> **Relevance**: High
> **Confidence**: High -- all agents agree on core behavior

## Summary
FreeCAD's Path DressupTag is an open-source implementation of automatic tab (holding tag) placement within a full-featured CAM system. The feature works as a "dressup" -- a post-processing modification applied to an existing toolpath (typically a 2D contour). Rather than generating tabs during initial toolpath creation, the dressup modifies a completed path to add material-retaining tags at specified locations.

The automatic placement algorithm generates tags evenly spaced along the contour, beginning with the longest edge. This edge-length-weighted distribution ensures that longer edges get tabs first, providing better coverage. Tags are intended for convex sections of the profile where they are easier to file down after removal -- a physical heuristic noted by all three agents. Users can adjust the automatic placement by deleting unwanted tags or repositioning them.

The tag parameters include: Angle (the plunge/ascent angle as the tool approaches and leaves the tag), Height (measured from the bottom of the profile cut), Radius (fillet at the tag transition), Width (along the cutting direction), and a Segmentation Factor. This parametric flexibility goes beyond the simple rectangular/triangular dichotomy offered by most commercial tools.

## Key Findings
- Tags placed evenly along contour, starting from longest edge (all agents)
- Dressup pattern: modify existing toolpath rather than generate from scratch (all agents)
- Placement preference for convex sections for easier post-processing (all agents)
- Parameters: Angle, Height, Radius, Width, Segmentation Factor (Claude, Gemini)
- Applying tabs to the final path rather than the base geometry can cause issues if the path has already been optimized for something else (Gemini)
- Open source (LGPL), written in Python (Claude)

## Technical Details
- **Software**: FreeCAD (open source, LGPL)
- **Module**: Path/CAM -> DressupTag
- **Architecture**: Dressup applied to existing 2D contour path
- **Placement algorithm**: Even distribution along contour, weighted by edge length, starting at longest edge
- **Parameters**: Angle (plunge/ascent), Height (from cut bottom), Radius (fillet), Width, Segmentation Factor
- **Tag geometry**: Parametric shape defined by angle + radius, more flexible than rectangular/triangular
- **Language**: Python

## Strengths
- Open source -- full algorithm available for study
- Edge-length-weighted distribution is more sophisticated than simple even spacing
- Parametric tag shape (angle + radius) more flexible than discrete shape choices
- Convex-section preference is a good heuristic for post-processing ease
- Separation of core pathing logic from fixturing logic is architecturally clean (Gemini)

## Weaknesses & Limitations
- Described as "the touchiest of all the dressups and in desperate need of a rewrite" by original author (sliptonic)
- Automatic generation is acknowledged as problematic by the author
- No support for shared tabs between adjacent parts
- Complexity from trying to work on imported G-code (geometry reconstruction)
- Single-contour focus -- doesn't consider nesting context

## Consensus & Disagreements
All agents agree on the core behavior: edge-length-weighted distribution, dressup architecture, convex preference. Claude provided the most complete parameter list and developer quotes. Gemini contributed the architectural critique that applying tabs to a post-optimized path can cause issues. Codex was concise but accurate. No factual disagreements.

## Connections
- [RES-012] FreeCAD DressupTag Source -- the actual Python source code implementing this feature
- [RES-010] CamBam Holding Tabs -- perimeter formula vs. edge-length weighting
- [RES-013] bCNC Tabs -- another open-source Python tab implementation
- [RES-015] viaconstructor -- Python CAM tool with tab support and known bugs
