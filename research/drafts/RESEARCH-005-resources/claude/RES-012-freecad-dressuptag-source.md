# FreeCAD PathDressupTag.py Source Code
> **URL**: https://github.com/FreeCAD/FreeCAD/commits/master/src/Mod/Path/PathScripts/PathDressupTag.py
> **Type**: Code
> **Agent**: Claude
> **Relevance**: High
> **Credibility**: High

## Summary
This resource examines the actual source code of FreeCAD's PathDressupTag feature, providing implementation-level details that complement the wiki documentation (RES-011). The source analysis, gathered from FreeCAD forum discussions of the code, reveals a two-phase algorithm: first find the bottom wire of the contour, then distribute tabs along it proportional to edge lengths.

The `findBottomWire()` function locates all edges whose Z value matches the lowest Z coordinate in the path, constructs a `Part.Wire` from them, and verifies it is closed. This bottom wire represents the final-pass contour where tabs must be placed. The `generateTags(obj, count)` function then distributes the requested number of tabs across the edges of this wire. It builds an `edgeDict` mapping edge indices to the number of tabs allocated to each edge, distributing roughly proportional to edge length and starting with the longest edge.

The code's original author (sliptonic, a FreeCAD core developer) has noted that this is "the touchiest of all the dressups and in desperate need of a rewrite." He also commented that he "probably should have never added the automatic tag generation." The complexity arises partly because the feature attempts to work on imported G-code (not just native FreeCAD geometry), requiring geometry reconstruction from linear/arc segments — a fragile process.

These candid assessments from the original developer are invaluable for Thetta's implementation planning. They highlight the key pitfalls: reconstructing geometry from toolpaths is fragile, edge-based distribution can fail on unusual geometries, and automatic placement requires careful handling of edge cases.

## Key Findings
- `findBottomWire()`: extracts lowest-Z edges, constructs closed wire for tab placement
- `generateTags(obj, count)`: distributes tabs proportional to edge length via `edgeDict`
- Tabs start on longest edge, then distributed to remaining edges
- Original author considers it the "touchiest dressup" needing a rewrite
- Working on imported G-code adds significant complexity (geometry reconstruction)
- Automatic generation was retrospectively considered a mistake by the author

## Technical Details
- **Language**: Python
- **License**: LGPL
- **Key functions**: `findBottomWire()`, `generateTags(obj, count)`
- **Data structure**: `edgeDict = { edge_index: tab_count }` — maps edges to their allocated tab count
- **Algorithm**: Edge-length-proportional distribution starting from longest edge
- **Complexity source**: G-code geometry reconstruction (linear/arc segments → wire → tab placement)
- **Repository**: github.com/FreeCAD/FreeCAD, `src/Mod/Path/PathScripts/PathDressupTag.py`

## Strengths
- Real, production code — demonstrates actual implementation challenges
- Edge-length weighting is a reasonable heuristic
- Developer commentary provides honest assessment of pitfalls
- LGPL license allows study and adaptation
- Demonstrates the dressup pattern (post-process existing path)

## Weaknesses & Limitations
- Code acknowledged as fragile and needing rewrite
- G-code reconstruction adds unnecessary complexity (Thetta has native geometry, avoiding this issue)
- No geometry-aware placement (corner avoidance, concavity handling)
- No multi-part or shared-tab awareness
- Edge-based distribution may place tabs at edge boundaries (suboptimal locations)

## Connections
- [RES-011 FreeCAD DressupTag] — wiki documentation for this same code
- [RES-013 bCNC Tabs] — simpler Python tab implementation without edge-length weighting
- [RES-015 viaconstructor] — another Python CAM with tab bugs, similar challenges
- [RES-010 CamBam Holding Tabs] — CamBam's formula-based approach may be more robust than edge-proportional
