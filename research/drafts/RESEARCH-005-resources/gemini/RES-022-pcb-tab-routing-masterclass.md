# PCB Tab Routing Masterclass
> **URL**: https://www.allpcb.com/blog/pcb-design/tab-routing-masterclass-optimizing-breakaway-tabs-for-clean-pcb-separation.html
> **Type**: Community
> **Agent**: Gemini
> **Relevance**: Medium
> **Credibility**: High

## Summary
This technical blog post from AllPCB details the highly specialized world of tab routing in Printed Circuit Board (PCB) manufacturing. While PCB routing is different from wood or metal CNC nesting, the underlying principles of securing small parts within a larger panel are identical.

The standard in PCB manufacturing is the use of "mouse bites"—small tabs perforated with a series of tiny drill holes. This ensures that when the board is snapped out of the panel, it breaks cleanly along the perforations rather than splintering the fiberglass (FR-4) substrate.

The article provides strict quantitative heuristics for placement: one tab every 50-75mm, with 3-5 tabs per side for standard boards. Crucially, it dictates that tabs must be kept away from high-stress areas (like heavy components) and corners to prevent delamination during the breakout process.

## Key Findings
- PCB manufacturing utilizes perforated tabs ("mouse bites") to control the exact break line and minimize stress on the part.
- Quantitative spacing heuristic: one tab every 50-75mm of perimeter length.
- Tabs should be explicitly kept away from corners to prevent structural damage during removal.

## Technical Details
Typical mouse bite design involves 5-7 holes per tab, with hole diameters of 0.5-1.0mm spaced 1.0mm apart. The tab itself is usually 0.2-0.5mm wide. The material strength (FR-4 tensile strength ~310 MPa) dictates these highly specific parameters.

## Strengths
- Provides highly specific, quantitative rules for tab placement and spacing.
- Introduces the concept of perforated tabs to control the break point, a concept that could be adapted to standard CNC routing (e.g., drilling a relief hole in a thick tab).

## Weaknesses & Limitations
- Highly specific to fiberglass/FR-4 material and manual snap-out removal.
- The scale is much smaller than typical CNC router jobs.

## Connections
- The mouse bite concept is further detailed in the Altium documentation (RES-023).
- The spacing heuristic (every 50-75mm) is a concrete example of the "By Distance" method seen in Fusion 360 (RES-006).
