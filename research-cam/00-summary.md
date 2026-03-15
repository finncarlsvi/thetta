# Open-Source CAM Research Summary

**Date:** 2026-03-09
**Purpose:** Evaluate 12 open-source CAM projects for integration into a Rust-based CNC nesting application (Thetta). Required operations: profile cuts, tab placement, and pocket cuts for a 3-axis CNC router/mill.
**Models consulted:** Claude Opus 4.6, Gemini, GPT/Codex

---

## Comparison Table

| # | Project | Language | License | Profile | Tabs | Pocket | G-code | Integration | Avg Rating |
|---|---------|----------|---------|---------|------|--------|--------|-------------|------------|
| 1 | FreeCAD CAM | Python/C++ | LGPL-2.1 | Excellent | Excellent (auto) | Excellent | Excellent (36 posts) | Subprocess only | 3.0/5 |
| 2 | BlenderCAM/Fabex | Python | GPL-3.0 | Basic-Good | Disputed | Good (3D focus) | Basic-Good | Requires Blender | 1.7/5 |
| 3 | PyCAM | Python | GPL-3.0 | Good | Yes (auto bridges) | Basic | Limited (LinuxCNC) | CLI subprocess | 2.3/5 |
| 4 | OpenCAMLib | C++ | LGPL-2.1 | Limited (3D) | None | Experimental | None | Excellent FFI, wrong algos | 2.3/5 |
| 5 | DerpCAM | Python | GPL-3.0 | Good | Yes (auto+manual) | Good (trochoidal) | LinuxCNC/GRBL | Python API | 2.7/5 |
| 6 | Kiri:Moto | JavaScript | MIT | Excellent | Yes (outside only) | Good | Configurable | Node.js CLI/API | **3.8/5** |
| 7 | jscut | JS/C++ | GPL-3.0 | Good | Buggy | Basic (Clipper) | Basic | Abandoned | 2.0/5 |
| 8 | OpenBuilds CAM | JavaScript | AGPL-3.0 | Good | Yes (auto) | Basic | GRBL only | AGPL blocks integration | 2.0/5 |
| 9 | CAMotics | C++ | GPL-2.0+ | N/A (simulator) | N/A | N/A | Simulation only | Verifier, not generator | 1.0/5 |
| 10 | HeeksCNC/libarea | C++ | BSD (libarea) | Good (libarea) | HeeksCNC only | Good (libarea) | Via Python scripts | libarea FFI viable | 2.7/5 |
| 11 | DXF2GCODE | Python | GPL-3.0 | Excellent | Manual (breaking layer) | Basic | Configurable | CLI subprocess | 2.5/5 |
| 12 | PathCAM | C# | GPL-3.0+ | Good | Yes (auto) | None/Limited | GRBL only | C# -- poor interop | 1.3/5 |

---

## Top Picks

### Tier 1: Recommended for Integration

**1. Kiri:Moto (Avg 3.8/5) -- Best overall integration candidate**
- MIT license (most favorable for commercial use)
- Headless Node.js CLI and Engine API for programmatic access
- Active development through March 2026 with 13+ year history
- Profile, tab, and pocket support all present
- Gemini rated it 4.5/5 as "one of the most feature-complete, decoupled engines"
- **Trade-off:** Requires Node.js runtime alongside Rust binary

**2. DerpCAM (Avg 2.7/5) -- Best feature alignment for 2D nesting**
- Purpose-built for exactly the 2.5D use case: profiles, tabs, pockets
- Python API for programmatic G-code generation (not just GUI)
- Trochoidal milling and rest machining -- advanced features rare in open-source
- Claude and Codex both rated it highly (3/5, 4/5); Gemini undervalued it at 1/5
- **Trade-off:** Single developer, GPL-3.0, limited G-code dialect support

**3. libarea (from HeeksCNC) (Avg 2.7/5) -- Best for native Rust FFI**
- BSD-licensed C++ library -- the most integration-friendly IP position
- Small, focused (~23K LOC) library for exactly profile + pocket operations
- Proven algorithms (used by FreeCAD's CAM workbench)
- Can be compiled and called from Rust via `bindgen` or `cxx`
- **Trade-off:** Unmaintained since 2020; no tab support; needs modernization

### Tier 2: Useful as Reference or Subprocess

**4. FreeCAD CAM (Avg 3.0/5)** -- Most feature-complete but heaviest dependency. Use as external process via Python scripting when maximum compatibility is needed. `ocp-freecad-cam` project demonstrates feasibility.

**5. DXF2GCODE (Avg 2.5/5)** -- Good reference for cutter compensation (G41/G42) and route optimization. The `headless-dxf2gcode` fork adds CLI capability. Very practical for pure 2D contour workflows.

**6. PyCAM (Avg 2.3/5)** -- Useful reference for automatic bridge/tab placement algorithms and parallel toolpath computation.

### Not Recommended

- **BlenderCAM (1.7/5):** Requires full Blender; overkill for 2D
- **OpenBuilds CAM (2.0/5):** AGPL license blocks integration
- **jscut (2.0/5):** Abandoned with buggy tab support
- **CAMotics (1.0/5):** Simulator only, not a CAM generator (though useful for verification)
- **PathCAM (1.3/5):** C# tech stack, no pocketing, uncertain maintenance
- **OpenCAMLib (2.3/5):** Great architecture but wrong algorithms (3D surface, not 2D)

---

## Recommended Approach for Rust Integration

All three models converge on a similar strategy. There are two viable paths:

### Path A: Native Rust Implementation (Recommended)

Implement core CAM algorithms natively in Rust, using open-source projects as references:

1. **Use the Clipper2 library** (`clipper2` Rust crate) for polygon offsetting -- this is the same foundation used by libarea, PathCAM, jscut, and others.
2. **Study DerpCAM and libarea** for algorithm reference:
   - Profile offsetting with inside/outside contour support
   - Tab insertion as a geometry operation (split contour, insert bridge geometry)
   - Pocket clearing using offset-based contour strategy with optional zigzag
3. **Implement tab placement** natively -- split the profile contour at tab locations, raise Z to tab height.
4. **Generate G-code directly** with a simple, customizable post-processor (G-code generation itself is straightforward string formatting).
5. **Optionally use CAMotics** as a verification/simulation tool for generated G-code.

**Why this path:** Avoids heavy external dependencies, keeps everything in Rust for performance, and the MIT/BSD-licensed references (Kiri:Moto for algorithms, libarea for C++ FFI patterns) provide the cleanest IP position.

### Path B: Subprocess Integration (Quick Start)

If you need CAM output quickly before native implementation:

1. **Kiri:Moto via Node.js** -- serialize nested sheets as JSON, run headless Kiri CLI, receive G-code. MIT license. Best for full profile + tab + pocket support.
2. **DXF2GCODE via Python** -- output DXF from nester, shell out to `dxf2gcode` CLI, get G-code. Best for pure profile/contour workflows with cutter compensation.

### Path C: C++ FFI (Middle Ground)

1. Compile **libarea** (BSD license) and wrap via Rust `cxx` or `bindgen`.
2. Get fast offset/pocket generation from proven C++ code.
3. Implement tab placement and G-code emission natively in Rust.
4. **Trade-off:** Inherits unmaintained C++ code that may need modernization.

---

## Key Insights from Cross-Model Analysis

1. **No plug-and-play Rust solution exists.** All 12 projects are in Python, JavaScript, C++, or C#. Native Rust CAM implementation or subprocess bridging is required.

2. **Licensing matters enormously.** Only Kiri:Moto (MIT) and libarea (BSD) offer permissive licenses suitable for commercial integration without copyleft obligations. All others are GPL/AGPL.

3. **The Clipper library is the universal foundation.** Nearly every project that does 2D polygon offsetting uses Clipper or a derivative. The `clipper2` Rust crate provides this same capability natively.

4. **Tab placement is the hardest feature to find.** Many projects lack it entirely or implement it poorly. This is the operation most likely to require custom implementation.

5. **2D CAM is fundamentally simpler than 3D CAM.** For a 2D/2.5D nesting use case, the required algorithms (contour offsetting, zigzag/offset pocketing, tab insertion) are well-understood and implementable without the complexity of 3D surface machining.

6. **Models disagreed most on DerpCAM and BlenderCAM.** Gemini significantly undervalued DerpCAM (1/5 vs 3-4/5 from others) and the models disagreed on BlenderCAM's tab support. These disagreements appear to stem from different depths of investigation into project documentation.

---

## Combined Sources

### Primary Projects
- [FreeCAD GitHub](https://github.com/FreeCAD/FreeCAD) | [CAM Module](https://github.com/FreeCAD/FreeCAD/tree/main/src/Mod/CAM)
- [BlenderCAM GitHub](https://github.com/vilemduha/blendercam) | [Website](https://blendercam.com/)
- [PyCAM GitHub](https://github.com/SebKuzminsky/pycam) | [Website](https://pycam.sourceforge.net/)
- [OpenCAMLib GitHub](https://github.com/aewallin/opencamlib) | [Docs](https://opencamlib.readthedocs.io/)
- [DerpCAM GitHub](https://github.com/kfoltman/DerpCAM)
- [Kiri:Moto GitHub](https://github.com/GridSpace/grid-apps) | [Website](https://grid.space/kiri/) | [API Docs](https://docs.grid.space/kiri-moto/apis/)
- [jscut GitHub](https://github.com/tbfleming/jscut) | [Website](https://jscut.org/)
- [OpenBuilds CAM GitHub](https://github.com/OpenBuilds/OpenBuilds-CAM) | [Website](https://cam.openbuilds.com/)
- [CAMotics GitHub](https://github.com/CauldronDevelopmentLLC/CAMotics) | [Website](https://camotics.org/)
- [HeeksCNC GitHub](https://github.com/Heeks/heekscnc) | [libarea GitHub](https://github.com/Heeks/libarea)
- [DXF2GCODE SourceForge](https://sourceforge.net/projects/dxf2gcode/) | [headless fork](https://github.com/holoujak/headless-dxf2gcode)
- [PathCAM GitHub](https://github.com/xenovacivus/PathCAM)

### Related Tools & Libraries
- [ocp-freecad-cam](https://github.com/voneiden/ocp-freecad-cam) -- FreeCAD CAM Python API wrapper
- [Clipper2](https://github.com/ArcInstitute/clipper2) -- Polygon clipping/offsetting (Rust crate available)
- [TPL Language](https://tplang.org/) -- CAMotics scripting language
