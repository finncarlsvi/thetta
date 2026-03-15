# MecSoft Bridges & Tabs Explored
> **URL**: https://mecsoft.com/blog/bridges-tabs-explored/
> **Type**: Commercial
> **Agent**: Gemini
> **Relevance**: High
> **Credibility**: High

## Summary
This commercial documentation from MecSoft (creators of VisualCAD/CAM and RhinoCAM) outlines how their software handles holding tabs, which they refer to as "bridges." It provides a clear view into industry-standard CAM features for tab placement.

MecSoft supports two primary shapes for bridges: Rectangular and Triangular. Rectangular tabs provide consistent hold but require more post-processing, while triangular tabs taper up and down, making them easier to break and clean up. The software allows these to be configured via Bridge Height and Bridge Length parameters.

Crucially for this research, MecSoft offers both automatic and manual placement modes. In automatic mode, the software distributes tabs based on either a specified "Number of Bridges" or a "Distance between Bridges." This highlights the two dominant heuristics in commercial CAM: count-based distribution and distance-based distribution.

## Key Findings
- The two standard tab shapes in commercial CAM are Rectangular and Triangular.
- Automatic placement algorithms generally rely on two simple rules: a fixed count distributed evenly, or a fixed distance interval along the perimeter.
- Manual override is considered essential; operators need the ability to add, edit, or delete automatically placed bridge points along a curve region.

## Technical Details
MecSoft's implementation places bridge configuration within the "Advanced Cut Parameters" of their 2½ Axis Profiling operation. The system allows users to define Bridge Height, Bridge Length, and choose between specifying the exact Number of Bridges or the Distance between them.

## Strengths
- Provides a clear, practical look at how top-tier commercial CAM software exposes tab configuration to users.
- Highlights the necessity of both automatic generation and manual editing capabilities.

## Weaknesses & Limitations
- As product documentation, it doesn't reveal the underlying code for how tabs are distributed, especially on complex curves or internal corners.
- Doesn't address the shared-tab problem for adjacent nested parts.

## Connections
- The feature set (Rectangular vs Triangular, By Distance vs By Count) is nearly identical to Fusion 360's offerings (RES-006, RES-007).
