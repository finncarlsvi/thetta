# Maslow CNC Tabs Strategy Discussion
> **URL**: https://forums.maslowcnc.com/t/help-with-tabs-strategy/432
> **Type**: Community
> **Agent**: Gemini
> **Relevance**: Low
> **Credibility**: Medium

## Summary
The Maslow CNC is a unique, vertical, large-format CNC router popular in the maker community. Due to its vertical orientation, gravity plays a massive role, making tab strategy absolutely critical to prevent parts from falling out of the sheet during cutting.

Users in this forum thread discuss practical dimensions for tabs. A common consensus for 0.500" plywood is to use 0.255" thick tabs, with a width equal to 1.5 times the depth. This provides a tangible, ratio-based heuristic for sizing tabs based on material thickness.

Because the Maslow uses relatively low-power routers, users heavily rely on multi-pass strategies. They cut almost entirely through the material, leaving thick tabs, and then do a final, very light pass to trim the tabs down to their final breakaway thickness. Finally, they discuss various manual removal methods, favoring oscillating multi-tools or flush-cut pull saws.

## Key Findings
- Tab size should scale proportionally with material thickness (e.g., width = 1.5x depth).
- Vertical or unconventional CNC setups require significantly more robust tabbing strategies due to gravity.
- Multi-pass cutting is essential for lower-power machines to avoid snapping tabs.

## Technical Details
The multi-pass strategy involves generating two separate toolpaths: one that leaves massive tabs for stability during the heavy hogging passes, and a second, delicate finishing pass that reduces the tabs to easily breakable dimensions.

## Strengths
- Provides a concrete ratio heuristic for tab sizing (width = 1.5x depth).
- Highlights the extreme importance of tabs in non-standard (vertical) machining environments.

## Weaknesses & Limitations
- Highly specific to a niche, low-power maker machine.
- Low relevance to the algorithmic placement of tabs in a software engine.

## Connections
- The multi-pass strategy echoes the onion-skinning and two-pass techniques discussed in WoodWeb (RES-020).
