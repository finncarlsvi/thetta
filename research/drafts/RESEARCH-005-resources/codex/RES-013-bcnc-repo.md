# bCNC — GRBL CNC Sender with Cutout Tabs
> **URL**: https://github.com/vlachoudis/bCNC
> **Type**: Code
> **Agent**: Codex
> **Relevance**: Medium
> **Credibility**: High

## Summary
bCNC is an open-source Python GRBL sender/editor with integrated CAM utilities, including cutout tab support. It is relevant as a practical open implementation used by hobby and light-production workflows.

The tool’s tab feature is integrated into the cut operation rather than a standalone optimization engine. That makes it useful for understanding default tab insertion UX and parameter conventions, though less useful for advanced global optimization strategies.

From a research perspective, bCNC contributes operational lessons: tabs are often generated in a specific pass context, and defaults can quietly disable expected behavior if options are misconfigured. This reinforces the need for explicit validation and user feedback in tab workflows.

## Key Findings
- bCNC includes native cutout tab support within its CAM toolchain and defaults to simple tab insertion behavior.
- A critical setting (“only leave selected islands uncut”) can prevent expected tab results unless disabled.

## Technical Details
The project is primarily Python and ties tab generation to G-code transformation in cut operations. Practical architecture appears rule-based and pass-oriented, suitable for lightweight jobs but not necessarily for multi-part shared-tab optimization across nested boundaries.

## Strengths
- Open, widely used codebase that demonstrates tab insertion in an end-to-end CAM-to-G-code workflow.

## Weaknesses & Limitations
- Limited evidence of advanced automatic placement algorithms or adjacent-part coordination.

## Connections
- [RES-014-bcnc-tabs-tutorial.md](/Users/finn.carlsvi/thetta/research/drafts/RESEARCH-005-resources/codex/RES-014-bcnc-tabs-tutorial.md) — the tutorial provides concrete operator sequence for invoking bCNC tab insertion.
