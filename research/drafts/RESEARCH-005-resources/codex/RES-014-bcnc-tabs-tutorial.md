# bCNC Tabs Tool Tutorial
> **URL**: https://github.com/vlachoudis/bCNC/wiki/Tutorials:-Tabs-tool
> **Type**: Community
> **Agent**: Codex
> **Relevance**: Medium
> **Credibility**: Medium

## Summary
This wiki tutorial documents the practical steps for adding tabs in bCNC, with emphasis on selecting the final pass and executing tab insertion. It is concise but useful for understanding real usage flow and defaults.

The guide confirms that tabbing in bCNC is tied to pass/block selection, which means automation quality depends on prior operation structuring. It also repeats the key configuration caveat about island settings, highlighting a common user trap.

For research, the tutorial is less about algorithm design and more about workflow ergonomics. It demonstrates how tab logic in many tools is operationally embedded and can fail silently when prerequisite settings are not met.

## Key Findings
- Standard usage is to select the last-pass block and run tab insertion (defaulting to three tabs).
- A specific cut-tool option must be disabled for tabs to apply as intended in many cases.

## Technical Details
The process suggests a block-level transformation pipeline over existing G-code/path definitions. This reinforces a pattern where tab insertion is a late-stage editing step rather than a globally optimized planner.

## Strengths
- Clear operator procedure for a real open-source CAM tool.

## Weaknesses & Limitations
- Minimal technical depth; no criteria for optimal tab position or count.

## Connections
- [RES-013-bcnc-repo.md](/Users/finn.carlsvi/thetta/research/drafts/RESEARCH-005-resources/codex/RES-013-bcnc-repo.md) — repository context explains where the tutorial fits in bCNC’s broader toolchain.
