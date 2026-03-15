# Fusion 360 Tabs Tutorial -- CAD CAM Lessons

> **URL**: https://cadcamlessons.com/fusion-360-tabs/
> **Type**: Community
> **Agents**: Claude, Codex, Gemini
> **Relevance**: Medium
> **Confidence**: High -- consistent with official documentation

## Summary
This tutorial from CAD CAM Lessons provides a step-by-step guide to configuring tabs in Fusion 360's 2D contour operations. It complements Autodesk's official reference (RES-006) with a more practical, hands-on walkthrough. The content largely mirrors the official documentation but with added context on the physical mechanism: during machining, the cutting tool deliberately raises slightly at tab locations, leaving thin material layers that connect the part to the stock.

Gemini uniquely breaks down the G-code implementation: for a rectangular tab, the tool retracts to the tab height, moves laterally for the tab width, and plunges back. For a triangular tab, it performs an angled ramp up and ramp down.

## Key Findings
- Tabs work by raising the cutting tool at specified locations, leaving thin material layers (all agents)
- Two shapes: Rectangular (consistent support) and Triangular (easier removal) (all agents)
- Three core parameters: Tab Width, Tab Height, Tab Distance (all agents)
- G-code: rectangular = retract/traverse/plunge; triangular = angled ramp up/down (Gemini)
- Users expect immediate geometric feedback when tab parameters change (Codex)

## Technical Details
- **Software**: Fusion 360 (Autodesk)
- **Operation**: 2D Contour -> Tabs section
- **Tab mechanism**: Tool raises at tab location during final pass
- **G-code**: Rectangular tabs use discrete Z moves; triangular tabs use coordinated XZ moves

## Strengths
- Accessible explanation of tab concepts for newcomers
- Confirms the standard parameter model across commercial CAM tools
- Gemini's G-code breakdown clarifies physical implementation

## Weaknesses & Limitations
- Largely duplicates official documentation (RES-006)
- No algorithmic details on automatic placement
- No coverage of nested-part scenarios

## Consensus & Disagreements
All agents are aligned. This is supplementary to RES-006 with no novel algorithmic insights. Gemini provides the unique G-code-level detail.

## Connections
- [RES-006] Fusion 360 Tabs Reference -- official version of the same content
- [RES-008] Fusion 360 Adjacent Tabs Forum -- where this approach breaks down (adjacent parts)
- [RES-005] MecSoft Bridges & Tabs -- same framework from a different vendor
