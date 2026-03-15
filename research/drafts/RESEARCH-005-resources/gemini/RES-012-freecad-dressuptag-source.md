# FreeCAD PathDressupTag.py Source Code
> **URL**: https://github.com/FreeCAD/FreeCAD/commits/master/src/Mod/Path/PathScripts/PathDressupTag.py
> **Type**: Code
> **Agent**: Gemini
> **Relevance**: High
> **Credibility**: High

## Summary
An analysis of the FreeCAD source code (specifically `PathDressupTag.py`) reveals the internal mechanics of their open-source tab placement algorithm. The code provides a concrete example of how automatic distribution is handled programmatically in a production environment.

The algorithm relies on analyzing the edges of the "bottom wire" (the lowest Z-height contour). The core function, `generateTags`, distributes the requested number of tabs proportionally based on edge length. It creates an `edgeDict` mapping edge indices to the number of tabs assigned to that edge.

Interestingly, the source code and developer comments reveal the fragility of this approach. The original author (sliptonic) noted that this feature is "the touchiest of all the dressups and in desperate need of a rewrite." The complexity stems from attempting to reconstruct geometry and distribute tabs along raw G-code paths (including imported G-code) rather than working strictly from the original CAD boundary.

## Key Findings
- FreeCAD distributes tabs mathematically based on the proportional length of each edge segment (longer edges get more tabs).
- Applying tab logic as a post-processor to G-code (rather than to the geometric model) introduces severe fragility and complexity.
- Even in established open-source tools, automatic tab placement algorithms are often considered problematic and difficult to maintain.

## Technical Details
The script uses a `findBottomWire()` function to locate all edges matching the lowest Z-value, creating a `Part.Wire` to ensure the contour is closed. It then iterates through these edges, using their lengths as weights to distribute the total requested tab count.

## Strengths
- Provides a real-world, open-source Python implementation of edge-proportional tab distribution.
- Highlights the architectural pitfalls of applying tabs too late in the CAM pipeline.

## Weaknesses & Limitations
- The code is self-admittedly "touchy" and brittle.
- Proportional length distribution does not account for cutting forces, part weight, or the physical mechanics of the routing process.

## Connections
- This is the code implementation of the documentation found in RES-011.
- Highlights the difference between geometric placement (RES-001) and path-based placement.
