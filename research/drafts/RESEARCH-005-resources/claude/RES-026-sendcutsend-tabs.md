# SendCutSend CNC Routing Tabs Documentation
> **URL**: https://sendcutsend.com/services/cnc-routing/tabs/
> **Type**: Commercial
> **Agent**: Claude
> **Relevance**: Medium
> **Credibility**: Medium

## Summary
SendCutSend is an online CNC manufacturing service that provides on-demand cutting, routing, and bending. Their tabs documentation describes a fully automatic tab placement system where tabs are placed based on part geometry with no option for custom placement. This represents one end of the automation spectrum — a completely opaque, service-provider-controlled tab system.

The key design principle is that all tabs and lead-in/lead-out points are placed outside the part geometry, ensuring the design remains true to the original dimensions. Tabs are designed to be as small as possible for easy removal, reflecting a customer-experience priority (minimize post-processing effort). The fully automatic placement based on part geometry suggests an algorithm that considers factors like perimeter length, narrow features, and part stability, though no details are disclosed.

This resource is relevant to Thetta as an example of a production-grade automatic tab placement system. The fact that a manufacturing service processes thousands of parts with fully automatic tab placement validates that automatic algorithms can work reliably at scale. The "outside part geometry" constraint is an important design decision — it means tab material extends beyond the part boundary rather than creating a bump on the part edge, which simplifies post-processing but requires additional stock material.

## Key Findings
- Fully automatic tab placement based on part geometry — no custom placement available
- All tabs and lead-ins are outside part geometry (design integrity preserved)
- Tabs designed to be as small as possible for easy removal
- Production service processes many parts with automatic placement — validates algorithmic feasibility
- No user control over tab count, size, or position

## Technical Details
- **Service**: SendCutSend (online CNC manufacturing)
- **Tab placement**: Fully automatic, geometry-based
- **Tab position**: Outside part geometry
- **Tab size**: Minimized for easy removal
- **User control**: None — all tab decisions are automatic
- **Lead-in/out**: Also automatic, outside part geometry

## Strengths
- Validates that fully automatic tab placement works at production scale
- "Outside part geometry" principle ensures design integrity
- Small-as-possible tabs reflect good UX for post-processing
- Simple, opinionated approach reduces user complexity

## Weaknesses & Limitations
- Zero details on the placement algorithm
- No user customization — inappropriate for applications needing specific tab locations
- Production service context may not generalize (they control material, tooling, process)
- No information on how they handle adjacent parts or shared edges
- Commercial marketing page, not technical documentation

## Connections
- [RES-010 CamBam Holding Tabs] — CamBam provides user control that SendCutSend eliminates
- [RES-006 Fusion 360 Tabs Reference] — Fusion offers both automatic and manual, contrasting with SendCutSend's auto-only
- [RES-018 HOMAG Nesting Features] — HOMAG's industrial automatic approach with more configurability
