# How to position tabs for cutting out adjacent pieces
> **URL**: https://forums.autodesk.com/t5/fusion-manufacture-forum/how-to-position-tabs-for-cutting-out-adjacent-pieces/td-p/12566903
> **Type**: Community
> **Agent**: Gemini
> **Relevance**: High
> **Credibility**: Medium

## Summary
This Autodesk forum thread directly addresses one of the most complex challenges in CNC nesting: the "shared tab" problem. Users discuss the issue where tabs placed on one part are inadvertently destroyed when the cutting tool profiles an adjacent, closely nested part.

The core problem is interference. If two parts share a common cut line (or have a gap smaller than the tool diameter), the toolpath for Part B will mill right through the tab that was meant to hold Part A. Users note that unless tabs are perfectly duplicated and aligned on both adjacent edges, they will be severed.

The solutions proposed by the community highlight the lack of automatic handling in Fusion 360. Users either manually draw the tabs into the CAD model (bypassing the CAM tab feature entirely), manually align tabs using points, or simply space the parts far enough apart (e.g., a 1/4" gap) so that the toolpaths do not overlap at the tab locations.

## Key Findings
- Standard CAM software (like Fusion 360) often fails to protect tabs from adjacent toolpaths in tight nests.
- If tabs on adjacent parts are not perfectly aligned, the second contour pass will destroy the first part's tab.
- Current user workarounds involve tedious manual alignment or sacrificing material efficiency by increasing part spacing.

## Technical Details
The failure occurs because CAM operations are typically generated per-part (or per-contour). The toolpath generator for Part B is unaware of the safe Z-height regions (tabs) established for Part A. To fix this automatically, a global pathing engine must recognize tab locations as protected zones across all adjacent paths.

## Strengths
- Explicitly defines the "shared tab" failure mode in commercial software.
- Demonstrates the real-world need for algorithmic solutions to adjacent tab alignment.

## Weaknesses & Limitations
- Forum discussions provide workarounds, not formal algorithmic solutions.
- Specific to Fusion 360's limitations, though the problem is universal.

## Connections
- Directly relates to the common-edge cutting theories in RES-002 and RES-017.
- Highlights a major limitation of the systems described in RES-005 and RES-006.
