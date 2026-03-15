# SendCutSend CNC Routing Tabs Documentation

> **URL**: https://sendcutsend.com/services/cnc-routing/tabs/
> **Type**: Commercial
> **Agents**: Claude, Codex, Gemini
> **Relevance**: Medium
> **Confidence**: High -- agents agree on auto-only approach

## Summary
SendCutSend is an online CNC manufacturing service that provides on-demand cutting, routing, and bending. Their tabs documentation describes a fully automatic tab placement system where tabs are placed based on part geometry with no option for custom placement. This represents one end of the automation spectrum -- a completely opaque, service-provider-controlled tab system.

The key design principle is that all tabs and lead-in/lead-out points are placed outside the part geometry, ensuring the design remains true to the original dimensions. Tabs are designed to be as small as possible for easy removal. The fact that a manufacturing service processes thousands of parts with fully automatic tab placement validates that automatic algorithms can work reliably at scale.

## Key Findings
- Fully automatic tab placement based on part geometry -- no custom placement available (all agents)
- All tabs and lead-ins are outside part geometry (design integrity preserved) (all agents)
- Tabs designed to be as small as possible for easy removal (Claude, Gemini)
- Production service validates algorithmic feasibility at scale (Claude, Gemini)
- "Outside" placement prevents accidental undercutting of critical dimensions (Gemini)

## Technical Details
- **Service**: SendCutSend (online CNC manufacturing)
- **Tab placement**: Fully automatic, geometry-based
- **Tab position**: Outside part geometry
- **Tab size**: Minimized for easy removal
- **User control**: None -- all tab decisions are automatic

## Strengths
- Validates that fully automatic tab placement works at production scale
- "Outside part geometry" principle ensures design integrity
- Simple, opinionated approach reduces user complexity

## Weaknesses & Limitations
- Zero details on the placement algorithm
- No user customization -- inappropriate for applications needing specific tab locations
- Production service context may not generalize (they control material, tooling, process)
- No information on shared edges/adjacent parts

## Consensus & Disagreements
All agents agree on the auto-only approach and outside-geometry placement. Gemini provides the strongest contrast with "manual-override philosophy" of traditional CAM software. No factual disagreements.

## Connections
- [RES-010] CamBam Holding Tabs -- CamBam provides user control that SendCutSend eliminates
- [RES-006] Fusion 360 Tabs Reference -- offers both automatic and manual
- [RES-018] HOMAG Nesting Features -- industrial automatic approach with more configurability
