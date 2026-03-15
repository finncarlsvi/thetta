# FreeCAD PathDressupTag.py Source Code
> **URL**: https://github.com/FreeCAD/FreeCAD/commits/master/src/Mod/Path/PathScripts/PathDressupTag.py
> **Type**: Code
> **Agent**: Codex
> **Relevance**: High
> **Credibility**: High

## Summary
This source-focused resource provides implementation clues for FreeCAD’s tag generation internals, including geometry extraction and edge-length-based tab distribution. It is a critical bridge from high-level documentation to executable heuristics.

The notes point to functions like `findBottomWire()` and `generateTags()`, with automatic tag allocation weighted by edge lengths and seeded from longer edges. That strategy is practical and computationally cheap, but the source commentary also indicates fragility.

The maintainer remarks about the feature being “touchy” and needing rewrite are valuable evidence for risk. They suggest that robust automatic tag generation is non-trivial, especially when reconstructing geometry from imported G-code rather than clean CAD contours.

## Key Findings
- Automatic tag distribution appears to allocate counts by edge length using an edge dictionary and longest-edge-first logic.
- Maintainer commentary flags significant complexity and reliability issues in automatic generation workflows.

## Technical Details
The pipeline indicated in cache: identify bottom contour wire, verify closure, derive edge set, distribute tag counts by relative edge length, then emit modified path segments. Complexity increases when geometry must be reconstructed from less-structured inputs, which likely drives many edge-case failures.

## Strengths
- Offers direct insight into concrete open-source algorithm choices and known pain points.

## Weaknesses & Limitations
- Cached view is partial; full current source and commit context were not fully mirrored locally.

## Connections
- [RES-011-freecad-dressuptag.md](/Users/finn.carlsvi/thetta/research/drafts/RESEARCH-005-resources/codex/RES-011-freecad-dressuptag.md) — complements user-facing FreeCAD documentation with internals and maintainability signals.
