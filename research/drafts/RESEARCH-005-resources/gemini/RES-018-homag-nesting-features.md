# HOMAG Stay-Down, Common Line, Tabs, Onion Skinning
> **URL**: https://docs.homag.cloud/en/news/article/stay-down-common-line-include-tabs-onion-skinning
> **Type**: Commercial
> **Agent**: Gemini
> **Relevance**: High
> **Credibility**: High

## Summary
This documentation from HOMAG, a massive industrial manufacturer of woodworking machinery, describes their integrated approach to CNC nesting through their woodWOP software. It presents a holistic view of how high-end commercial systems combine multiple strategies to maximize efficiency and part stability.

HOMAG bundles four features into a single formatting macro: Stay-Down, Common Line, Tabs (Retaining Bars), and Onion Skinning. "Stay-Down" keeps the tool at milling depth between cuts, eliminating Z-retracts. "Common Line" spaces parts exactly one tool diameter apart, allowing a single pass to separate two workpieces simultaneously.

Crucially, HOMAG explicitly integrates Tabs (which they call "retaining bars") into this common-line workflow. They note that tabs are specifically required for small parts that lack enough surface area for vacuum hold-down. The system allows customization of tab size and quantity, and uniquely, it can combine tabs with "Onion Skinning" (multi-pass cutting) for ultimate stability.

## Key Findings
- High-end commercial systems explicitly support placing tabs on common-line cuts between adjacent parts.
- The primary driver for tabs in industrial nesting is surface area: small parts require tabs because they cannot generate enough vacuum hold-down force.
- "Stay-Down" pathing relies heavily on tabs, as the tool never retracts to clear separated parts; all parts must remain rigidly locked in the sheet.
- Tabs and Onion Skinning are not mutually exclusive; they can be combined for difficult materials.

## Technical Details
The system requires the toolpath to be perfectly offset by exactly one tool diameter to achieve the common-line cut. The retaining bars are generated dynamically within this shared toolpath. This feature is integrated deeply into their intelliDivide Nesting engine.

## Strengths
- Shows the state-of-the-art in industrial nesting and tab placement.
- Directly addresses the integration of tabs with common-line cutting.

## Weaknesses & Limitations
- Proprietary software; the exact algorithms are closed-source.
- Designed specifically for heavy-duty industrial vacuum tables, which may differ from hobby/prosumer CNC requirements.

## Connections
- Solves the problem described in RES-008 (adjacent parts destroying tabs).
- Commercial equivalent to the common-line theories in RES-002 and RES-017.
