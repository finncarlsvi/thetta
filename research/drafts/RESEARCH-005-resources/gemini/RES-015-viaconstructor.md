# viaconstructor Python CAM tool
> **URL**: https://github.com/multigcs/viaconstructor
> **Type**: Code
> **Agent**: Gemini
> **Relevance**: Medium
> **Credibility**: High

## Summary
Viaconstructor is an open-source CAM tool written in a mix of Python (approx 50%) and C++ (approx 46%). It is designed to convert 2D vectors (DXF, SVG, etc.) and 3D cross-sections (STL) into G-code. It specifically advertises support for tabs, pockets, and islands.

The tool features automatic offset finding (inside vs. outside cuts) and automatic path ordering using a nearest-neighbor algorithm. It is capable of handling nested parts with correct cutting sequences (cutting inner holes before outer perimeters).

A notable detail in the repository's documentation is a known bug: tab generation sometimes fails on circular paths. This highlights a common difficulty in computational geometry for CAM: algorithms that work perfectly on straight line segments often break down when calculating offsets or retractions along arcs and splines.

## Key Findings
- Python and C++ hybrid architectures are common for CAM tools, balancing rapid UI/logic development (Python) with high-performance geometric calculations (C++).
- Correct sequencing (inner cuts before outer cuts) is a prerequisite for any tab placement system to work.
- Circular paths and arcs present specific mathematical challenges for automatic tab generation algorithms, often causing failures in open-source implementations.

## Technical Details
Viaconstructor uses PyQt5 for its interface and OpenGL for 3D visualization. The core logic handles automatic offset calculation and nearest-neighbor sequencing. The failure on circular paths likely stems from how arcs are discretized or how normal vectors are calculated at the tab points.

## Strengths
- Open-source example of handling complex CAM features (offsets, sequencing, tabs) in a Python/C++ environment.
- Explicitly documents its limitations, providing insight into difficult edge cases (circular paths).

## Weaknesses & Limitations
- Tab generation is known to be buggy on curves.
- Does not appear to have specific logic for shared tabs between closely nested parts.

## Connections
- Similar architectural approach (Python CAM) to bCNC (RES-013), but with a more sophisticated C++ backend for geometry.
