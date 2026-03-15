# bCNC Tabs Tool Tutorial

> **URL**: https://github.com/vlachoudis/bCNC/wiki/Tutorials:-Tabs-tool
> **Type**: Community
> **Agents**: Claude, Codex, Gemini
> **Relevance**: Medium
> **Confidence**: High -- agents agree on workflow

## Summary
This GitHub wiki tutorial provides a brief walkthrough of the tab tool in bCNC. The tutorial describes a simple three-step process: select the last pass block, execute the tab tool, and verify that 3 tabs have been added. It also warns about the critical "only leave selected islands uncut" setting which must be unchecked for tabs to function correctly -- a silent failure mode that is a UX anti-pattern worth noting for Thetta's design.

The tutorial confirms that bCNC's tab approach is last-pass-only, which limits tab thickness to the depth of a single cutting pass. The 3-tab default aligns with the minimum-3-tab rule found across academic and community sources.

## Key Findings
- Simple workflow: select last pass -> execute -> 3 tabs added (all agents)
- Critical: must uncheck "only leave selected islands uncut" in cut tool settings (all agents)
- Tabs apply to last pass only (standard approach) (all agents)
- Last-pass-only tabs limit tab thickness to single pass depth (Gemini)
- Silent failure is a UX anti-pattern: tab settings should not have hidden dependencies (Claude)
- Default of 3 tabs is minimum required to prevent pivoting (Gemini)

## Technical Details
- **Software**: bCNC (Python, open source)
- **Workflow**: Select last pass block -> Tab tool -> Execute
- **Default tabs**: 3
- **Prerequisite**: "only leave selected islands uncut" must be unchecked
- **Tab mechanism**: Injects Z-retract/plunge at 3 calculated points along final pass

## Strengths
- Simple, practical workflow documentation
- Identifies a critical gotcha that new users encounter
- Confirms last-pass-only tab pattern and 3-tab default

## Weaknesses & Limitations
- Very brief -- minimal detail on tab configuration options
- No information on tab placement algorithm or parameters
- No coverage of advanced scenarios

## Consensus & Disagreements
All agents agree on the workflow and the island-checkbox issue. Gemini uniquely identifies the pass-depth constraint on tab thickness. Claude uniquely flags the UX anti-pattern. No factual disagreements.

## Connections
- [RES-013] bCNC Repo -- the parent project
- [RES-011] FreeCAD DressupTag -- more detailed open-source approach
- [RES-007] Fusion 360 Tabs Tutorial -- commercial equivalent with more parameters
