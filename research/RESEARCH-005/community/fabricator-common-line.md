# When to Cut a Common Line -- The Fabricator

> **URL**: https://www.thefabricator.com/thefabricator/article/punching/when-to-cut-a-common-line
> **Type**: Community
> **Agents**: Claude, Codex, Gemini
> **Relevance**: Medium
> **Confidence**: High -- agents agree on microtab/common-line combination

## Summary
This article from The Fabricator, a leading trade publication for metal fabrication, discusses common-line cutting -- where two adjacent parts share a single cut line. The article is particularly valuable because it explicitly addresses the combination of common-line cutting with microtabs, the exact shared-tab scenario Thetta aims to support.

Common-line cutting reduces material waste and total cutting distance. The key finding for Thetta: parts stay tabbed in the nest until the end of the cutting operation, then are snapped out. When two adjacent parts share a common cut line and both have tabs on that line, a single tab holds both parts until deliberate removal. Parts tabbed together on a common line act as a single structural unit during the cutting process.

The article raises important concerns about thermal cutting: heat expansion can shift parts mid-cut and propagate errors across an entire sheet. For cold cutting processes (CNC routing, waterjet), these thermal concerns do not apply.

## Key Findings
- Common-line cutting + microtabs = parts stay in nest until end, then snapped out (all agents)
- Microtabs on common lines add process stability (all agents)
- Never let kerf from one piece overlap kerf from adjacent piece (Claude, Gemini)
- Thermal cutting: heat expansion can shift parts and propagate errors across sheet (all agents)
- Cold cutting (CNC routing) avoids thermal concerns, simplifying shared-tab design (Claude)
- Parts tabbed on a common line act as a single structural unit (Gemini)

## Technical Details
- **Publication**: The Fabricator (leading metal fabrication trade publication)
- **Topic**: Common-line cutting practices and when to use them
- **Key technique**: Microtabs on shared edges for process stability
- **Kerf rule**: Never overlap kerf paths from adjacent parts
- **Content access**: Behind paywall (403)

## Strengths
- Explicitly describes microtab + common-line combination
- Authoritative trade publication with industry credibility
- Thermal vs. cold cutting distinction is relevant for material-specific strategies

## Weaknesses & Limitations
- Metal fabrication focus -- some concerns less relevant for CNC routing
- No algorithmic detail on tab placement
- Does not describe how microtabs are positioned on common lines

## Consensus & Disagreements
All agents agree on the microtab + common-line combination. Gemini provides the cascading error-propagation framing. Claude provides the thermal vs. cold cutting distinction. No factual disagreements.

## Connections
- [RES-002] Common-Edge Nested Cutting -- academic treatment of common-edge optimization
- [RES-017] Deepnest -- open-source common-line merging
- [RES-018] HOMAG Nesting Features -- commercial common-line + tab integration
- [RES-008] Fusion 360 Adjacent Tabs Forum -- user-facing version of shared-tab problem
