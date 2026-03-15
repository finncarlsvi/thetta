# RES-012: FreeCAD PathDressupTag.py Source Code
**URL**: https://github.com/FreeCAD/FreeCAD/commits/master/src/Mod/Path/PathScripts/PathDressupTag.py
**Type**: Code
**Status**: PARTIAL (commit history only, not full source)

## From Search Results (Forum Analysis of Source Code)
1. **findBottomWire()**: finds all edges whose Z value matches lowest Z, creates Part.Wire, checks isClosed
2. **generateTags(obj, count)**: distributes tabs by edge length
   - Creates edgeDict = { edgeindex: number_of_tabs_on_this_edge }
   - Tabs roughly distributed by length, starting at longest edge
3. Tags described as "the touchiest of all the dressups and in desperate need of a rewrite"
4. Original author (sliptonic) noted he "probably should have never added the automatic tag generation"
5. The automatic generation tries to work on imported G-code too (reconstructing geometry), which adds complexity
