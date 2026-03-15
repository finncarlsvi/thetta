# FreeCAD CAM Workbench - CAM Research Summary

## Overview
FreeCAD's CAM Workbench (formerly "Path Workbench") is the integrated CAM module within FreeCAD, the popular open-source parametric 3D modeler. It generates G-code toolpaths from 3D models for CNC machines including mills, lathes, and laser cutters. The CAM workbench is primarily written in Python with a C++ core (via OpenCASCADE), and is by far the most feature-complete open-source CAM solution available. All three models agree it is the gold standard for open-source CAM capabilities, though they diverge sharply on its integration potential.

- **Primary Language:** C++ core with Python scripting layer (CAM workbench is mostly Python)
- **License:** LGPL v2.1
- **Source:** https://github.com/FreeCAD/FreeCAD (CAM module at `src/Mod/CAM/`)

## Activity & Maturity
All three models agree FreeCAD is extremely active and mature:
- **Last Commit:** Active daily; Codex confirms last push 2026-03-09
- **GitHub Stars:** ~28,700 (Claude)
- **Contributors:** ~960 (Codex), hundreds overall with notable CAM contributors in 2026 including @Connor9220, @Daniel-Khodabakhsh, @tarman3, @petterreinholdtsen (Claude)
- **Releases:** 84 releases (Codex); FreeCAD 1.0 released 2024 (Claude); weekly dev builds in 2026
- **Community Size:** Very large. Active forum, extensive documentation, thousands of users globally.

## Profile Cut Support
**All models agree: Excellent.** Fully supports 2D contouring with inside/outside contours, climb vs. conventional milling, CW/CCW direction control, step-down (multiple depths), lead-in/lead-out moves, and exact cutter radius compensation. The Profile operation can be applied to faces, edges, or loops. Also supports an "Adaptive" clearing/profiling operation for high-efficiency machining (Claude, Gemini).

## Tab Support
**All models agree: Excellent.** Tabs are implemented as a "Dressup" (also called "Tags" or "Holding Tags"). Key details:
- Automatic placement of 4 tabs by default, with manual override (Claude)
- Width, height, angle, spacing, and count are all configurable (Codex)
- Shape options include rectangular and triangular (Gemini)
- Tab parameters configurable in Edit > Preferences > CAM > Dressups (Claude)

## Pocket Cut Support
**All models agree: Excellent.** Multiple pocket strategies available:
- Pocket Shape (2.5D), Pocket 3D (Claude)
- Zigzag and offset/contour clearing patterns (all models)
- Adaptive/trochoidal clearing for high-efficiency machining (Claude, Gemini)
- Forum users report 2.5D pocketing works well; 3D pocketing is less reliable (Claude)

## G-code Output
**All models agree: Extensive.** FreeCAD uses an internal machine-agnostic G-code dialect, then translates via post-processors:
- 36 post-processor scripts in tree (Codex verified)
- Supported dialects: LinuxCNC, GRBL, Fanuc, HAAS, Heidenhain, Mach3/4, Masso, Marlin, Smoothie, Centroid, and many more
- Post-processors are Python scripts, fully customizable
- New post-processors can be written following documented conventions

## Integration Potential (for Rust application)
**Models disagree on degree, but agree on direction:**
- **Claude (moderate pessimism):** Can run headless via Python scripting; `ocp-freecad-cam` project exists for fluent Python API; massive dependency (~2GB+); extracting CAM algorithms extremely difficult; subprocess is most realistic approach.
- **Gemini (strong pessimism):** "Terrible for embedded Rust integration"; distributing a Rust app requiring entire FreeCAD ecosystem is "impractically heavy"; extracting C++ CAM algorithms "notoriously difficult" due to tight OpenCASCADE coupling.
- **Codex (moderate):** "Weak as embeddable library (monolith), but strong scripting/Python automation"; Rust integration likely via process/Python bridge, not direct linking.

**Consensus:** Not viable for embedding. Feasible as an external subprocess via Python scripting, but adds enormous dependency weight.

## Strengths
- Most feature-complete open-source CAM solution available (all models)
- Excellent profile, pocket, and tab support with advanced strategies (all models)
- Most extensive post-processor library of any open-source CAM (all models)
- Can be scripted headlessly via Python (Claude, Codex)
- Battle-tested in production by many users (all models)
- Very active development with large community (all models)
- `ocp-freecad-cam` project demonstrates headless Python API feasibility (Claude)

## Weaknesses
- Enormous dependency -- full 3D CAD/CAM application, ~2GB+ (all models)
- Python-based CAM algorithms not easily extracted into Rust (all models)
- Requires FreeCAD's document model and Part module; cannot use CAM in isolation (Claude)
- Tightly coupled to OpenCASCADE geometry engine (Gemini)
- Complex build system; not designed as an embeddable library (all models)
- Performance overhead of running as subprocess (Claude)

## Ratings

| Model | Rating | Key Reasoning |
|-------|--------|---------------|
| Claude Opus 4.6 | 3/5 | Most capable but integration burden very high; best as external process |
| Gemini | 2/5 | Incredible CAM tool, terrible for embedded Rust integration |
| GPT/Codex | 4/5 | Feature complete and mature; scripting/automation strong despite monolith |
| **Average** | **3.0/5** | |

## Sources
- [FreeCAD GitHub](https://github.com/FreeCAD/FreeCAD)
- [FreeCAD CAM Workbench Documentation](https://github.com/FreeCAD/FreeCAD-documentation/blob/main/wiki/CAM_Workbench.md)
- [FreeCAD CAM FAQ](https://wiki.freecad.org/Path_FAQ/en)
- [FreeCAD CAM Post Processor](https://wiki.freecad.org/Path_Post/en)
- [ocp-freecad-cam](https://github.com/voneiden/ocp-freecad-cam)
- [FreeCAD CNC Workflow (Way of Wood)](https://wayofwood.com/freecad-cnc-cad-and-cam-workflow/)
- [FreeCAD CAM modules and posts](https://github.com/FreeCAD/FreeCAD/tree/main/src/Mod/CAM)
