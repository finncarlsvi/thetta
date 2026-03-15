# HOMAG Stay-Down, Common Line, Tabs, Onion Skinning
> **URL**: https://docs.homag.cloud/en/news/article/stay-down-common-line-include-tabs-onion-skinning
> **Type**: Commercial
> **Agent**: Claude
> **Relevance**: High
> **Credibility**: High

## Summary
HOMAG, a leading industrial CNC woodworking machinery manufacturer, provides the most comprehensive integrated approach to the tab placement problem found in this research. Their intelliDivide Nesting software combines four complementary workholding strategies: Stay-Down, Common Line, Include Tabs (retaining bars), and Onion Skinning. This integrated approach is the closest existing commercial solution to what Thetta aims to achieve.

**Stay-Down** eliminates individual tool approach/departure movements per workpiece. The tool approaches once and remains at milling depth for the entire cutting cycle, significantly reducing machining time. This technique works in conjunction with tabs — the tool stays down between parts, only raising at tab locations.

**Common Line** sets part spacing to match the tool diameter exactly, so a single cutter pass between two workpieces separates them simultaneously. This is the industrial implementation of the shared-edge concept from Deepnest and the academic literature.

**Include Tabs (Retaining Bars)** specifically targets small parts prone to shifting due to insufficient vacuum hold-down. The intelliDivide Nesting software connects small parts with retaining bars, with customizable tab size and quantity. This is the most relevant feature — it integrates tab placement directly into the nesting optimization, rather than treating it as a separate post-processing step.

**Onion Skinning** uses multi-pass cutting: pre-mill small parts in the first pass (leaving a thin skin), then complete all parts in the second pass. This reduces the window of time during which a part is fully separated and vulnerable to movement.

The integration of all four strategies in a single Formatting macro (requiring woodWOP 8 + Nesting Plugin, tabs available in v8.1.225+) represents the state of the art in commercial CNC workholding automation.

## Key Findings
- Four integrated strategies: Stay-Down + Common Line + Tabs + Onion Skinning
- Tab placement integrated into nesting optimization (not post-processing)
- Tabs specifically target small parts with insufficient vacuum hold-down
- Common line = single cut separates two parts (spacing = tool diameter)
- Onion skinning as complementary strategy for multi-pass stability
- Requires woodWOP 8 + Nesting Plugin (tabs in v8.1.225+)

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
- Proprietary — no algorithmic details available
- Specific to HOMAG machinery ecosystem (woodWOP)
- Focus on wood panel processing — may not generalize to metal/plastics
- No details on how tab placement is optimized within the nesting context
- Expensive industrial solution, not accessible for study

## Connections
- [RES-017 Deepnest] — open-source common-line implementation, Deepnest without tabs
- [RES-002 Common-Edge Nested Cutting] — academic formalization of HOMAG's common-line approach
- [RES-008 Fusion 360 Adjacent Tabs Forum] — the problem HOMAG's integrated approach solves
- [RES-019 WOODWEB Tabs vs Onion Skinning] — community discussion of the tabs/onion-skin trade-off HOMAG combines
- [RES-020 WOODWEB Tabbing Small Parts] — practical perspective on the small-part problem HOMAG's tabs target
