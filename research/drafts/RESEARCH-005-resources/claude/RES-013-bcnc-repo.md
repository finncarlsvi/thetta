# bCNC — GRBL CNC Sender with Cutout Tabs
> **URL**: https://github.com/vlachoudis/bCNC
> **Type**: Code
> **Agent**: Claude
> **Relevance**: Medium
> **Credibility**: Medium

## Summary
bCNC is a cross-platform, Python-based CNC command sender for GRBL/GrblHAL controllers that includes built-in CAM functionality, including tab support. Unlike dedicated CAM tools, bCNC is primarily a G-code sender with CAM features added for convenience, making its tab implementation simpler but less feature-rich than dedicated solutions.

The software's CAM capabilities include profiling, pocketing, drilling, flat/helical/ramp cutting, thread milling, cutout tabs, and drag knife operations. Tab functionality is integrated into the "cut" tool rather than being a standalone feature. The default configuration adds 3 tabs to the last pass of a cutting operation. An important implementation detail: the "only leave selected islands uncut" checkbox must be unchecked for tabs to function, indicating that tabs and island handling share underlying code paths.

bCNC's value for Thetta is as a reference implementation of a simple, pragmatic tab approach. The 3-tab default aligns with the academic recommendation from RES-001 (centroid-based 3 micro-joints) and the community consensus from RES-021 (minimum 3 tabs to prevent rotation). The simplicity of the implementation — add N tabs to the last pass — is a useful baseline for comparison with more sophisticated algorithms.

## Key Findings
- Default: 3 tabs added to last pass of cutting operation
- Tab feature integrated into the "cut" tool, not standalone
- Critical setting: must uncheck "only leave selected islands uncut" for tabs to work
- Cross-platform Python implementation
- Open source, actively maintained

## Technical Details
- **Language**: Python
- **License**: GPL-2.0
- **Controller**: GRBL / GrblHAL
- **Tab default**: 3 tabs on last pass
- **CAM features**: Profiling, pocketing, drilling, cutout tabs, drag knife, thread milling
- **Platform**: Cross-platform (Windows, macOS, Linux)
- **Dependencies**: Python + tkinter

## Strengths
- Simple, working implementation of tabs in an open-source tool
- 3-tab default is well-aligned with academic and community recommendations
- Active open-source project with community support
- Practical tool used by many hobbyist CNC operators

## Weaknesses & Limitations
- Very basic tab placement — no automatic distribution algorithm documented
- Tab-island interaction (checkbox dependency) suggests implementation coupling
- Primarily a G-code sender, not a dedicated CAM tool — limited features
- No documentation of tab placement heuristics or algorithms
- No support for adjacent/shared tabs

## Connections
- [RES-014 bCNC Tabs Tutorial] — tutorial for using the tab feature
- [RES-012 FreeCAD DressupTag Source] — more sophisticated Python tab implementation
- [RES-015 viaconstructor] — another Python CAM tool with tab support
- [RES-001 Micro-Joints Genetic Algorithm] — academic basis for the 3-tab default
