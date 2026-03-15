# Tabbing Versus Onion-Skinning for Small Parts — WOODWEB
> **URL**: https://woodweb.com/knowledge_base/Tabbing_Versus_OnionSkinning_for_Small_Parts.html
> **Type**: Community
> **Agent**: Claude
> **Relevance**: Medium
> **Credibility**: Medium

## Summary
This WOODWEB Knowledge Base article provides a practical comparison of tabbing and onion-skinning strategies specifically for small parts in CNC woodworking. Small parts present a unique workholding challenge because they have insufficient surface area for vacuum hold-down once separated from the stock sheet. The discussion covers when each approach is appropriate and offers practical tips for improving vacuum performance.

For tabs, the article notes that thick tabs (such as 1/4" thick) "totally lock everything down," providing excellent holding force. However, thick tabs require more sanding to remove, which adds labor cost. The trade-off between holding force and post-processing effort is the central design tension for tab sizing.

For onion skinning, a two-stage approach is described: cut to 0.70" depth in a 0.75" sheet (leaving 0.05" skin), then make a full-depth pass to complete the cut except at tab/skin locations. Full-sheet onion skinning can work with optimization but requires careful process control.

The practical tips section is particularly valuable: reduce effective table size by covering unused spoilboard with cardboard to concentrate vacuum pressure, ensure sheets lie perfectly flat (warped material creates gaps reducing vacuum), cut small parts first when vacuum is strongest (before large cutouts reduce suction), use compression bits for large parts and down-shear bits for small parts, and employ two-pass methods to reduce chip accumulation and part movement.

## Key Findings
- Thick tabs (1/4") provide very strong hold-down but require more sanding
- Onion skinning: cut to 0.70" in 0.75" sheet, then full depth for final pass
- Cover unused spoilboard with cardboard to concentrate vacuum on work area
- Cut small parts first while vacuum is strongest
- Use compression bits for large parts, down-shear bits for small parts
- Two-pass cutting reduces chip accumulation and movement risk

## Technical Details
- **Context**: CNC woodworking, small part workholding
- **Tab sizing**: 1/4" thick for strong hold in typical 3/4" sheet
- **Onion skin depth**: Leave ~0.05" (0.70" cut in 0.75" sheet)
- **Vacuum optimization**: Reduce exposed spoilboard area, ensure flat stock, strategic cut ordering
- **Bit selection**: Compression for large parts, down-shear for small
- **Process**: Two-pass strategy (roughing + finishing) reduces movement risk

## Strengths
- Practical, experience-based guidance on tab sizing relative to material thickness
- Vacuum optimization tips applicable regardless of tab strategy
- Cut ordering insight (small parts first) is relevant for toolpath optimization
- Specific dimensional recommendations (1/4" tabs, 0.05" skin)

## Weaknesses & Limitations
- Woodworking-specific — tab sizing doesn't apply directly to metal or plastics
- Anecdotal practitioner knowledge, not systematic testing
- Doesn't address tab placement (position along contour), only sizing
- No coverage of shared tabs or adjacent part coordination

## Connections
- [RES-019 WOODWEB Tabs vs Onion Skinning] — companion discussion on the same trade-off
- [RES-021 CNCZone Optimize Tabs] — CNCZone community adds tab placement heuristics
- [RES-028 Maslow Tabs Strategy] — similar material-relative sizing discussion
- [RES-018 HOMAG Nesting Features] — industrial solution combining tabs and onion skinning
