# FreeCAD PathDressupTag.py Source Code

> **URL**: https://github.com/FreeCAD/FreeCAD/commits/master/src/Mod/Path/PathScripts/PathDressupTag.py
> **Type**: Code
> **Agents**: Claude, Codex, Gemini
> **Relevance**: High
> **Confidence**: High -- all agents agree on implementation details

## Summary
This resource examines the actual source code of FreeCAD's PathDressupTag feature, providing implementation-level details that complement the wiki documentation (RES-011). The source analysis reveals a two-phase algorithm: first find the bottom wire of the contour, then distribute tabs along it proportional to edge lengths.

The `findBottomWire()` function locates all edges whose Z value matches the lowest Z coordinate in the path, constructs a `Part.Wire` from them, and verifies it is closed. This bottom wire represents the final-pass contour where tabs must be placed. The `generateTags(obj, count)` function then distributes the requested number of tabs across the edges of this wire. It builds an `edgeDict` mapping edge indices to the number of tabs allocated to each edge, distributing roughly proportional to edge length and starting with the longest edge.

The code's original author (sliptonic, a FreeCAD core developer) has noted that this is "the touchiest of all the dressups and in desperate need of a rewrite" and that he "probably should have never added the automatic tag generation." The complexity arises partly because the feature attempts to work on imported G-code, requiring geometry reconstruction from linear/arc segments -- a fragile process. Thetta avoids this problem entirely because it has native geometry rather than imported G-code.

## Key Findings
- `findBottomWire()`: extracts lowest-Z edges, constructs closed wire for tab placement (Claude, Gemini)
- `generateTags(obj, count)`: distributes tabs proportional to edge length via `edgeDict` (all agents)
- Tabs start on longest edge, then distributed to remaining edges (all agents)
- Original author considers it the "touchiest dressup" needing a rewrite (all agents)
- Automatic generation was retrospectively considered a mistake by the author (Claude)
- Proportional length distribution does not account for cutting forces, part weight, or routing mechanics (Gemini)

## Technical Details
- **Language**: Python
- **License**: LGPL
- **Key functions**: `findBottomWire()`, `generateTags(obj, count)`
- **Data structure**: `edgeDict = { edge_index: tab_count }` -- maps edges to their allocated tab count
- **Algorithm**: Edge-length-proportional distribution starting from longest edge
- **Complexity source**: G-code geometry reconstruction (linear/arc segments -> wire -> tab placement)
- **Repository**: github.com/FreeCAD/FreeCAD, `src/Mod/Path/PathScripts/PathDressupTag.py`

## Strengths
- Real, production code -- demonstrates actual implementation challenges
- Edge-length weighting is a reasonable heuristic
- Developer commentary provides honest assessment of pitfalls
- LGPL license allows study and adaptation

## Weaknesses & Limitations
- Code acknowledged as fragile and needing rewrite
- G-code reconstruction adds unnecessary complexity (Thetta has native geometry)
- No geometry-aware placement (corner avoidance, concavity handling)
- No multi-part or shared-tab awareness
- Edge-based distribution may place tabs at edge boundaries (suboptimal locations)

## Consensus & Disagreements
Highly consistent across all three agents. The function names, edgeDict data structure, and maintainer quotes are reported identically. Claude provides the additional "should have never added automatic generation" quote. Gemini uniquely criticizes that the algorithm ignores cutting forces and physical mechanics. No factual disagreements.

## Connections
- [RES-011] FreeCAD DressupTag -- wiki documentation for this same code
- [RES-013] bCNC Tabs -- simpler Python tab implementation without edge-length weighting
- [RES-015] viaconstructor -- another Python CAM with tab bugs
- [RES-010] CamBam Holding Tabs -- CamBam's formula-based approach as alternative
