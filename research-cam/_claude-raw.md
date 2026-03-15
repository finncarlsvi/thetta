# Open-Source CAM Software Research for Thetta CNC Nesting Application

**Date:** 2026-03-09
**Purpose:** Evaluate 12 open-source CAM projects for integration into a Rust-based CNC nesting application as a post-processing step. Required operations: profile cuts, tab placement, and pocket cuts for a 3-axis CNC router/mill.

---

## 1. FreeCAD Path/CAM Workbench

### Project Overview
FreeCAD's CAM Workbench (formerly "Path Workbench") is the integrated CAM module within FreeCAD, the popular open-source parametric 3D modeler. It generates G-code toolpaths from 3D models for CNC machines including mills, lathes, and laser cutters.

- **Primary Language:** C++ core with Python scripting layer (CAM workbench is mostly Python)
- **License:** LGPL v2.1 (FreeCAD core)
- **Source:** https://github.com/FreeCAD/FreeCAD (CAM module at `src/Mod/CAM/`)

### Activity & Maturity
- **Last Commit:** Active daily; weekly development builds through February 2026
- **GitHub Stars:** ~28,700 (entire FreeCAD repo)
- **Contributors:** Hundreds (FreeCAD overall); notable CAM contributors in 2026 include @Connor9220, @Daniel-Khodabakhsh, @tarman3, @petterreinholdtsen
- **Release History:** FreeCAD 1.0 released 2024; weekly dev builds in 2026. The CAM workbench has been part of FreeCAD for many years and is one of the most actively maintained open-source CAM solutions.
- **Community Size:** Very large. Active forum, extensive documentation, thousands of users.

### Profile Cut Support
Excellent. The CAM Workbench has dedicated Profile and Contour operations. Supports inside and outside contours. Climb vs. conventional milling is configurable per operation. The Profile operation can be applied to faces, edges, or loops of a 3D model. Also supports an "Adaptive" clearing/profiling operation for high-efficiency machining.

### Tab Support
Yes. Tabs are implemented as a "Dressup" that can be applied to any profile operation. Path > Path Dressup > Tag automatically creates 4 tabs. Tab parameters (width, height, angle) are configurable in Edit > Preferences > CAM > Dressups. Automatic placement with manual override capability.

### Pocket Cut Support
Yes. Multiple pocket strategies: Pocket Shape (2.5D), Pocket 3D, Adaptive clearing. Zigzag and offset/contour patterns. Climb vs. conventional milling configurable. The Adaptive operation provides trochoidal/high-efficiency clearing. Forum users report 2.5D pocketing works well; 3D pocketing is less reliable.

### G-code Output
Extensive post-processor system. FreeCAD uses an internal "machine-agnostic" G-code dialect, then translates via post-processors. Supported dialects include LinuxCNC, GRBL, Fanuc, HAAS, Heidenhain, Mitsubishi, and more. Post-processors are Python scripts and are fully customizable. New post-processors can be written following documented conventions.

### Integration Potential
FreeCAD can run headless (without GUI) via Python scripting. The App/Gui separation means CAM operations can be scripted without Qt or graphics. There is a separate project `ocp-freecad-cam` (voneiden/ocp-freecad-cam) that exposes FreeCAD's Path workbench as a fluent Python API. However, FreeCAD is a massive dependency (~2GB+). Extracting just the CAM algorithms would be extremely difficult due to deep integration with FreeCAD's document model, Part module, and OpenCASCADE. Using it as a subprocess or Python API is more realistic than embedding.

### Strengths for Our Use Case
- Most feature-complete open-source CAM solution available
- Excellent profile, pocket, and tab support
- Extensive post-processor library
- Can be scripted headlessly via Python
- Very active development with large community
- Well-tested in production by many users

### Weaknesses for Our Use Case
- Enormous dependency (FreeCAD is a full 3D CAD/CAM application)
- Python-based CAM algorithms are not easily extracted into Rust
- Requires FreeCAD's document model and Part module; cannot use CAM in isolation
- Performance overhead of running FreeCAD as a subprocess
- Complex build system; not designed as an embeddable library
- GPLv2/LGPL licensing may be a concern depending on distribution model

### Overall Rating: 3/5
The most capable option feature-wise, but the integration burden is very high. Best suited if you can accept calling FreeCAD as an external process or via Python subprocess rather than embedding algorithms directly.

---

## 2. BlenderCAM / FabexCNC

### Project Overview
FabexCNC (formerly BlenderCAM, renamed due to Blender trademark requirements) is an open-source CAM extension for Blender that generates toolpaths and G-code from 3D models or 2D curves. It has been used for milling projects since 2012.

- **Primary Language:** Python (Blender addon)
- **License:** GPLv3
- **Source:** https://github.com/vilemduha/blendercam
- **Website:** https://blendercam.com/

### Activity & Maturity
- **Last Commit:** November 2025 (repository updated)
- **GitHub Stars:** ~539
- **Forks:** ~129
- **Contributors:** Multiple; originally by Vilem Novak, now maintained by Alain Pelletier and team (15+ credited contributors)
- **Release History:** Latest release v1.0.68. Active development since 2012.
- **Community Size:** Matrix chat room, active issues on GitHub, Carbide 3D community discussions.

### Profile Cut Support
Yes. Profile is one of the core milling strategies. Supports 2D, 3D, and multi-axis machining strategies. Contour following of curves and edges. Climb vs. conventional not explicitly documented in search results but likely configurable given the maturity of the toolpath system.

### Tab Support
Not clearly documented. Profile operations exist but explicit holding tab support was not prominently featured in documentation or search results. May need to be handled manually through curve modifications. This is a notable gap compared to dedicated 2.5D CAM tools.

### Pocket Cut Support
Yes. Pocket is a core milling strategy. Supports various clearing patterns. The system can generate paths from 3D objects or 2D depth images (height maps). Includes collision detection and simulation.

### G-code Output
Supports G-code export. Post-processor customization available but details on supported dialects are less documented than FreeCAD. The focus appears to be on general-purpose G-code output.

### Integration Potential
Poor for our use case. Requires Blender 4.2.1+ as the host application. The addon is deeply tied to Blender's render engine framework (FABEX_ENGINE) and scene graph. Extracting algorithms would require significant reverse engineering. Cannot be used headlessly without Blender's Python environment.

### Strengths for Our Use Case
- Mature project with 14+ years of development
- Full 3D machining capability (more than we need, but robust)
- Active development and community
- Good simulation and preview capabilities

### Weaknesses for Our Use Case
- Requires Blender as a dependency (even larger than FreeCAD for our purposes)
- Deeply coupled to Blender's 3D engine and scene model
- Tab support appears weak or absent
- Not designed as a library; extraction of algorithms would be very difficult
- GPLv3 license
- Overkill for 2D/2.5D operations

### Overall Rating: 1/5
Too tightly coupled to Blender to be useful for integration. The 3D focus is overkill for our 2D nesting + CAM needs, and the lack of clear tab support is a significant gap.

---

## 3. PyCAM

### Project Overview
PyCAM is a toolpath generator for 3-axis CNC machining. It loads 3D models (STL) or 2D contour models (DXF, SVG) and generates G-code compatible with LinuxCNC and other controllers.

- **Primary Language:** Python
- **License:** GPLv3
- **Source:** https://github.com/SebKuzminsky/pycam
- **Website:** https://pycam.sourceforge.net/

### Activity & Maturity
- **Last Commit:** Sporadic; issues opened in Feb 2025, but development has been inconsistent
- **GitHub Stars:** ~289
- **Forks:** ~85
- **Contributors:** Small team; primarily SebKuzminsky
- **Release History:** v0.5.1 (2012), v0.6.0 (2017), v0.6.4 (latest on PyPI). There was a 5-year pause between 2012-2017. The codebase is described as being "modernized."
- **Community Size:** Small. SourceForge forums, limited GitHub activity.

### Profile Cut Support
Yes. Supports contour/follow strategies (marked as experimental since v0.4). Push Cutter mode removes material in layers. Drop Cutter for surface following. Conventional/climb milling options available. Engraving strategy for 2D contour following.

### Tab Support
Yes. "Support bridges" can be added to models, with automatic distribution along the outline (corners or edges) of parts. Width, height, and length are configurable. This is a notable feature.

### Pocket Cut Support
Limited. A "very simple pocketing mode for 2D models" was added. Basic functionality only. Not comparable to the pocket strategies in FreeCAD or commercial tools.

### G-code Output
Generates G-code for LinuxCNC. Post-processor customization appears limited compared to FreeCAD. Primarily targets LinuxCNC/GRBL.

### Integration Potential
Moderate. The master branch allows scripted (non-interactive) usage of PyCAM, though documentation is lacking. Being pure Python, algorithms could potentially be studied and reimplemented in Rust. The codebase is reasonably modular. Multi-processing support for parallel toolpath calculation. However, the code is aging and inconsistently maintained.

### Strengths for Our Use Case
- Has tab/bridge support with automatic placement
- Supports 2D contour models (DXF, SVG) directly
- Scripted/headless usage possible
- Multi-core parallel processing
- Pure Python makes algorithm study feasible

### Weaknesses for Our Use Case
- Inconsistent development; appears semi-abandoned
- Very basic pocketing support
- Aging codebase with GTK2/3 GUI dependencies
- Limited post-processor options
- Performance likely poor for production use (pure Python)
- GPLv3 license

### Overall Rating: 2/5
The tab support and 2D contour focus are relevant, but the project's inconsistent maintenance and basic pocketing make it a risky choice. Better as a reference for algorithm study than as a direct integration target.

---

## 4. OpenCAMLib (aewallin/opencamlib)

### Project Overview
OpenCAMLib (OCL) is a C++ library for creating 3D toolpaths for CNC machines. It provides core algorithms for cutter-surface intersection calculations, focusing on drop-cutter and push-cutter (waterline) operations.

- **Primary Language:** C++
- **License:** LGPL (since August 2018)
- **Source:** https://github.com/aewallin/opencamlib
- **Bindings:** Python (PyPI), Node.js, browser (WASM)

### Activity & Maturity
- **Last Commit:** Early-to-mid 2024 (approximately)
- **GitHub Stars:** ~473
- **Forks:** ~155
- **Contributors:** ~10 or fewer
- **Release History:** Latest release 2023.01.11. No releases since. Considered potentially discontinued by package health analyzers.
- **Community Size:** Small. LinuxCNC forum discussions, academic references.

### Profile Cut Support
Limited for 2D. OCL is primarily focused on 3D surface machining (drop-cutter for surface following, push-cutter for waterline contouring). For 2D profile cutting, it offers OpenVoronoi, a separate Voronoi diagram library for 2D offsetting of line segments. However, 2D profiling is not a primary focus and the Voronoi-based approach is still somewhat experimental.

### Tab Support
No built-in tab support. OCL is a low-level algorithm library, not a complete CAM system. Tab placement would need to be implemented externally.

### Pocket Cut Support
Experimental. Medial-axis pocketing is described as "just a demonstration" that could be developed into an adaptive pocketing algorithm. Terrace roughing (waterline + pocket) is described as "not implemented yet but doable." No production-ready pocket strategies.

### G-code Output
No G-code generation. OCL produces toolpath point data; G-code generation must be handled externally.

### Integration Potential
Best of the 12 projects for library integration. LGPL license is the most permissive for commercial use. C++ with CMake build system and `find_package(OpenCAMLib)` support. Pre-compiled libraries available. Python/Node.js/WASM bindings. Could be called from Rust via FFI. However, the algorithms focus on 3D surface machining, not the 2D profile/pocket operations we need.

### Strengths for Our Use Case
- LGPL license (most integration-friendly)
- Designed as a library, not a GUI application
- C++ with clean CMake integration
- FFI-callable from Rust
- Academically rigorous algorithms

### Weaknesses for Our Use Case
- Focused on 3D surface machining, not 2D profile/pocket
- No tab support
- No G-code generation
- Experimental/incomplete pocketing
- Declining maintenance (no releases since 2023)
- Waterline algorithm has known bugs

### Overall Rating: 2/5
Excellent architecture for integration (it's actually a library!), but the wrong algorithms for our use case. We need 2D profile cutting, tabs, and pocketing -- OCL focuses on 3D surface machining. The LGPL license and C++ FFI potential are very attractive, but the feature gap is too large.

---

## 5. DerpCAM (kfoltman/DerpCAM)

### Project Overview
DerpCAM is a 2.5D CAM package and toolpath generator specifically designed for hobby 3-axis CNC routers and mills. It targets LinuxCNC and GRBL controllers.

- **Primary Language:** Python 3
- **License:** GPLv3
- **Source:** https://github.com/kfoltman/DerpCAM
- **GUI:** Qt5

### Activity & Maturity
- **Last Commit:** April 28, 2025
- **GitHub Stars:** ~26
- **Forks:** Small number
- **Contributors:** Primarily Krzysztof Foltman (single developer)
- **Release History:** No formal releases; run from source directory
- **Community Size:** Very small. V1E forum mentions.

### Profile Cut Support
Yes. Outside/inside slot milling is a core feature. Supports wide slots. DXF import from LibreCAD. Basic but functional 2.5D profiling.

### Tab Support
Yes. "Outside/inside slot milling with optional tabs" is listed as a key feature. Tabs are part of the core slot milling operation. Configuration details not extensively documented but present.

### Pocket Cut Support
Yes, with notable advanced features. Supports trochoidal paths for efficient clearing. Rest machining -- refining a coarsely-milled pocket or outline using a finer tool. This is a sophisticated feature typically found in commercial CAM software.

### G-code Output
Generates G-code for LinuxCNC and GRBL. Does NOT support commercial machine vendor dialects. Limited post-processor customization. The README explicitly warns about this limitation.

### Integration Potential
Moderate-to-good. Has a Python API for generating G-code from data instead of a CAD drawing (not just GUI). Wall profile support available via Python API only. Being pure Python, algorithms are readable and could be studied for reimplementation. No installation needed; runs from directory. However, small community means bugs may go unfixed.

### Strengths for Our Use Case
- Purpose-built for exactly our use case: 2.5D profile cuts, tabs, and pockets
- Trochoidal milling and rest machining (advanced features)
- Python API for programmatic use (not just GUI)
- Simple, focused codebase
- Active development (as of April 2025)

### Weaknesses for Our Use Case
- Very small community (essentially one developer)
- Not meant for professional use (author's own disclaimer)
- Limited G-code dialect support (LinuxCNC/GRBL only)
- GPLv3 license
- No formal releases or versioning
- DXF import limited to LibreCAD output
- Small star count suggests limited real-world validation

### Overall Rating: 3/5
The best feature-to-complexity ratio for our specific use case. Profile cuts with tabs, pocket clearing with trochoidal paths, and a Python API are exactly what we need. The single-developer risk and small community are concerning, but the focused codebase makes it an excellent study reference for implementing these algorithms in Rust.

---

## 6. Kiri:Moto (GridSpace/grid-apps)

### Project Overview
Kiri:Moto is a browser-based slicer and toolpath generator for 3D Printing, CNC milling, and laser cutting. It runs entirely in the browser using JavaScript, WebWorkers, and WASM.

- **Primary Language:** JavaScript
- **License:** MIT
- **Source:** https://github.com/GridSpace/grid-apps
- **Website:** https://grid.space/kiri/

### Activity & Maturity
- **Last Commit:** March 3, 2026 (very active)
- **GitHub Stars:** ~817
- **Forks:** ~183
- **Contributors:** Primarily Stewart Allen (GridSpace founder), with community contributions
- **Release History:** Over 13 years of development (since 2013). Regular releases on GitHub.
- **Community Size:** Active forum (forum.grid.space), Discord, used in classrooms and makerspaces. Integrated with Onshape.

### Profile Cut Support
Yes. Supports "Outline" operations for profile/contour cutting. Rough and contour operations available. Multiple operation types can be combined per job.

### Tab Support
Yes. Tab feature exists for holding parts during cutting. However, users report tabs only work on outside contours, not inside contours. Tab placement and configuration available.

### Pocket Cut Support
Yes. Pocket operations supported with contour option. However, forum reports indicate some issues: toolpaths extending beyond pocket boundaries, and difficulty with combined pocket + contour operations. These suggest the pocket implementation may be less mature than profile cutting.

### G-code Output
Customizable G-code post-processors. Supports GRBL and other controllers. G-code macros for start, tool change, and end operations. RML dialect support. File extension customizable. Output is unbundled by operation for manual sequencing.

### Integration Potential
Good. Kiri:Moto offers multiple integration approaches:
1. **JavaScript Engine API** -- a script include that performs slicing/G-code generation asynchronously via web workers
2. **IFrame Messaging API** -- for embedding in web pages
3. **CLI** -- command-line interface for headless operation

MIT license is very favorable for integration. The JavaScript codebase could potentially be run via Node.js from Rust, or algorithms could be studied and reimplemented. The Engine API is particularly interesting for programmatic use.

### Strengths for Our Use Case
- MIT license (most integration-friendly along with OCL's LGPL)
- Multiple programmatic APIs (Engine, CLI, IFrame)
- Active development through March 2026
- Written from scratch (no dependencies on other CAM codebases)
- Browser-based = potentially embeddable via Node.js/WASM
- Good community and documentation

### Weaknesses for Our Use Case
- JavaScript-based (not ideal for Rust FFI; would need Node.js runtime or WASM bridge)
- Pocket operations have reported bugs
- Tab support limited to outside contours only
- Primarily designed for STL/3D model input, not 2D contours
- CNC mode is secondary to 3D printing mode
- Browser-oriented architecture may not suit server-side/CLI batch processing well

### Overall Rating: 3/5
The MIT license and multiple APIs make this attractive for integration. The CNC mode is functional but secondary to the 3D printing focus. Tab limitations and pocket bugs are concerns. The JavaScript Engine API is a unique advantage for programmatic access, though the JavaScript-to-Rust bridge adds complexity.

---

## 7. jscut

### Project Overview
jscut is a browser-based CAM tool that converts SVG files to CNC cutting paths and G-code. It focuses on 2.5D operations.

- **Primary Language:** JavaScript + C++ (compiled to WASM via Emscripten)
- **License:** GPLv3
- **Source:** https://github.com/tbfleming/jscut
- **Website:** https://jscut.org/

### Activity & Maturity
- **Last Commit:** Inactive for several years
- **GitHub Stars:** ~324
- **Forks:** ~143
- **Contributors:** Primarily tbfleming (single developer)
- **Release History:** No recent releases. Author states: "I no longer add new features to jscut. LaserWeb4 is the successor."
- **Community Size:** Defunct. Open issues remain unresolved.

### Profile Cut Support
Yes. Supports "Inside" and "Outside" cut operations, which correspond to inside and outside profile/contour cutting. SVG-based input.

### Tab Support
Partially. Tab support was requested (issue #17) and partially implemented. The implementation had issues: "the tool path unpredictably changes direction on cut-up path segments, sending the cutter through areas that shouldn't be cut." Tabs were added but with reliability concerns.

### Pocket Cut Support
Yes. "Pocket" is one of the core operations. C++ implementation compiled to WASM includes `hspocket.cpp`. V Pocket operation also supported. Strategies not well documented.

### G-code Output
Basic G-code output. Limited post-processor customization. Primarily generates generic G-code.

### Integration Potential
Low. The project is abandoned. The C++ core (compiled to Emscripten/WASM) could theoretically be studied, but the codebase is small and dated. SVG-only input is limiting. GPLv3 license.

### Strengths for Our Use Case
- SVG-based 2D input aligns with our 2D nesting output
- Simple, focused feature set
- C++ core algorithms could be studied

### Weaknesses for Our Use Case
- Abandoned/unmaintained (author moved to LaserWeb4)
- Tab implementation is buggy
- GPLv3 license
- Limited post-processor support
- No API for programmatic use
- Small, dated codebase with known issues

### Overall Rating: 1/5
Abandoned project with buggy tab support. Not suitable for integration. May have some educational value for studying the C++ pocket/profile algorithms, but there are better references available.

---

## 8. OpenBuilds CAM

### Project Overview
OpenBuilds CAM is a web-based CAM application for converting SVG, DXF, and Bitmap files to G-code for CNC, Laser, Plasma, and Drag Knife machines. Part of the OpenBuilds ecosystem.

- **Primary Language:** JavaScript
- **License:** AGPL-3.0
- **Source:** https://github.com/OpenBuilds/OpenBuilds-CAM
- **Website:** https://cam.openbuilds.com/

### Activity & Maturity
- **Last Commit:** March 4, 2025
- **GitHub Stars:** ~775
- **Forks:** ~337
- **Contributors:** OpenBuilds team
- **Release History:** Regular updates; latest changelog entry December 2024.
- **Community Size:** Large. Part of the OpenBuilds ecosystem, described as "the world's largest online community" for CNC.

### Profile Cut Support
Yes. Supports vector outside/inside paths for profile cutting. Part of the core functionality for CNC routing.

### Tab Support
Yes. Tab support is documented with community tutorials on "OpenBuilds CAM adding Tabs." Users can add tabs to toolpaths via the GUI.

### Pocket Cut Support
Yes. Pocketing is supported, with bug fixes noted for Z-depth issues on unmerged geometry. Basic pocket strategies.

### G-code Output
Generates G-code primarily for GRBL controllers (the OpenBuilds ecosystem standard). Pairs with OpenBuilds CONTROL desktop app for machine communication. Limited to the OpenBuilds/GRBL ecosystem.

### Integration Potential
Low-to-moderate. AGPL-3.0 license is the most restrictive of the group -- requires source disclosure for any software that interacts with it over a network. JavaScript codebase. The web-based architecture is not designed for library extraction. Primarily a GUI application with no documented API for headless use.

### Strengths for Our Use Case
- Large community and well-tested for hobby CNC
- Tab support included
- Active development
- Good user experience for end users

### Weaknesses for Our Use Case
- AGPL-3.0 license is very restrictive for integration
- No API or headless mode
- JavaScript GUI application, not a library
- Limited to GRBL ecosystem
- Web-based architecture not suited for embedding
- No documented programmatic interface

### Overall Rating: 1/5
The AGPL license alone makes this unsuitable for integration into a commercial/proprietary application. Beyond licensing, there's no API or library interface -- it's purely a web GUI application.

---

## 9. CAMotics

### Project Overview
CAMotics is an open-source 3-axis CNC G-code simulator with TPL (Tool Path Language) scripting for programmatic toolpath generation. It is primarily a simulation tool, not a CAM tool.

- **Primary Language:** C++
- **License:** GPLv2+
- **Source:** https://github.com/CauldronDevelopmentLLC/CAMotics
- **Website:** https://camotics.org/

### Activity & Maturity
- **Last Commit:** Approximately early 2025 (issues filed April 2025)
- **GitHub Stars:** Not explicitly found, but the project is well-established
- **Contributors:** Small team; maintained by CauldronDevelopmentLLC
- **Release History:** Latest release v1.2.0. Well-established project with mature codebase (estimated 24 person-years of effort by COCOMO).
- **Community Size:** Moderate. Used with Buildbotics CNC controller, V1E community, OpenBuilds.

### Profile Cut Support
No native CAM toolpath generation. CAMotics is a simulator -- it reads G-code and simulates the cutting result. The TPL (Tool Path Language) provides JavaScript-based scripting for generating G-code programmatically, but it's a programming interface, not a CAM system that generates toolpaths from geometry.

### Tab Support
TPL added tab support for 2D profile cuts in the DXF TPL library. This is script-based, not automatic.

### Pocket Cut Support
No native pocket generation. CAMotics' roadmap lists "CAM routines (pocketing, offsetting, arrays, etc.)" as planned features, meaning they don't exist yet.

### G-code Output
TPL outputs G-code. Implements LinuxCNC O-codes. The simulation engine parses and verifies G-code comprehensively. But the output is only through TPL scripting, not automated CAM.

### Integration Potential
Moderate as a simulation/verification tool, not as a CAM tool. The C++ codebase could be used for G-code simulation and verification after toolpath generation by another tool. TPL is JavaScript-based and could be used for scripted toolpath generation, but this is manual programming, not automatic CAM.

### Strengths for Our Use Case
- Excellent G-code simulation and verification
- Could be used as a post-processing verification step
- C++ codebase
- TPL scripting for programmatic G-code generation
- Export cut workpiece to STL

### Weaknesses for Our Use Case
- Not a CAM tool -- no automatic toolpath generation from geometry
- No profile, pocket, or tab algorithms (only simulation)
- TPL is manual scripting, not automatic CAM
- GPLv2+ license
- Would only be useful for verification, not generation

### Overall Rating: 1/5
CAMotics is a simulator, not a CAM tool. It could be useful as a verification step (simulate generated G-code before sending to machine), but it cannot generate toolpaths from geometry. Misaligned with our core requirements.

---

## 10. HeeksCNC / libarea

### Project Overview
HeeksCNC is a CAM add-on for HeeksCAD, using libarea (a C++ library for pocketing and profiling) and OpenCAMLib. While HeeksCNC itself is inactive, libarea lives on as the pocketing/profiling engine used by FreeCAD's CAM workbench.

- **Primary Languages:** C++ (libarea), Python (HeeksCNC scripts)
- **License:** libarea: New BSD License (permissive); HeeksCNC: GPLv3
- **Source:**
  - https://github.com/Heeks/heekscnc (HeeksCNC - last updated Oct 2021)
  - https://github.com/Heeks/libarea (libarea - last updated Jan 2020)
  - https://github.com/danheeks/HeeksCAM (combined, never released)

### Activity & Maturity
- **HeeksCNC Last Commit:** October 2021 (inactive)
- **libarea Last Commit:** January 2020 (inactive)
- **HeeksCNC GitHub Stars:** ~103 / Forks: ~47 / Contributors: ~10
- **libarea GitHub Stars:** ~41 / Forks: ~22 / Contributors: ~20 (Open Hub: 7 with 122 commits)
- **Release History:** No formal releases in years. HeeksCAM never reached release state.
- **Community Size:** Defunct. Legacy influence through FreeCAD.

### Profile Cut Support
Yes (via libarea). libarea provides profiling operations -- tool offset calculation for following 2D contours. Inside and outside profiling. The algorithms are the same ones used by FreeCAD's CAM workbench (inherited).

### Tab Support
Not clearly documented in libarea itself. HeeksCNC may have handled tabs at the Python script level. Tab support would likely need to be added externally.

### Pocket Cut Support
Yes (via libarea). Pocketing is a core function of libarea. The library computes offset-based pocket toolpaths for 2D regions. Used by FreeCAD for its pocket operations. The underlying algorithm uses the Clipper library for polygon operations.

### G-code Output
HeeksCNC generated G-code via Python scripts (post.py). Machine-specific formatting through user-editable scripts. Supported EMC2/LinuxCNC and others. But this is all through the HeeksCNC Python layer, not libarea itself.

### Integration Potential
**libarea is the most interesting component here.** It is a focused C++ library with BSD license for pocketing and profiling operations. ~23,000 lines of code. Uses the Clipper library (also permissively licensed) for polygon operations. Could be called from Rust via FFI. The BSD license is very favorable. However, it's unmaintained (last commit 2020) and would need modernization.

### Strengths for Our Use Case
- libarea: BSD license (permissive, integration-friendly)
- libarea: Small, focused C++ library for exactly profile + pocket operations
- Proven algorithms (used by FreeCAD)
- Could be wrapped via Rust FFI
- Reasonable code size (~23K LOC)

### Weaknesses for Our Use Case
- Unmaintained since 2020
- No tab support in libarea
- Limited documentation
- Aging C++ code may need modernization
- HeeksCNC (the user-facing part) is defunct
- No active community for bug fixes

### Overall Rating: 3/5
libarea is a hidden gem for our specific needs. BSD-licensed C++ library for profiling and pocketing, small enough to study and potentially wrap via FFI. The lack of maintenance is a risk, but the algorithms are proven (FreeCAD uses them). No tab support means we'd need to implement that ourselves. Best suited as a reference implementation or FFI target for profile/pocket algorithms.

---

## 11. DXF2GCODE

### Project Overview
DXF2GCODE converts 2D drawings (DXF, PDF, PS) to CNC-compatible G-code. It's a focused tool for 2D contour cutting with support for milling, lathe, and drag knife operations.

- **Primary Language:** Python 3
- **License:** GPLv3
- **Source:** https://sourceforge.net/projects/dxf2gcode/ (primary), various GitHub forks
- **Dependencies:** Python 3, PyQt5, PyOpenGL, configobj

### Activity & Maturity
- **Last Activity:** Development build DXF2GCODE-2024.11.2 available. SourceForge shows last update 2025-12-31.
- **Contributors:** ~7 (Christian Kohloffel, Jean-Paul Schouwstra, John Bradshaw, Vinzenz Schulz, Robert Lichtenberger, Damian Wrobel, Daniel Luginbuhl)
- **Release History:** Latest stable: 2022.2.26. Development builds through November 2024.
- **Community Size:** Small but active. Available in Fedora/Debian repositories. SourceForge-based.

### Profile Cut Support
Yes. This is the primary function. Contour cutting with cutter compensation (G41 left, G42 right, G40 no compensation). Shape enabling/disabling/reversing. Start point shifting. Route optimization for minimizing G0 moves. Very focused on 2D contour cutting.

### Tab Support
Yes. Tabs are supported through a "breaking layer" mechanism: draw rectangles on a special layer in the DXF file, and where they intersect a cutting contour exactly twice, a tab is created. This is a manual-but-CAD-integrated approach. The user draws tabs in the DXF source file.

### Pocket Cut Support
Yes, basic. "Pocket Mill" option available for inside compensation paths. Not as sophisticated as dedicated CAM tools -- limited to simple offset-based pocketing.

### G-code Output
Fully adjustable post-processor. EMC2/LinuxCNC integration. Supports milling, lathe, and drag knife machine types. Route optimization for efficient G0 moves. Multiple language support.

### Integration Potential
Moderate. Python codebase is readable. A headless fork exists (holoujak/headless-dxf2gcode) with CLI operation, YAML configuration, automatic contour ordering (inner shapes first), and G-code optimization. This fork is particularly interesting for integration -- it adds the headless/batch processing capability we'd need. However, still Python-based and GPLv3.

### Strengths for Our Use Case
- Purpose-built for 2D contour cutting (our primary use case)
- Tab support via breaking layers
- Cutter compensation (G41/G42)
- Route optimization for efficient moves
- headless-dxf2gcode fork adds CLI/batch capability
- DXF input aligns with common CAD output formats

### Weaknesses for Our Use Case
- Tab placement is manual (drawn in DXF), not automatic
- Basic pocketing only
- GPLv3 license
- Python (performance concerns, not easily embedded in Rust)
- SourceForge-based development (less accessible than GitHub)
- Small community
- Limited pocket strategies

### Overall Rating: 2/5
Good for studying 2D contour cutting algorithms, especially route optimization and cutter compensation. The headless fork is interesting. However, manual tab placement, basic pocketing, and GPLv3 license limit its integration value. Better as a reference than a direct dependency.

---

## 12. PathCAM (xenovacivus/PathCAM)

### Project Overview
PathCAM is a simple tool for generating 2.5D toolpaths to cut shapes from stock material using a CNC router. It supports direct robot connection via GRBL.

- **Primary Language:** C#
- **License:** GPLv3+
- **Source:** https://github.com/xenovacivus/PathCAM
- **GUI:** OpenTK (OpenGL-based)

### Activity & Maturity
- **Last Commit:** Unknown (search results say "under active development" but no recent dates found)
- **GitHub Stars:** ~237
- **Forks:** ~63
- **Contributors:** Primarily Benjamin R. Porter (single developer)
- **Release History:** MSI installer available. No formal version numbering visible.
- **Community Size:** Very small. Occasional issues filed.

### Profile Cut Support
Yes. "Add Perimeter Paths" feature follows edges of objects. Paths are divided into layers based on max cut depth. Two passes per edge: rough cut (bulk removal) and clean cut (exact dimension). Uses Clipper library for polygon offsetting.

### Tab Support
Yes. Tabs appear as orange elements in the GUI that can be moved around. Users can position tabs to hold parts during cutting. Appears to be manual placement.

### Pocket Cut Support
No explicit pocket support documented. The tool focuses on perimeter/profile cutting with layered depth passes. No pocket clearing strategies mentioned.

### G-code Output
Exports .gcode files. Supports GRBL 8.0c+ for direct machine connection. Basic G-code output without extensive post-processor customization.

### Integration Potential
Low. C# codebase on .NET/Mono. Building on Linux requires Mono. Uses OpenTK, Clipper, STLdotNET, Triangle.NET. The architecture is a GUI application, not a library. Extracting algorithms would require porting from C# to Rust. The Clipper usage for polygon offsetting is noteworthy (Clipper has Rust bindings available).

### Strengths for Our Use Case
- Simple 2.5D focus matches our needs
- Tab support included
- Uses Clipper for polygon offsetting (Clipper is available in Rust)
- Clean, understandable codebase

### Weaknesses for Our Use Case
- C# / .NET dependency (poor Rust interop)
- No pocket support
- Single developer, uncertain maintenance status
- GPLv3+ license
- GUI application, not a library
- Basic G-code output only (GRBL)
- No headless/API mode

### Overall Rating: 1/5
Too limited (no pocketing) and wrong technology stack (C#) for our needs. The use of Clipper for polygon offsetting is a useful data point, since Clipper has Rust bindings, but PathCAM itself is not a viable integration target.

---

## Summary Comparison Table

| Project | Language | License | Profile | Tabs | Pocket | G-code | Library/API | Active | Rating |
|---------|----------|---------|---------|------|--------|--------|-------------|--------|--------|
| FreeCAD CAM | Python/C++ | LGPL | Excellent | Yes (auto) | Excellent | Excellent | Headless Python | Very active | 3/5 |
| BlenderCAM/Fabex | Python | GPLv3 | Good | Unclear | Good | Basic | No | Active | 1/5 |
| PyCAM | Python | GPLv3 | Good | Yes (auto) | Basic | Limited | Scripted (WIP) | Semi-active | 2/5 |
| OpenCAMLib | C++ | LGPL | Limited (3D) | No | Experimental | None | Yes (library) | Low | 2/5 |
| DerpCAM | Python | GPLv3 | Good | Yes | Good+ | LinuxCNC/GRBL | Python API | Active | 3/5 |
| Kiri:Moto | JavaScript | MIT | Good | Partial | Buggy | Customizable | JS API/CLI | Very active | 3/5 |
| jscut | JS/C++ | GPLv3 | Good | Buggy | Basic | Basic | No | Abandoned | 1/5 |
| OpenBuilds CAM | JavaScript | AGPL-3.0 | Good | Yes | Basic | GRBL | No | Active | 1/5 |
| CAMotics | C++ | GPLv2+ | No (simulator) | TPL only | No | TPL scripting | Simulation lib | Active | 1/5 |
| HeeksCNC/libarea | C++ | BSD (libarea) | Good | No | Good | Via scripts | libarea=library | Inactive | 3/5 |
| DXF2GCODE | Python | GPLv3 | Good | Manual | Basic | Adjustable | Headless fork | Active | 2/5 |
| PathCAM | C# | GPLv3+ | Good | Manual | No | GRBL | No | Uncertain | 1/5 |

---

## Recommendations for Integration into Thetta

### Tier 1: Best Candidates for Algorithm Study/Extraction

1. **DerpCAM (3/5)** -- The most aligned feature set: 2.5D profile cuts with tabs, pocket clearing with trochoidal paths, and a Python API. The codebase is focused and readable. Study its algorithms for reimplementation in Rust. Key algorithms to extract: profile offsetting with tab insertion, trochoidal pocket clearing, rest machining logic.

2. **libarea from HeeksCNC (3/5)** -- BSD-licensed C++ library specifically for profiling and pocketing. Small enough to wrap via Rust FFI or study for reimplementation. Proven algorithms (used by FreeCAD). Missing tab support would need to be added. The Clipper library dependency has Rust equivalents available.

3. **Kiri:Moto (3/5)** -- MIT license and JavaScript Engine API make this the most integration-friendly from a licensing perspective. The CLI mode could be called as a subprocess. The JavaScript algorithms could be studied and reimplemented. Active development ensures ongoing improvements.

### Tier 2: Useful as Reference/Subprocess

4. **FreeCAD CAM (3/5)** -- Most feature-complete but heaviest dependency. Could be used as an external process via Python scripting for cases where maximum compatibility is needed. The `ocp-freecad-cam` project shows this is feasible. Not suitable for embedding.

5. **DXF2GCODE (2/5)** -- The headless fork is interesting for 2D contour cutting with route optimization. Good reference for cutter compensation and G0 optimization algorithms.

6. **PyCAM (2/5)** -- Useful reference for bridge/tab automatic placement algorithms and parallel toolpath computation.

### Recommended Approach

Given that Thetta is a Rust application, the most practical path is:

1. **Implement core CAM algorithms natively in Rust**, using DerpCAM and libarea as algorithm references
2. **Use the Clipper2 library** (Rust bindings available via `clipper2` crate) for polygon offsetting -- this is the same foundation used by libarea, PathCAM, and others
3. **Implement tab placement** as a geometry operation on the profile toolpath (split contour, insert bridge geometry)
4. **Implement pocket clearing** using offset-based contour strategy (simplest) with optional zigzag
5. **Generate G-code directly** with a simple, customizable post-processor (the G-code generation part is straightforward)
6. **Optionally use CAMotics** as a verification/simulation tool for generated G-code

This approach avoids heavy dependencies, keeps everything in Rust for performance, and the MIT/BSD-licensed references (Kiri:Moto, libarea) provide the cleanest IP position.

---

## Sources

### FreeCAD CAM
- [FreeCAD GitHub](https://github.com/FreeCAD/FreeCAD)
- [FreeCAD CAM Workbench Documentation](https://github.com/FreeCAD/FreeCAD-documentation/blob/main/wiki/CAM_Workbench.md)
- [FreeCAD CAM FAQ](https://wiki.freecad.org/Path_FAQ/en)
- [FreeCAD CAM Post Processor](https://wiki.freecad.org/Path_Post/en)
- [ocp-freecad-cam](https://github.com/voneiden/ocp-freecad-cam)
- [FreeCAD CNC Workflow (Way of Wood)](https://wayofwood.com/freecad-cnc-cad-and-cam-workflow/)

### BlenderCAM / FabexCNC
- [BlenderCAM GitHub](https://github.com/vilemduha/blendercam)
- [FabexCNC Documentation](https://spectralvectors.github.io/blendercam/index.html)
- [BlenderCAM Website](https://blendercam.com/)
- [BlenderCAM DeepWiki](https://deepwiki.com/vilemduha/blendercam/1-overview)

### PyCAM
- [PyCAM GitHub](https://github.com/SebKuzminsky/pycam)
- [PyCAM Website](https://pycam.sourceforge.net/)
- [PyCAM Features](https://pycam.sourceforge.net/features/)
- [PyCAM PyPI](https://pypi.org/project/pycam/)

### OpenCAMLib
- [OpenCAMLib GitHub](https://github.com/aewallin/opencamlib)
- [OpenCAMLib Documentation](https://opencamlib.readthedocs.io/)
- [OpenCAMLib Releases](https://github.com/aewallin/opencamlib/releases)
- [Anders Wallin CAM Blog](https://www.anderswallin.net/cam/)
- [OpenCAMLib on Snyk](https://snyk.io/advisor/python/opencamlib)

### DerpCAM
- [DerpCAM GitHub](https://github.com/kfoltman/DerpCAM)
- [DerpCAM V1E Forum Discussion](https://forum.v1e.com/t/derpcam-open-source-2-5d-cam/47525)

### Kiri:Moto
- [Kiri:Moto GitHub (grid-apps)](https://github.com/GridSpace/grid-apps)
- [Kiri:Moto Website](https://grid.space/kiri/)
- [Kiri:Moto API Documentation](https://docs.grid.space/kiri-moto/apis/)
- [Kiri:Moto CAM Overview](https://docs.grid.space/kiri-moto/CAM/)
- [V1E Kiri:Moto Basics](https://docs.v1e.com/software/kiri/)

### jscut
- [jscut GitHub](https://github.com/tbfleming/jscut)
- [jscut Website](https://jscut.org/)
- [jscut Tab Support Issue](https://github.com/tbfleming/jscut/issues/17)

### OpenBuilds CAM
- [OpenBuilds CAM GitHub](https://github.com/OpenBuilds/OpenBuilds-CAM)
- [OpenBuilds CAM Website](https://cam.openbuilds.com/)
- [OpenBuilds Software](https://software.openbuilds.com/)
- [OpenBuilds Documentation](https://docs.openbuilds.com/doku.php?id=docs:software:cam)

### CAMotics
- [CAMotics GitHub](https://github.com/CauldronDevelopmentLLC/CAMotics)
- [CAMotics Website](https://camotics.org/)
- [TPL Language](https://tplang.org/)
- [CAMotics Manual](https://camotics.org/manual.html)

### HeeksCNC / libarea
- [HeeksCNC GitHub](https://github.com/Heeks/heekscnc)
- [libarea GitHub](https://github.com/Heeks/libarea)
- [HeeksCAM GitHub](https://github.com/danheeks/HeeksCAM)

### DXF2GCODE
- [DXF2GCODE SourceForge](https://sourceforge.net/projects/dxf2gcode/)
- [DXF2GCODE User Guide](https://sourceforge.net/p/dxf2gcode/wiki/User%20Guide%20DXF2GCODE/)
- [headless-dxf2gcode GitHub](https://github.com/holoujak/headless-dxf2gcode)

### PathCAM
- [PathCAM GitHub](https://github.com/xenovacivus/PathCAM)
