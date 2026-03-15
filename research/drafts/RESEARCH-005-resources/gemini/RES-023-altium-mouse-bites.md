# PCB Mouse Bites and V-Scores
> **URL**: https://resources.altium.com/p/mouse-bites-and-v-scores-how-depanelize-pcbs
> **Type**: Community
> **Agent**: Gemini
> **Relevance**: Low
> **Credibility**: High

## Summary
Altium's documentation covers the two primary methods for depanelizing PCBs: Mouse Bites (perforated tabs) and V-Scores (continuous shallow grooves). This provides context on how a highly automated, high-precision industry handles the problem of separating nested parts.

For mouse bites, Altium specifies design rules: holes must be placed tangent to the PCB edge, and the spacing between hole walls must be less than the hole diameter to ensure a clean break. A routing area of 1.5-2mm is required around the holes to allow the router bit to define the tab shape.

V-scoring is presented as an alternative to tabs for straight-line separations, where a rolling blade cuts a shallow groove on both sides of the board. While highly efficient for square boards, it is strictly limited to straight lines that traverse the entire panel.

## Key Findings
- Tangential placement of perforations on a tab ensures that the broken edge remains flush with or slightly recessed from the final part boundary.
- V-scoring serves as a non-tab alternative, but is severely limited by geometric constraints (straight lines only).

## Technical Details
Design rules for mouse bites are stringent to prevent board delamination. The tangential positioning minimizes intrusion into the board area. For V-scores, designers must specify score angle, depth, and the remaining web thickness.

## Strengths
- Clear explanation of specialized PCB tab geometry.
- Highlights the geometric limitations of non-tab separation methods (V-scores).

## Weaknesses & Limitations
- Very low relevance to general CNC routing of irregular shapes.
- Focuses more on panel design than automatic placement algorithms.

## Connections
- A direct companion to the PCB tab routing masterclass in RES-022.
