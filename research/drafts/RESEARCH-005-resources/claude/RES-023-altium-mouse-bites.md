# PCB Panelization: Mouse Bites and V-Scores — Altium
> **URL**: https://resources.altium.com/p/mouse-bites-and-v-scores-how-depanelize-pcbs
> **Type**: Community
> **Agent**: Claude
> **Relevance**: Low
> **Credibility**: High

## Summary
This Altium resource covers two PCB depanelization techniques: mouse bites and V-scores. While this is primarily a PCB manufacturing resource, the underlying principles of controlled material separation along predefined boundaries are conceptually related to CNC tab design. The article provides design rules for both approaches to help PCB designers choose the appropriate method and configure it correctly.

Mouse bites are small cutout regions formed by rows of drilled holes placed tangent to the PCB edge. When the panel is broken apart, the material between the holes fractures, leaving small bumps that can be filed smooth. Key design rules: place holes tangent to the PCB edge (minimizing board intrusion), use large diameter holes for cleaner separation, and maintain hole-wall spacing less than hole diameter for reliable fracture. The routing area around mouse bites should be 1.5-2mm.

V-scores are narrow grooves cut into both sides of the panel at the separation line. The remaining material in the center is thin enough to break by hand or with a rolling blade. V-scores require large clearance from nearby copper features and no components near the score regions. Design documentation must specify score angle, depth, and remaining material thickness.

Panel design guidance includes verifying fabricator material stock in the required panel size, adding tooling holes and fiducials to panel corners, and including test coupons for impedance testing.

## Key Findings
- Mouse bites: holes tangent to PCB edge, large diameter, hole spacing < hole diameter
- V-scores: grooves on both sides, requires large clearance from copper
- Mouse bite routing area: 1.5-2mm around holes
- Tangential hole positioning minimizes board intrusion
- Panel design: tooling holes and fiducials in corners

## Technical Details
- **Domain**: PCB panelization / depanelization
- **Mouse bites**: Drilled holes tangent to board edge, 1.5-2mm routing area
- **V-scores**: Dual-side grooves with controlled remaining thickness
- **Design rules**: Hole tangency, spacing < diameter, copper clearance for V-scores
- **Source**: Altium (professional PCB EDA vendor)

## Strengths
- Authoritative source (Altium is a major PCB EDA vendor)
- Clear design rules for controlled material separation
- Tangential hole placement principle could inspire tab geometry optimization
- V-score concept (dual-side weakening) is conceptually interesting for tab design

## Weaknesses & Limitations
- PCB-specific — not directly applicable to CNC wood/metal routing
- Mouse bite and V-score techniques don't translate to CNC tab placement
- No relevance to tab placement algorithms or positioning heuristics
- Panel grid layout differs fundamentally from irregular nesting

## Connections
- [RES-022 PCB Tab Routing Masterclass] — complementary PCB tab resource with more placement detail
- [RES-024 The Fabricator Common Line] — CNC equivalent of shared-edge separation
