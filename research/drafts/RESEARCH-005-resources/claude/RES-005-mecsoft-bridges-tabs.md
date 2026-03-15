# Bridges & Tabs Explored — MecSoft Corporation
> **URL**: https://mecsoft.com/blog/bridges-tabs-explored/
> **Type**: Commercial
> **Agent**: Claude
> **Relevance**: High
> **Credibility**: High

## Summary
This blog post by Don LaCourse (20+ years CAD/CAM experience) from MecSoft Corporation provides a clear overview of bridge/tab functionality in their RhinoCAM product. Bridges and tabs are defined as areas where stock material is intentionally left to connect the part to the surrounding stock, preventing movement during machining. Once machining is complete, bridges are manually removed by cutting, grinding, or sanding.

MecSoft's implementation in the Mill module offers tabs through the Advanced Cut Parameters tab of the 2½ Axis Profiling operation dialog. Two bridge types are supported: Triangular (tapered, less material to remove post-machining) and Rectangular (straight-sided, provides more consistent support). Key parameters include Bridge Height, Bridge Length, and either Number of Bridges or Distance between Bridges — the user can choose count-based or spacing-based placement.

The software supports both automatic and manual placement modes. In automatic mode, tabs are distributed evenly along the toolpath when the operation is generated. In manual mode, the user places Bridge Points on a Curve Region using Add/Edit/Delete controls for precise positioning. This dual-mode approach is the industry standard pattern seen across commercial CAM tools and provides a good design reference for Thetta.

## Key Findings
- Two bridge shapes: Triangular (easier removal) and Rectangular (stronger hold)
- Placement by count OR by distance — two distinct automatic distribution methods
- Parameters: Bridge Height, Bridge Length, Number/Distance
- Automatic mode distributes evenly along toolpath; Manual mode allows precise point placement
- Bridges defined in 2½ Axis Profiling operation context

## Technical Details
- **Software**: RhinoCAM (MecSoft's CAM plugin for Rhino)
- **Operation**: 2½ Axis Profiling → Advanced Cut Parameters tab
- **Bridge Types**: Triangular, Rectangular
- **Placement Modes**: Automatic (by count or distance), Manual (point-based with Add/Edit/Delete)
- **Parameters**: Height, Length, Count/Distance
- **Author**: Don LaCourse, 20+ years CAD/CAM experience
- **Published**: July 10, 2017

## Strengths
- Clear explanation of bridge/tab concepts from experienced CAM professional
- Defines the standard parameter set (height, length, count/distance) used across the industry
- Dual automatic/manual placement is a well-proven UX pattern
- Triangular vs. rectangular shape options address different use cases

## Weaknesses & Limitations
- No algorithmic details on how automatic placement distributes tabs
- Doesn't address tab placement for nested/adjacent parts
- Commercial product documentation — implementation details are proprietary
- Page is JS-heavy, full content not fully captured in cache

## Connections
- [RES-006 Fusion 360 Tabs Reference] — same parameter pattern (shape, height, width, distance)
- [RES-027 Mastercam Tabs] — Mastercam offers the same count-vs-distance automatic placement
- [RES-010 CamBam Holding Tabs] — CamBam adds min/max clamp and size threshold to the same model
- [RES-009 VCarve Tab Customization] — VCarve adds 3D tabs (smooth ramping) as a third shape option
