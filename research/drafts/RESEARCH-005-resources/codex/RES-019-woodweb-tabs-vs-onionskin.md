# Tabs vs. Onion Skinning — WOODWEB CNC Forum
> **URL**: https://www.woodweb.com/cgi-bin/forums/cnc.pl?read=818617
> **Type**: Community
> **Agent**: Codex
> **Relevance**: Medium
> **Credibility**: Low

## Summary
This forum discussion compares tabs and onion-skinning for maintaining part retention during nested cutting, especially under vacuum hold-down constraints. It reflects practitioner problem-solving in real shop conditions rather than formal experimentation.

Its most relevant contribution is the observation that misaligned tabs between neighboring parts can become disconnected. That directly validates the shared-tab alignment problem described in the research brief and shows that operators notice it as a practical failure mode.

The source is less reliable for universal rules, but it is still useful as field evidence. It highlights where CAM defaults fail and where manual intervention is commonly required in repeated production layouts.

## Key Findings
- Practitioners report that adjacent-part tab misalignment can break hold continuity when neighboring cuts remove support.
- Forum responses suggest some CAM systems can align tabs, but manual setup is often used for repeat nests.

## Technical Details
The discussion implies a process-level decision tree: choose tabbing for stronger discrete retention, onion skinning for broad support and cleaner finishing in some materials, and align tabs across neighboring contours when spacing is tight.

## Strengths
- Captures real operator pain points that map directly to the shared-tab research question.

## Weaknesses & Limitations
- Anecdotal forum content with expired source thread limits verifiability and detail.

## Connections
- [RES-008-fusion360-adjacent-tabs-forum.md](/Users/finn.carlsvi/thetta/research/drafts/RESEARCH-005-resources/codex/RES-008-fusion360-adjacent-tabs-forum.md) — both sources independently report adjacent-part tab interference as a recurring CAM issue.
