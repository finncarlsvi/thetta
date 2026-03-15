# Tabs vs. Onion Skinning — WOODWEB CNC Forum
> **URL**: https://www.woodweb.com/cgi-bin/forums/cnc.pl?read=818617
> **Type**: Community
> **Agent**: Claude
> **Relevance**: Medium
> **Credibility**: Medium

## Summary
This WOODWEB forum discussion compares tabs and onion skinning as workholding strategies for CNC woodworking, with particular attention to the problem of maintaining vacuum hold-down when cutting small parts. The discussion is notable for directly mentioning the shared-tab alignment problem: tabs that don't line up with adjacent part tabs get "disconnected" — the same core issue identified in the Fusion 360 forum discussion (RES-008).

The forum participants discuss strategies for maintaining vacuum effectiveness while cutting small parts that are prone to movement once separated from the stock. Tabs provide stronger hold-down but require post-processing (sanding, cutting). Onion skinning (leaving a thin layer on the bottom) is easier to clean up but provides less holding force. The choice between them depends on part size, material, vacuum system quality, and acceptable post-processing effort.

A key insight from this discussion is that some CAD/CAM systems have tabbing options that ensure alignment with adjacent part tabs. This suggests that at least some commercial systems have attempted to solve the shared-tab problem, though the specific software is not named. This aligns with HOMAG's integrated approach (RES-018) where tab placement is part of the nesting optimization.

The forum content was expired at fetch time, so details come from search result summaries. Despite the limited content, the confirmation of the shared-tab alignment problem from a practitioner perspective is valuable.

## Key Findings
- Misaligned tabs between adjacent parts get "disconnected" when neighboring part is cut
- Some CAD/CAM systems have tabbing options ensuring alignment with adjacent part tabs
- Tabs vs. onion skinning trade-off depends on part size, material, vacuum quality
- Manual tab input more suited for repeated nesting layouts
- Vacuum effectiveness is central to the workholding decision

## Technical Details
- **Forum**: WOODWEB CNC Forum
- **Topic**: Tabs vs. onion skinning for small part workholding
- **Key problem**: Tab misalignment between adjacent nested parts
- **Solutions mentioned**: CAM systems with alignment features, manual input for repeated layouts
- **Content access**: FETCH_FAILED (message expired)

## Strengths
- Directly confirms the shared-tab alignment problem from practitioner experience
- Notes that some commercial CAM systems have addressed tab alignment
- Practical comparison of tabs vs. onion skinning trade-offs
- Represents real-world woodworking CNC community knowledge

## Weaknesses & Limitations
- Forum message expired — limited content available
- Anecdotal practitioner discussion, not systematic analysis
- Specific software with tab alignment features not identified
- Woodworking-specific context may not generalize fully

## Connections
- [RES-008 Fusion 360 Adjacent Tabs Forum] — same shared-tab alignment problem, different community
- [RES-020 WOODWEB Tabbing Small Parts] — related WOODWEB discussion on small parts
- [RES-018 HOMAG Nesting Features] — likely candidate for the CAM system with tab alignment mentioned
- [RES-021 CNCZone Optimize Tabs] — another community perspective on tab optimization
