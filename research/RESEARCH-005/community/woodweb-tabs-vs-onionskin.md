# Tabs vs. Onion Skinning -- WOODWEB CNC Forum

> **URL**: https://www.woodweb.com/cgi-bin/forums/cnc.pl?read=818617
> **Type**: Community
> **Agents**: Claude, Codex, Gemini
> **Relevance**: Medium
> **Confidence**: Medium -- source expired, agents working from search summaries

## Summary
This WOODWEB forum discussion compares tabs and onion skinning as workholding strategies for CNC woodworking, with particular attention to maintaining vacuum hold-down for small parts. Critically, the discussion directly mentions the shared-tab alignment problem: tabs that do not line up with adjacent part tabs get "disconnected" when the neighboring part is cut -- the same core issue from the Fusion 360 forum (RES-008).

Some participants indicate that certain CAD/CAM systems have tabbing options ensuring alignment with adjacent part tabs, though the specific software is not named. This aligns with HOMAG's integrated approach (RES-018).

## Key Findings
- Misaligned tabs between adjacent parts get "disconnected" when neighboring part is cut (all agents)
- Some CAD/CAM systems have tab alignment features for adjacent parts (Claude, Gemini)
- Tabs vs. onion skinning trade-off depends on part size, material, vacuum quality (all agents)
- Manual tab input more suited for repeated nesting layouts (Claude, Codex)
- Failure occurs when part gaps are <= 2 tool diameters (Gemini)
- Financial consequences in professional shops: tool damage, ruined parts (Gemini)

## Technical Details
- **Forum**: WOODWEB CNC Forum
- **Problem**: Tab misalignment between adjacent nested parts
- **Threshold**: Problem occurs when gap <= 2 tool diameters (Gemini)
- **Content access**: Message expired -- content from search summaries

## Strengths
- Directly confirms the shared-tab alignment problem from practitioner experience
- Notes that some commercial CAM systems have addressed tab alignment
- Practical tabs vs. onion skinning trade-off comparison

## Weaknesses & Limitations
- Forum message expired -- limited content available
- Anecdotal practitioner discussion
- Specific software with tab alignment features not identified

## Consensus & Disagreements
All agents agree on the shared-tab misalignment finding. Gemini uniquely provides the two-tool-diameter threshold for when the problem occurs -- a concrete, actionable design parameter. Credibility varies: Codex rates Low, Claude/Gemini rate Medium. No factual disagreements.

## Connections
- [RES-008] Fusion 360 Adjacent Tabs Forum -- same problem, different community
- [RES-020] WOODWEB Tabbing Small Parts -- related discussion on small parts
- [RES-018] HOMAG Nesting Features -- likely candidate for the CAM system with tab alignment
- [RES-021] CNCZone Optimize Tabs -- another community perspective on tab optimization
