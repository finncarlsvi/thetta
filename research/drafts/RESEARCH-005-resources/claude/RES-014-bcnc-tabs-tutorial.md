# bCNC Tabs Tool Tutorial
> **URL**: https://github.com/vlachoudis/bCNC/wiki/Tutorials:-Tabs-tool
> **Type**: Community
> **Agent**: Claude
> **Relevance**: Medium
> **Credibility**: Medium

## Summary
This GitHub wiki tutorial provides a brief walkthrough of the tab tool in bCNC, the open-source GRBL CNC sender. The tutorial describes a simple three-step process: select the last pass block, execute the tab tool, and verify that 3 tabs have been added to the cutting path. It also links to an external PDF guide for more detailed configuration.

The key practical insight from this tutorial is the critical warning about the "only leave selected islands uncut" setting. This checkbox, found in the cut tool settings, must be unchecked for tabs to function correctly. If left checked (the default), the tab feature fails silently — no tabs are added, and the user may not realize until the part moves during cutting. This is a UX anti-pattern worth noting for Thetta's design: tab settings should not have hidden dependencies on unrelated features.

The tutorial confirms that bCNC's tab approach is last-pass-only: tabs are added as modifications to the final cutting pass, leaving the earlier passes (roughing) unaffected. This is the standard approach across CAM tools — tabs only matter for the final separation cut.

## Key Findings
- Simple workflow: select last pass → execute → 3 tabs added
- Critical: must uncheck "only leave selected islands uncut" in cut tool settings
- Tabs apply to last pass only (standard approach)
- External PDF guide available for detailed configuration
- Default tab count: 3

## Technical Details
- **Software**: bCNC (Python, open source)
- **Workflow**: Select last pass block → Tab tool → Execute
- **Default tabs**: 3
- **Prerequisite**: "only leave selected islands uncut" must be unchecked
- **Documentation**: GitHub wiki + external PDF guide

## Strengths
- Simple, practical workflow documentation
- Identifies a critical gotcha (island checkbox) that new users encounter
- Confirms last-pass-only tab pattern

## Weaknesses & Limitations
- Very brief — minimal detail on tab configuration options
- No information on tab placement algorithm or parameters
- Links to external PDF for actual detail (not accessible)
- No coverage of advanced scenarios (adjacent parts, varied tab sizes)

## Connections
- [RES-013 bCNC Repo] — the parent project for this tutorial
- [RES-011 FreeCAD DressupTag] — FreeCAD's more detailed approach to the same problem
- [RES-007 Fusion 360 Tabs Tutorial] — commercial equivalent tutorial with more parameter detail
