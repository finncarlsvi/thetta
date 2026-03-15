# PCB Panelization: Mouse Bites and V-Scores — Altium
> **URL**: https://resources.altium.com/p/mouse-bites-and-v-scores-how-depanelize-pcbs
> **Type**: Community
> **Agent**: Codex
> **Relevance**: Low
> **Credibility**: Medium

## Summary
Altium’s article explains PCB depanelization options, primarily mouse bites and V-scores, with design-rule guidance for manufacturability and downstream assembly. It is tangential to CNC routing tabs but useful as a structured treatment of breakaway interfaces.

Mouse-bite recommendations in the source focus on hole placement relative to edges and spacing relationships that balance detachability against structural integrity. V-score guidance emphasizes keepout from copper and components, reinforcing the broader principle of protecting critical regions near separation features.

For this research, the relevance is lower because geometry, tools, and materials differ substantially from profile-cut nesting. Still, the article contributes transferable ideas around rule-based constraints and documentation of separation geometry in production workflows.

## Key Findings
- Mouse-bite quality depends on hole geometry and tangent placement relative to board edges plus adequate routed clearance.
- V-score design requires explicit clearance/keepout management and documented scoring parameters.

## Technical Details
The resource frames breakaway features as DFM constraints integrated with panel design (tooling holes, fiducials, coupons). Analogously, CNC tab planners can treat tab placement as a constrained design layer coupled with manufacturing setup metadata.

## Strengths
- Clear design-rule framing with manufacturability context.

## Weaknesses & Limitations
- Low direct applicability to nested CNC tab sharing beyond high-level principles.

## Connections
- [RES-022-pcb-tab-routing-masterclass.md](/Users/finn.carlsvi/thetta/research/drafts/RESEARCH-005-resources/codex/RES-022-pcb-tab-routing-masterclass.md) — both describe PCB breakaway-tab geometry and spacing considerations.
