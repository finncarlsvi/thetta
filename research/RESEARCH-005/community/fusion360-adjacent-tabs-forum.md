# How to Position Tabs for Cutting Out Adjacent Pieces -- Autodesk Forum

> **URL**: https://forums.autodesk.com/t5/fusion-manufacture-forum/how-to-position-tabs-for-cutting-out-adjacent-pieces/td-p/12566903
> **Type**: Community
> **Agents**: Claude, Codex, Gemini
> **Relevance**: High
> **Confidence**: High -- all agents identify the same core problem

## Summary
This Autodesk forum discussion directly addresses the core problem that motivates Thetta's shared-tab feature: when CNC-cutting adjacent nested pieces, tabs placed on one part can be destroyed when the neighboring part is cut. This is the practical manifestation of the shared-tab problem and demonstrates that even users of industry-leading CAM software (Fusion 360) struggle with it.

The root cause is that each part's tab placement is computed independently without awareness of neighbors. The toolpath generator for Part B is unaware of the safe Z-height regions (tabs) established for Part A. If tabs on adjacent parts do not line up at identical positions on their shared edge, cutting one part destroys the other part's tabs.

Current workarounds are unsatisfying: manually drawing tabs to ensure alignment, increasing part spacing so tabs fall on non-shared edges, or manual edge-by-edge path selection with reduced gap (1/4 inch minimum). No existing CAM software automatically aligns tabs between adjacent nested parts.

## Key Findings
- Tabs on shared edges are destroyed when the neighboring part is cut (all agents)
- Tabs must be placed at identical positions on both sides of a shared edge to survive (Claude, Gemini)
- No existing CAM software automatically aligns tabs between adjacent parts (all agents)
- Root cause: per-contour path generation lacking global spatial awareness (Gemini)
- Workarounds: manual tab alignment, increased spacing, edge-by-edge path selection (Claude)
- A shared-edge/tab graph model with synchronized hold regions could solve this (Codex)

## Technical Details
- **Platform**: Autodesk Fusion 360 Manufacture Forum
- **Root cause**: Each part's tab placement computed independently
- **Workaround 1**: Manually draw/position tabs to align across shared edges
- **Workaround 2**: Increase spacing so tabs fall on non-shared edges
- **Workaround 3**: Manual edge selection with 1/4 inch gap
- **Failure mode**: Tool for Part B mills through tab meant for Part A

## Strengths
- Directly describes the shared-tab problem Thetta aims to solve
- Real user experiences validate the importance of this feature
- Demonstrates this is an unsolved problem in commercial CAM software
- Multiple workarounds provide insight into user needs

## Weaknesses & Limitations
- Forum content is anecdotal
- Limited to Fusion 360 context
- Workarounds are manual and do not scale to large nesting layouts

## Consensus & Disagreements
All agents agree this is a critical resource for understanding the shared-tab problem. Claude considers it "arguably the most important resource for Thetta's feature design." Gemini frames the root cause as Z-height unawareness. Codex proposes a graph-model solution. No factual disagreements.

## Connections
- [RES-006] Fusion 360 Tabs Reference -- the feature that fails to solve this problem
- [RES-019] WOODWEB Tabs vs Onion Skinning -- same problem from woodworking perspective
- [RES-018] HOMAG Nesting Features -- HOMAG's integrated approach addresses this
- [RES-002] Common-Edge Nested Cutting -- academic treatment of common-edge optimization
- [RES-024] The Fabricator Common Line -- industry perspective on microtabs with common-line cutting
