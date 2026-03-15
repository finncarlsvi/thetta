# Tabbing Versus Onion-Skinning for Small Parts -- WOODWEB

> **URL**: https://woodweb.com/knowledge_base/Tabbing_Versus_OnionSkinning_for_Small_Parts.html
> **Type**: Community
> **Agents**: Claude, Codex, Gemini
> **Relevance**: Medium
> **Confidence**: High -- agents agree on practical heuristics

## Summary
This WOODWEB Knowledge Base article provides a practical comparison of tabbing and onion-skinning strategies specifically for small parts in CNC woodworking. Small parts present a unique workholding challenge because they have insufficient surface area for vacuum hold-down once separated from the stock sheet.

For tabs, thick tabs (such as 1/4 inch) "totally lock everything down" but require more sanding. For onion skinning, a two-stage approach is described: cut to 0.70 inches in a 0.75 inch sheet (leaving 0.05 inch skin), then make a full-depth pass. The article includes valuable practical tips: reduce effective table size by covering unused spoilboard with cardboard to concentrate vacuum, ensure sheets lie flat, cut small parts first when vacuum is strongest, and employ two-pass methods.

## Key Findings
- Thick tabs (1/4 inch) provide very strong hold-down but require more sanding (all agents)
- Onion skinning: cut to 0.70 inch in 0.75 inch sheet, then full depth for final pass (Claude, Gemini)
- Multi-pass cutting reduces the force applied to tabs/skins during final separation (Gemini)
- Cover unused spoilboard with cardboard to concentrate vacuum (Claude)
- Cut small parts first while vacuum is strongest (Claude)
- Use compression bits for large parts, down-shear bits for small parts (Claude)
- The necessity of tabs is inversely proportional to vacuum effectiveness and part size (Gemini)

## Technical Details
- **Context**: CNC woodworking, small part workholding
- **Tab sizing**: 1/4 inch thick for strong hold in typical 3/4 inch sheet
- **Onion skin depth**: Leave ~0.05 inch (0.70 inch cut in 0.75 inch sheet)
- **Physics**: Balancing lateral cutting force against shear strength of tab and vacuum friction (Gemini)

## Strengths
- Practical, experience-based guidance on tab sizing relative to material thickness
- Vacuum optimization tips applicable regardless of tab strategy
- Specific dimensional recommendations

## Weaknesses & Limitations
- Woodworking-specific -- tab sizing may not apply to other materials
- Anecdotal practitioner knowledge
- Does not address tab placement (position along contour)
- No coverage of shared tabs

## Consensus & Disagreements
All agents agree on the tabs vs. onion skinning trade-off and multi-pass strategy. Claude provides the most operational tips (spoilboard coverage, bit selection). Gemini provides the physics framing (lateral force vs. shear strength). No factual disagreements.

## Connections
- [RES-019] WOODWEB Tabs vs Onion Skinning -- companion discussion
- [RES-021] CNCZone Optimize Tabs -- adds tab placement heuristics
- [RES-028] Maslow Tabs Strategy -- similar material-relative sizing
- [RES-018] HOMAG Nesting Features -- industrial solution combining tabs and onion skinning
