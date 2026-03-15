# PCB Panelization: Mouse Bites and V-Scores -- Altium

> **URL**: https://resources.altium.com/p/mouse-bites-and-v-scores-how-depanelize-pcbs
> **Type**: Community
> **Agents**: Claude, Codex, Gemini
> **Relevance**: Low
> **Confidence**: High -- agents agree on low relevance

## Summary
This Altium resource covers two PCB depanelization techniques: mouse bites (perforated holes along tab boundaries) and V-scores (narrow grooves cut into both sides of the panel). While primarily a PCB manufacturing resource, the underlying principles of controlled material separation are conceptually related to CNC tab design.

Mouse bites use small cutout regions formed by rows of drilled holes placed tangent to the PCB edge. Key design rules: place holes tangent to the edge (minimizing board intrusion), use large diameter holes for cleaner separation, and maintain hole-wall spacing less than hole diameter for reliable fracture. V-scores are grooves on both sides requiring large clearance, strictly limited to straight lines traversing the entire panel.

## Key Findings
- Mouse bites: holes tangent to PCB edge, large diameter, spacing < hole diameter (Claude, Gemini)
- V-scores: dual-side grooves, strictly limited to straight lines (Gemini)
- Mouse bite routing area: 1.5-2mm around holes (Claude)
- Tangential placement keeps broken edges flush with final boundary (Gemini)

## Technical Details
- **Domain**: PCB panelization / depanelization
- **Mouse bites**: Drilled holes tangent to board edge, 1.5-2mm routing area
- **V-scores**: Dual-side grooves with controlled remaining thickness
- **Design rules**: Hole tangency, spacing < diameter, copper clearance for V-scores

## Strengths
- Authoritative source (Altium is a major PCB EDA vendor)
- Clear design rules for controlled material separation

## Weaknesses & Limitations
- PCB-specific -- not directly applicable to CNC wood/metal routing
- No relevance to tab placement algorithms or positioning heuristics
- Panel grid layout differs from irregular nesting

## Consensus & Disagreements
All agents agree on Low relevance. No factual disagreements.

## Connections
- [RES-022] PCB Tab Routing Masterclass -- complementary PCB tab resource
- [RES-024] The Fabricator Common Line -- CNC equivalent of shared-edge separation
