# RES-015: viaconstructor — Python CAM tool with tab support
**URL**: https://github.com/multigcs/viaconstructor
**Type**: Code
**Status**: CACHED

## Content
- Python-based CAM tool converting DXF/SVG/HPGL/STL to G-code
- Features: 3D visualization, tabs and pockets with islands, headless mode
- Automatic offset finding (inside/outside), automatic ordering (nearest-neighbor)
- Nested part handling with correct sequencing
- Input: DXF, SVG, HPGL, STL (cross-section), limited CDR/PDF/EPS/raster
- Output: gCode, HPGL
- ~50% Python, ~46% C++, uses PyQt5 + OpenGL
- Known limitation: tab generation on circular paths sometimes fails
- Requires Python 3.9+
