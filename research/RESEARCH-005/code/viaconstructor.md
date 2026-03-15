# viaconstructor -- Python CAM Tool with Tab Support

> **URL**: https://github.com/multigcs/viaconstructor
> **Type**: Code
> **Agents**: Claude, Codex, Gemini
> **Relevance**: Medium
> **Confidence**: Medium -- agents agree on features but speculate differently on bug cause

## Summary
viaconstructor is a Python/C++ CAM tool that converts DXF, SVG, HPGL, and STL files to G-code with features including 3D visualization, tabs, pockets with islands, and headless (batch) mode. It represents a mid-complexity open-source CAM implementation with tab support, sitting between bCNC's simplicity and FreeCAD's full-featured approach.

The tool implements automatic offset finding (inside/outside detection), automatic ordering via nearest-neighbor algorithm, and nested part handling with correct sequencing. Tab support is listed as a core feature, though the implementation has known limitations -- specifically, tab generation on circular paths sometimes fails. This is a common challenge in tab placement: circular/curved contours present specific mathematical challenges for algorithms that work perfectly on straight line segments.

## Key Findings
- Python/C++ hybrid architecture (~50%/~46%) with tab support as a core feature (all agents)
- Known bug: tab generation fails on circular paths (all agents)
- Automatic offset finding (inside/outside) and nearest-neighbor ordering (Claude, Gemini)
- Supports nested part handling with correct sequencing (Claude, Gemini)
- Circular-path failure likely stems from arc discretization or normal vector calculation issues (Gemini)

## Technical Details
- **Language**: Python (~50%), C++ (~46%)
- **UI**: PyQt5 + OpenGL for 3D visualization
- **Input formats**: DXF, SVG, HPGL, STL, limited CDR/PDF/EPS/raster
- **Output**: G-code, HPGL
- **Requirements**: Python 3.9+
- **Features**: Headless/batch mode, automatic offset finding, nearest-neighbor ordering

## Strengths
- Working tab implementation in open-source code available for study
- Nested part handling with sequencing relevant to Thetta's context
- Hybrid Python/C++ architecture balances development speed and performance
- Explicitly documents its limitations, providing insight into edge cases (Gemini)

## Weaknesses & Limitations
- Tab generation fails on circular paths -- indicates fragile algorithm
- No documentation of tab placement algorithm specifics
- Smaller community than FreeCAD or bCNC
- No shared-tab or adjacent-part tab awareness

## Consensus & Disagreements
All agents agree on the hybrid architecture, tab support, and circular-path bug. The three agents speculate differently on the bug cause: Claude suggests uniform curvature lacking natural segmentation points, Gemini suggests arc discretization/normal vector issues, and Codex suggests contour discretization and offset geometry interaction. None are verified. Credibility ratings differ (Medium from Claude, High from Codex/Gemini).

## Connections
- [RES-012] FreeCAD DressupTag Source -- more mature Python tab implementation for comparison
- [RES-013] bCNC Tabs -- simpler Python tab approach
- [RES-016] Kiri:Moto -- browser-based alternative with CNC tab support
- [RES-017] Deepnest -- nesting-focused tool that complements viaconstructor's CAM focus
