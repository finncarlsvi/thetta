# SendCutSend CNC Routing Tabs
> **URL**: https://sendcutsend.com/services/cnc-routing/tabs/
> **Type**: Commercial
> **Agent**: Gemini
> **Relevance**: Medium
> **Credibility**: High

## Summary
SendCutSend is a massive, highly automated online manufacturing service. Their documentation on CNC Routing Tabs provides a fascinating look at how tabs are handled in a zero-touch, fully automated commercial environment where user intervention is impossible.

According to their documentation, all routed parts have small fixturing tabs placed automatically based on part geometry. Because they operate a highly automated pipeline, they explicitly state that custom tab placement is not available to the user. The algorithm makes the decisions.

To ensure quality, they design tabs to be as small as physically possible for the material. Furthermore, all tabs (and lead-in/lead-out moves) are placed entirely outside the part geometry. This means the finished part requires tab removal, but the underlying dimension of the part is never undercut or compromised by the tab generation process.

## Key Findings
- Fully automated, geometry-based tab placement is viable in commercial, zero-touch manufacturing environments.
- To maintain automated scale, user customization of tabs must sometimes be restricted.
- Ensuring tabs are generated strictly "outside" the part geometry is crucial for maintaining dimensional accuracy.

## Technical Details
While the exact algorithm is proprietary, the pipeline must analyze the 2D DXF/STEP geometry, calculate the optimal bounding box or perimeter locations for stability, and inject the tab G-code on the exterior of the final offset path.

## Strengths
- Proves that fully automated tab placement without user intervention is commercially viable at scale.
- Highlights the importance of maintaining part geometry integrity.

## Weaknesses & Limitations
- As a black-box service, no details of the actual placement algorithm are revealed.

## Connections
- Contrasts heavily with the manual-override philosophy seen in traditional CAM software like VCarve (RES-009) and FreeCAD (RES-011).
