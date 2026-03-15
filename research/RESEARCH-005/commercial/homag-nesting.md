# HOMAG Stay-Down, Common Line, Tabs, Onion Skinning

> **URL**: https://docs.homag.cloud/en/news/article/stay-down-common-line-include-tabs-onion-skinning
> **Type**: Commercial
> **Agents**: Claude, Codex, Gemini
> **Relevance**: High
> **Confidence**: High -- agents agree on four-strategy integration

## Summary
HOMAG, a leading industrial CNC woodworking machinery manufacturer, provides the most comprehensive integrated approach to the tab placement problem found in this research. Their intelliDivide Nesting software combines four complementary workholding strategies in a single Formatting macro: Stay-Down, Common Line, Include Tabs (retaining bars), and Onion Skinning.

**Stay-Down** eliminates individual tool approach/departure movements per workpiece. The tool approaches once and remains at milling depth for the entire cutting cycle. **Common Line** sets part spacing to match the tool diameter exactly, so a single cutter pass between two workpieces separates them simultaneously. **Include Tabs (Retaining Bars)** specifically targets small parts prone to shifting due to insufficient vacuum hold-down, with customizable tab size and quantity. **Onion Skinning** uses multi-pass cutting: pre-mill small parts in the first pass (leaving a thin skin), then complete all parts in the second pass.

The integration of tab placement directly into the nesting optimization (rather than treating it as post-processing) is the most significant architectural distinction. This is the closest existing commercial solution to what Thetta aims to achieve.

## Key Findings
- Four integrated strategies: Stay-Down + Common Line + Tabs + Onion Skinning (all agents)
- Tab placement integrated into nesting optimization, not post-processing (Claude)
- Tabs specifically target small parts with insufficient vacuum hold-down (all agents)
- Common line = single cut separates two parts (spacing = tool diameter) (Claude, Gemini)
- Stay-Down pathing relies on tabs because the tool never retracts to clear separated parts (Gemini)
- Tabs and Onion Skinning are not mutually exclusive; they can be combined (Gemini)
- Requires woodWOP 8 + Nesting Plugin (tabs in v8.1.225+) (Claude)

## Technical Details
- **Vendor**: HOMAG (industrial CNC woodworking leader)
- **Software**: intelliDivide Nesting + woodWOP 8
- **Tab feature**: "Include Tabs" / Retaining Bars
- **Integration**: All four features work together in Formatting macro
- **Version requirement**: woodWOP 8.1.225+ for tab support
- **Target market**: Industrial wood panel processing, furniture manufacturing

## Strengths
- Only commercial solution found that integrates tab placement with nesting optimization
- Four complementary strategies provide comprehensive workholding
- Industrial-grade solution from a market leader
- Common-line + tabs combination directly addresses shared-tab problem
- Tab size and quantity are customizable

## Weaknesses & Limitations
- Proprietary -- no algorithmic details available
- Specific to HOMAG machinery ecosystem (woodWOP)
- Focus on wood panel processing -- may not generalize to metal/plastics
- No details on how tab placement is optimized within the nesting context

## Consensus & Disagreements
All agents agree on the four strategies and their integration. One point of discussion: Gemini claims Stay-Down "relies heavily on tabs" (causal dependency -- if the tool never retracts, every part must be locked), while Claude describes them as complementary. The truth likely lies between -- Stay-Down requires some hold-down mechanism (tabs, vacuum, or both). No other factual disagreements.

## Connections
- [RES-017] Deepnest -- open-source common-line implementation without tabs
- [RES-002] Common-Edge Nested Cutting -- academic formalization of common-line approach
- [RES-008] Fusion 360 Adjacent Tabs Forum -- the problem HOMAG's integrated approach solves
- [RES-019] WOODWEB Tabs vs Onion Skinning -- community discussion of the tabs/onion-skin trade-off
- [RES-020] WOODWEB Tabbing Small Parts -- practical perspective on the small-part problem HOMAG's tabs target
