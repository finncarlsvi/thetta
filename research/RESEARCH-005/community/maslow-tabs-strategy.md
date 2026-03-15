# Maslow CNC Tabs Strategy Discussion

> **URL**: https://forums.maslowcnc.com/t/help-with-tabs-strategy/432
> **Type**: Community
> **Agents**: Claude, Codex, Gemini
> **Relevance**: Low
> **Confidence**: Medium -- anecdotal forum content

## Summary
This Maslow CNC forum discussion covers practical tab sizing and cutting strategies for an affordable open-source large-format CNC router. The Maslow's vertical cutting orientation makes tab strategy particularly important, as parts are more prone to movement due to gravity.

The tab sizing recommendations are ratio-based: for 0.500 inch material, use 0.255 inch thick tabs (approximately 50% of material thickness), with tab width equal to 1.5x the depth value. This material-relative sizing is a useful heuristic. The multi-pass cutting strategy is detailed: first pass cuts with normal step-downs stopping short of full depth, second pass is an identical overlay cutting completely through except at tab locations, and the final pass cuts the remaining thin tabs from bottom to top.

## Key Findings
- Tab thickness: ~50% of material thickness (0.255 inch for 0.500 inch material) (Claude, Gemini)
- Tab width = 1.5x tab depth (Claude, Gemini)
- Multi-pass strategy: rough pass -> through-cut except tabs -> final tab cut (all agents)
- Tab size should scale with material thickness (all agents)
- Vertical CNC orientation makes tabs especially critical due to gravity (Gemini)
- Removal tools: oscillating multi-tools, chisels, flush-cut saws, Dremels, scissors (Claude)

## Technical Details
- **Platform**: Maslow CNC (open-source, vertical large-format router)
- **Tab thickness ratio**: ~50% of material thickness
- **Tab width ratio**: 1.5x depth value
- **Example**: 0.255 inch thick x ~0.38 inch wide for 0.500 inch material
- **Cutting strategy**: 3-pass (rough -> through except tabs -> tab removal pass)

## Strengths
- Material-relative sizing heuristic (50% thickness) is directly implementable
- Width-to-depth ratio (1.5x) provides structural cross-section guideline
- Multi-pass strategy minimizes tab stress during cutting

## Weaknesses & Limitations
- Maslow-specific concerns (vertical orientation, low rigidity) may not generalize
- Small sample of maker community opinions
- No discussion of tab placement (position along contour)
- Limited to wood/plywood materials

## Consensus & Disagreements
All agents agree on material-proportional sizing and multi-pass strategy. Claude provides the 50% and 1.5x ratios explicitly. Gemini uniquely emphasizes the gravity factor from vertical orientation. Codex describes two passes where Claude describes three -- a minor framing difference. No substantive factual disagreements.

## Connections
- [RES-021] CNCZone Optimize Tabs -- CNCZone's 0.75 x 0.125 sizing for comparison
- [RES-020] WOODWEB Tabbing Small Parts -- 1/4 inch thick tab for 3/4 inch sheet (~33% ratio)
- [RES-025] Mate Tabbing Practices -- material/force-dependent sizing principle
