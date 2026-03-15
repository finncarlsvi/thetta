# bCNC -- GRBL CNC Sender with Cutout Tabs

> **URL**: https://github.com/vlachoudis/bCNC
> **Type**: Code
> **Agents**: Claude, Codex, Gemini
> **Relevance**: Medium
> **Confidence**: High -- agents agree on feature set

## Summary
bCNC is a cross-platform, Python-based CNC command sender for GRBL/GrblHAL controllers that includes built-in CAM functionality, including tab support. Unlike dedicated CAM tools, bCNC is primarily a G-code sender with CAM features added for convenience, making its tab implementation simpler but less feature-rich than dedicated solutions.

The software's tab functionality is integrated into the "cut" tool rather than being a standalone feature. The default configuration adds 3 tabs to the last pass of a cutting operation. A critical implementation detail: the "only leave selected islands uncut" checkbox must be unchecked for tabs to function, indicating that tabs and island handling share underlying code paths. Gemini uniquely observes that bCNC treats holding tabs algorithmically as small "islands" of un-machined material along the toolpath, providing insight into the internal conceptual model.

bCNC's value for Thetta is as a reference implementation of a simple, pragmatic tab approach. The 3-tab default aligns with the academic recommendation from RES-001 and the community consensus from RES-021.

## Key Findings
- Default: 3 tabs added to last pass of cutting operation (Claude, Gemini)
- Tab feature integrated into the "cut" tool, not standalone (all agents)
- Critical setting: must uncheck "only leave selected islands uncut" for tabs to work (all agents)
- bCNC treats tabs algorithmically as small "islands" of un-machined material (Gemini)
- Integration into a CNC sender shows demand for last-minute fixturing adjustments at the machine interface (Gemini)

## Technical Details
- **Language**: Python
- **License**: GPL-2.0
- **Controller**: GRBL / GrblHAL
- **Tab default**: 3 tabs on last pass
- **CAM features**: Profiling, pocketing, drilling, cutout tabs, drag knife, thread milling
- **Platform**: Cross-platform (Windows, macOS, Linux)
- **Tab mechanism**: Injects Z-retract and Z-plunge commands at calculated points along the final XY perimeter pass

## Strengths
- Simple, working implementation of tabs in an open-source tool
- 3-tab default is well-aligned with academic and community recommendations
- Active open-source project with community support
- Highly accessible Python codebase (Gemini)

## Weaknesses & Limitations
- Very basic tab placement -- no automatic distribution algorithm documented
- Tab-island interaction (checkbox dependency) suggests implementation coupling
- Primarily a G-code sender, not a dedicated CAM tool
- No shared-tab or adjacent-part tab awareness
- Tab generation is sometimes counter-intuitive due to checkbox toggles (Gemini)

## Consensus & Disagreements
All agents agree on the core features and the island-checkbox issue. Gemini uniquely provides the conceptual insight that tabs are treated as "islands." Claude provides the specific 3-tab default and GPL-2.0 license. Credibility ratings differ: Codex and Gemini rate High, Claude rates Medium. No factual disagreements.

## Connections
- [RES-014] bCNC Tabs Tutorial -- tutorial for using the tab feature
- [RES-012] FreeCAD DressupTag Source -- more sophisticated Python tab implementation
- [RES-015] viaconstructor -- another Python CAM tool with tab support
- [RES-001] Micro-Joints Genetic Algorithm -- academic basis for the 3-tab default
