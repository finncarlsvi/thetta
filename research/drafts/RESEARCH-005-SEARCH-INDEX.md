# Research 005 Search Index

Cache: 28/28 resources downloaded (17 cached/partial, 11 from search results only)

## RES-001: Effective location of micro joints (Kandasamy & Udhayakumar 2020)
- **File**: `RESEARCH-005-cache/RES-001-micro-joints-genetic-algorithm.md`
- **URL**: https://link.springer.com/article/10.1007/s12289-019-01488-1
- **Type**: paper
- **Status**: PARTIAL (paywall)
- **Key terms**: micro-joint, centroid, genetic algorithm, heuristic, piercing point, sheet metal, AWJ, laser, plasma
- **Summary**: Proposes centroid-based heuristic to place 3 micro-joints per part, combined with GA to optimize cutting sequence. Most directly relevant paper on automatic tab placement.

## RES-002: Optimizing Cutting Sequences for Common-Edge Nested Parts
- **File**: `RESEARCH-005-cache/RES-002-common-edge-nested-cutting.md`
- **URL**: https://www.sciencedirect.com/science/article/abs/pii/S0010448523001422
- **Type**: paper
- **Status**: FETCH_FAILED
- **Key terms**: common-edge, nesting, cutting sequence, shared edge, path optimization
- **Summary**: Addresses cutting optimization when parts share edges. Relevant to shared-tab problem between adjacent nested parts.

## RES-003: Cutting path optimization using two-step GA
- **File**: `RESEARCH-005-cache/RES-003-cutting-path-two-step-ga.md`
- **URL**: https://www.tandfonline.com/doi/full/10.1080/00207540600579615
- **Type**: paper
- **Status**: FETCH_FAILED
- **Key terms**: genetic algorithm, TSP, piercing point, cutting path, non-productive distance
- **Summary**: Formulates CNC cutting path as generalized TSP. Two-step GA for piercing point optimization + part sequencing.

## RES-004: Optimization Models of Tool Path Problem for CNC Sheet Metal
- **File**: `RESEARCH-005-cache/RES-004-tool-path-optimization-models.md`
- **URL**: https://www.sciencedirect.com/science/article/pii/S2405896316307972
- **Type**: paper
- **Status**: FETCH_FAILED
- **Key terms**: tool path, mathematical model, GTSP, thermal cutting, sheet metal
- **Summary**: Mathematical formalization of tool path optimization as GTSP with thermal cutting constraints.

## RES-005: MecSoft Bridges & Tabs Explored
- **File**: `RESEARCH-005-cache/RES-005-mecsoft-bridges-tabs.md`
- **URL**: https://mecsoft.com/blog/bridges-tabs-explored/
- **Type**: commercial
- **Status**: PARTIAL
- **Key terms**: bridges, tabs, rectangular, triangular, bridge height, bridge length, RhinoCAM, automatic placement
- **Summary**: MecSoft's documentation on bridge types, parameters, and automatic/manual placement in their CAM software.

## RES-006: Fusion 360 Tabs Reference (Autodesk)
- **File**: `RESEARCH-005-cache/RES-006-fusion360-tabs-reference.md`
- **URL**: https://help.autodesk.com/view/fusion360/ENU/?guid=MFG-REF-2D-CONTOUR-TABS
- **Type**: commercial
- **Status**: PARTIAL
- **Key terms**: Fusion 360, tab shape, tab distance, rectangular, triangular, by distance, at points
- **Summary**: Autodesk's reference for tab settings in 2D contour operations: shapes, positioning modes, height/width parameters.

## RES-007: Fusion 360 Tabs Tutorial (CAD CAM Lessons)
- **File**: `RESEARCH-005-cache/RES-007-fusion360-tabs-tutorial.md`
- **URL**: https://cadcamlessons.com/fusion-360-tabs/
- **Type**: community
- **Status**: CACHED
- **Key terms**: Fusion 360, tab width, tab height, tab distance, rectangular, triangular, CAM
- **Summary**: Step-by-step tutorial on Fusion 360 tab configuration: shape options, positioning methods, parameters.

## RES-008: How to position tabs for adjacent pieces (Autodesk Forum)
- **File**: `RESEARCH-005-cache/RES-008-fusion360-adjacent-tabs-forum.md`
- **URL**: https://forums.autodesk.com/t5/fusion-manufacture-forum/how-to-position-tabs-for-cutting-out-adjacent-pieces/td-p/12566903
- **Type**: community
- **Status**: FETCH_FAILED
- **Key terms**: adjacent pieces, shared tabs, duplicate tabs, tab alignment, nested parts, common edge
- **Summary**: Forum discussion on the core problem of tabs being destroyed when adjacent parts are cut. Solutions include manual alignment and increased spacing.

## RES-009: VCarve/Aspire Tab Customization
- **File**: `RESEARCH-005-cache/RES-009-vcarve-tab-customization.md`
- **URL**: https://www.learnyourcnc.com/blog/how-to-customize-design-tabs-in-vectric
- **Type**: community
- **Status**: PARTIAL
- **Key terms**: VCarve, Aspire, Vectric, 3D tabs, automatic tabs, min/max tabs, tab distance
- **Summary**: VCarve offers auto/manual tab placement, 3D tabs (smooth ramp instead of retract), drag-to-reposition.

## RES-010: CamBam Holding Tabs Documentation
- **File**: `RESEARCH-005-cache/RES-010-cambam-holding-tabs.md`
- **URL**: http://www.cambam.info/doc/plus/cam/HoldingTabs.htm
- **Type**: commercial
- **Status**: FETCH_FAILED (expired cert)
- **Key terms**: CamBam, tab distance, perimeter, minimum tabs, maximum tabs, size threshold, automatic
- **Summary**: Most explicit algorithmic description found: tabs = perimeter ÷ tab_distance, clamped to min/max, with size threshold to skip small shapes.

## RES-011: FreeCAD CAM DressupTag
- **File**: `RESEARCH-005-cache/RES-011-freecad-dressuptag.md`
- **URL**: https://wiki.freecad.org/Path_DressupTag/en
- **Type**: code
- **Status**: CACHED
- **Key terms**: FreeCAD, DressupTag, holding tags, angle, height, radius, width, automatic placement, contour
- **Summary**: FreeCAD's tag feature: auto-places tags evenly along contour starting from largest edge. Parameters: angle, height, radius, width.

## RES-012: FreeCAD PathDressupTag.py Source Code
- **File**: `RESEARCH-005-cache/RES-012-freecad-dressuptag-source.md`
- **URL**: https://github.com/FreeCAD/FreeCAD/commits/master/src/Mod/Path/PathScripts/PathDressupTag.py
- **Type**: code
- **Status**: PARTIAL
- **Key terms**: FreeCAD, Python, findBottomWire, generateTags, edgeDict, edge length distribution
- **Summary**: Source analysis reveals: find bottom wire → distribute tabs proportional to edge length → described as "touchiest dressup, needs rewrite."

## RES-013: bCNC GRBL CNC Sender with Cutout Tabs
- **File**: `RESEARCH-005-cache/RES-013-bcnc-repo.md`
- **URL**: https://github.com/vlachoudis/bCNC
- **Type**: code
- **Status**: PARTIAL
- **Key terms**: bCNC, Python, GRBL, cutout tabs, CAM, G-code editor
- **Summary**: Python-based CNC sender with built-in tab support. Tab functionality in cut tool, default 3 tabs.

## RES-014: bCNC Tabs Tool Tutorial
- **File**: `RESEARCH-005-cache/RES-014-bcnc-tabs-tutorial.md`
- **URL**: https://github.com/vlachoudis/bCNC/wiki/Tutorials:-Tabs-tool
- **Type**: community
- **Status**: PARTIAL
- **Key terms**: bCNC, tabs tool, last pass, execute, holding tabs
- **Summary**: Quick tutorial: select last pass block, execute to add 3 tabs. Must disable "only leave selected islands uncut."

## RES-015: viaconstructor Python CAM tool
- **File**: `RESEARCH-005-cache/RES-015-viaconstructor.md`
- **URL**: https://github.com/multigcs/viaconstructor
- **Type**: code
- **Status**: CACHED
- **Key terms**: Python, C++, DXF, SVG, tabs, pockets, islands, nearest-neighbor, automatic offset
- **Summary**: Python/C++ CAM tool with tab support. Known issue: tab generation fails on circular paths.

## RES-016: Kiri:Moto Browser-based CAM
- **File**: `RESEARCH-005-cache/RES-016-kirimoto.md`
- **URL**: https://github.com/GridSpace/grid-apps
- **Type**: code
- **Status**: PARTIAL
- **Key terms**: Kiri:Moto, browser, JavaScript, CNC, laser, 3D printing, tabs, open source
- **Summary**: Browser-based multi-modal CAM tool with CNC tab support. 13+ years development, self-hostable.

## RES-017: Deepnest Open Source Nesting
- **File**: `RESEARCH-005-cache/RES-017-deepnest.md`
- **URL**: https://github.com/Jack000/Deepnest
- **Type**: code
- **Status**: CACHED
- **Key terms**: nesting, common line, SVG, DXF, laser, plasma, line merging, C engine
- **Summary**: Open-source nesting tool with common-line merging. No explicit tab placement but relevant for shared-edge context.

## RES-018: HOMAG Stay-Down, Common Line, Tabs, Onion Skinning
- **File**: `RESEARCH-005-cache/RES-018-homag-nesting-features.md`
- **URL**: https://docs.homag.cloud/en/news/article/stay-down-common-line-include-tabs-onion-skinning
- **Type**: commercial
- **Status**: CACHED
- **Key terms**: HOMAG, woodWOP, intelliDivide, stay-down, common line, retaining bars, onion skinning, nesting
- **Summary**: HOMAG's integrated approach: stay-down + common line + retaining bars (tabs) + onion skinning for CNC nesting.

## RES-019: Tabs vs Onion Skinning (WOODWEB Forum)
- **File**: `RESEARCH-005-cache/RES-019-woodweb-tabs-vs-onionskin.md`
- **URL**: https://www.woodweb.com/cgi-bin/forums/cnc.pl?read=818617
- **Type**: community
- **Status**: FETCH_FAILED (expired)
- **Key terms**: tabs, onion skinning, vacuum, adjacent parts, tab alignment
- **Summary**: Forum on tabs vs onion skin. Notes that misaligned tabs between adjacent parts get disconnected.

## RES-020: Tabbing vs Onion-Skinning for Small Parts
- **File**: `RESEARCH-005-cache/RES-020-woodweb-tabbing-small-parts.md`
- **URL**: https://woodweb.com/knowledge_base/Tabbing_Versus_OnionSkinning_for_Small_Parts.html
- **Type**: community
- **Status**: CACHED
- **Key terms**: tabs, onion skin, small parts, vacuum, two-pass, compression bits
- **Summary**: Practical comparison: thick tabs for strong hold, onion skin for easier cleanup. Tips on vacuum optimization.

## RES-021: How to optimize use of tabs (CNCZone)
- **File**: `RESEARCH-005-cache/RES-021-cnczone-optimize-tabs.md`
- **URL**: https://www.cnczone.com/forums/diy-cnc-router-table-machines/210140-optimize-use-tabs.html
- **Type**: community
- **Status**: FETCH_FAILED
- **Key terms**: tab count, minimum 3, straight edge, perimeter division, tab size, 0.75" × 0.125"
- **Summary**: Minimum 3 tabs to prevent rotation. Place on straight edges. Common size: 0.75" × 0.125".

## RES-022: PCB Tab Routing Masterclass (AllPCB)
- **File**: `RESEARCH-005-cache/RES-022-pcb-tab-routing-masterclass.md`
- **URL**: https://www.allpcb.com/blog/pcb-design/tab-routing-masterclass-optimizing-breakaway-tabs-for-clean-pcb-separation.html
- **Type**: community
- **Status**: CACHED
- **Key terms**: PCB, tab routing, mouse bites, breakaway tabs, stress distribution, FR-4, 50-75mm spacing
- **Summary**: PCB tab placement rules: one every 50-75mm, 3-5 per side, away from stress/corners. Simulation for stress prediction.

## RES-023: PCB Mouse Bites and V-Scores (Altium)
- **File**: `RESEARCH-005-cache/RES-023-altium-mouse-bites.md`
- **URL**: https://resources.altium.com/p/mouse-bites-and-v-scores-how-depanelize-pcbs
- **Type**: community
- **Status**: CACHED
- **Key terms**: mouse bites, V-score, panelization, depanelization, tangent holes, Altium
- **Summary**: PCB depanelization techniques: mouse bites (holes tangent to edge), V-scores. Design rules for both approaches.

## RES-024: When to cut a common line (The Fabricator)
- **File**: `RESEARCH-005-cache/RES-024-fabricator-common-line.md`
- **URL**: https://www.thefabricator.com/thefabricator/article/punching/when-to-cut-a-common-line
- **Type**: community
- **Status**: FETCH_FAILED
- **Key terms**: common line, microtabs, shared edge, thermal cutting, kerf, skeleton
- **Summary**: Common-line cutting with microtabs adds stability. Parts stay in nest until snapped out. Heat/kerf concerns.

## RES-025: Workholding Best Practices for Tabbing (Mate)
- **File**: `RESEARCH-005-cache/RES-025-mate-tabbing-practices.md`
- **URL**: https://www.mate.com/workholding-best-practices-for-tabbing/
- **Type**: commercial
- **Status**: CACHED
- **Key terms**: 5-axis, tab location, tab geometry, rigidity, CAM support, DynoMount
- **Summary**: 5-axis tabbing: tab location (non-machined face), geometry (rigidity vs easy removal), material-dependent sizing.

## RES-026: SendCutSend CNC Routing Tabs
- **File**: `RESEARCH-005-cache/RES-026-sendcutsend-tabs.md`
- **URL**: https://sendcutsend.com/services/cnc-routing/tabs/
- **Type**: commercial
- **Status**: CACHED
- **Key terms**: SendCutSend, automatic, fixturing tabs, part geometry, small tabs
- **Summary**: Fully automatic tab placement based on part geometry. No custom placement. Tabs outside part geometry.

## RES-027: Creating and Using Tabs in Mastercam
- **File**: `RESEARCH-005-cache/RES-027-mastercam-tabs.md`
- **URL**: https://blog.caminstructor.com/creating-and-using-tabs-in-mastercam
- **Type**: community
- **Status**: CACHED
- **Key terms**: Mastercam, 2D contour, automatic, number of tabs, max distance, tab thickness, tab width
- **Summary**: Mastercam tabs: automatic by count or max distance, manual by point placement. Customizable shape/dimensions.

## RES-028: Maslow CNC Tabs Strategy Discussion
- **File**: `RESEARCH-005-cache/RES-028-maslow-tabs-strategy.md`
- **URL**: https://forums.maslowcnc.com/t/help-with-tabs-strategy/432
- **Type**: community
- **Status**: CACHED
- **Key terms**: Maslow, tab thickness, material ratio, multi-pass, removal tools
- **Summary**: Practical tab sizing: 0.255" for 0.500" material, width = 1.5× depth. Multi-pass cutting strategy.
