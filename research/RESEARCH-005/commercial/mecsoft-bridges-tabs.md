# Bridges & Tabs Explored -- MecSoft Corporation

> **URL**: https://mecsoft.com/blog/bridges-tabs-explored/
> **Type**: Commercial
> **Agents**: Claude, Codex, Gemini
> **Relevance**: High
> **Confidence**: High -- all agents agree on feature set

## Summary
This blog post by Don LaCourse (20+ years CAD/CAM experience) from MecSoft Corporation provides a clear overview of bridge/tab functionality in their RhinoCAM product. Bridges and tabs are defined as areas where stock material is intentionally left to connect the part to the surrounding stock, preventing movement during machining. Once machining is complete, bridges are manually removed.

MecSoft's implementation offers two bridge types: Triangular (tapered, less material to remove post-machining) and Rectangular (straight-sided, provides more consistent support). Key parameters include Bridge Height, Bridge Length, and either Number of Bridges or Distance between Bridges. The software supports both automatic and manual placement modes. In automatic mode, tabs are distributed evenly along the toolpath. In manual mode, the user places Bridge Points on a Curve Region using Add/Edit/Delete controls. This dual-mode approach is the industry standard pattern.

## Key Findings
- Two bridge shapes: Triangular (easier removal) and Rectangular (stronger hold) (all agents)
- Placement by count OR by distance -- two distinct automatic distribution methods (all agents)
- Parameters: Bridge Height, Bridge Length, Number/Distance (all agents)
- Automatic mode distributes evenly along toolpath; Manual mode allows precise point placement (all agents)
- Bridges defined in 2.5 Axis Profiling operation context (Claude, Gemini)

## Technical Details
- **Software**: RhinoCAM (MecSoft's CAM plugin for Rhino)
- **Operation**: 2.5 Axis Profiling -> Advanced Cut Parameters tab
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
- Does not address tab placement for nested/adjacent parts
- Commercial product documentation -- implementation details are proprietary

## Consensus & Disagreements
All agents are fully aligned. The "Count vs Distance" dichotomy is identified by Gemini as the dominant industry heuristic. No disagreements on any finding.

## Connections
- [RES-006] Fusion 360 Tabs Reference -- same parameter pattern
- [RES-027] Mastercam Tabs -- same count-vs-distance automatic placement
- [RES-010] CamBam Holding Tabs -- adds min/max clamp and size threshold
- [RES-009] VCarve Tab Customization -- adds 3D tabs as a third shape option
