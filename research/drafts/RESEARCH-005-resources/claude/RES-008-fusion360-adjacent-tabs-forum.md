# How to Position Tabs for Cutting Out Adjacent Pieces — Autodesk Forum
> **URL**: https://forums.autodesk.com/t5/fusion-manufacture-forum/how-to-position-tabs-for-cutting-out-adjacent-pieces/td-p/12566903
> **Type**: Community
> **Agent**: Claude
> **Relevance**: High
> **Credibility**: Medium

## Summary
This Autodesk forum discussion directly addresses the core problem that motivates Thetta's shared-tab feature: when CNC-cutting adjacent nested pieces, tabs placed on one part can be destroyed when the neighboring part is cut. This is the practical manifestation of the shared-tab problem and demonstrates that even users of industry-leading CAM software (Fusion 360) struggle with this issue.

The forum poster identifies the key failure mode: if tabs on adjacent parts don't line up in the exact same location along their shared edge, cutting one part destroys the other part's tabs. For example, if Part A has a tab at position X on the shared edge, and Part B has no tab at the corresponding position on its side of the edge, cutting Part B will remove Part A's tab material. The only way tabs survive is if both parts have tabs at identical positions on the shared edge, creating a single shared tab that holds both parts.

The workarounds discussed are unsatisfying: manually drawing tabs to ensure alignment, or spacing parts farther apart so the CAM software places tabs on non-shared edges. One user mentions a manual edge selection approach where they select specific edges for the cutter to follow, allowing them to maintain a 1/4" gap while controlling tab placement. These workarounds highlight a clear gap in current CAM software — no tool automatically aligns tabs between adjacent nested parts.

This forum discussion is arguably the most important resource for Thetta's feature design because it articulates the exact user pain point and confirms that no existing solution automates shared-tab alignment.

## Key Findings
- Tabs on shared edges between adjacent parts are destroyed when the neighboring part is cut
- Tabs must be placed at identical positions on both sides of a shared edge to survive
- No existing CAM software automatically aligns tabs between adjacent parts
- Current workarounds: manual tab placement, increased part spacing, or edge-by-edge path selection
- The 1/4" gap between parts is a practical minimum for some workaround approaches

## Technical Details
- **Platform**: Autodesk Fusion 360 Manufacture Forum
- **Problem**: Adjacent nested parts with automatic tab placement — tabs are incompatible across shared edges
- **Root cause**: Each part's tab placement is computed independently without awareness of neighbors
- **Workaround 1**: Manually draw/position tabs to align across shared edges
- **Workaround 2**: Increase spacing between parts so tabs fall on non-shared edges
- **Workaround 3**: Manual edge selection for toolpath with reduced gap (1/4")

## Strengths
- Directly describes the shared-tab problem Thetta aims to solve
- Real user experiences validate the importance of this feature
- Demonstrates that this is an unsolved problem in commercial CAM software
- Multiple workarounds provide insight into user needs and constraints

## Weaknesses & Limitations
- Forum content is anecdotal — no quantitative analysis
- Limited to Fusion 360 context; other CAM tools may handle this differently
- Full thread content not available (FETCH_FAILED, 403)
- Workarounds are manual and don't scale to large nesting layouts

## Connections
- [RES-006 Fusion 360 Tabs Reference] — the feature set that fails to solve this problem
- [RES-019 WOODWEB Tabs vs Onion Skinning] — same shared-tab alignment problem from woodworking perspective
- [RES-018 HOMAG Nesting Features] — HOMAG's integrated approach may partially address this
- [RES-002 Common-Edge Nested Cutting] — academic treatment of the common-edge optimization problem
- [RES-024 The Fabricator Common Line] — industry perspective on microtabs with common-line cutting
