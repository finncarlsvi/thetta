# viaconstructor — Python CAM Tool with Tab Support
> **URL**: https://github.com/multigcs/viaconstructor
> **Type**: Code
> **Agent**: Claude
> **Relevance**: Medium
> **Credibility**: Medium

## Summary
viaconstructor is a Python/C++ CAM tool that converts DXF, SVG, HPGL, and STL files to G-code with features including 3D visualization, tabs, pockets with islands, and headless (batch) mode. It represents a mid-complexity open-source CAM implementation with tab support, sitting between bCNC's simplicity and FreeCAD's full-featured approach.

The tool implements automatic offset finding (inside/outside detection), automatic ordering via nearest-neighbor algorithm, and nested part handling with correct sequencing. Tab support is listed as a core feature, though the implementation has known limitations — specifically, tab generation on circular paths sometimes fails. This is a common challenge in tab placement: circular/curved contours have uniform curvature, making it harder to identify natural tab placement points (unlike polygonal contours where edges provide natural segmentation).

viaconstructor's architecture (~50% Python, ~46% C++) with PyQt5 and OpenGL visualization is interesting from an implementation perspective. The C++ component likely handles performance-critical geometry operations while Python handles the UI and workflow logic. The tool supports a wide range of input formats (DXF, SVG, HPGL, STL cross-sections) and limited support for CDR, PDF, EPS, and raster formats, making it relatively versatile.

## Key Findings
- Python/C++ hybrid architecture with tab support as a core feature
- Known bug: tab generation fails on circular paths
- Automatic offset finding (inside/outside) and nearest-neighbor ordering
- Supports nested part handling with correct sequencing
- Wide format support: DXF, SVG, HPGL, STL (cross-section)

## Technical Details
- **Language**: Python (~50%), C++ (~46%)
- **UI**: PyQt5 + OpenGL for 3D visualization
- **Input formats**: DXF, SVG, HPGL, STL, limited CDR/PDF/EPS/raster
- **Output**: G-code, HPGL
- **Algorithms**: Automatic offset finding, nearest-neighbor ordering, tab placement
- **Requirements**: Python 3.9+
- **License**: Open source (GitHub)

## Strengths
- Working tab implementation in open-source code available for study
- Nested part handling with sequencing — relevant to Thetta's context
- Hybrid Python/C++ architecture balances development speed and performance
- Wide format support demonstrates mature input pipeline
- Headless mode useful for batch processing

## Weaknesses & Limitations
- Tab generation fails on circular paths — indicates fragile algorithm
- No documentation of tab placement algorithm specifics
- Smaller community than FreeCAD or bCNC
- No shared-tab or adjacent-part tab awareness
- Limited information on tab configuration parameters

## Connections
- [RES-012 FreeCAD DressupTag Source] — more mature Python tab implementation for comparison
- [RES-013 bCNC Tabs] — simpler Python tab approach
- [RES-016 Kiri:Moto] — browser-based alternative with CNC tab support
- [RES-017 Deepnest] — nesting-focused tool (no tabs) that complements viaconstructor's CAM focus
