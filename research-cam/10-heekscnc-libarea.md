# HeeksCNC / libarea - CAM Research Summary

## Overview
HeeksCNC is a CAM add-on for HeeksCAD, using libarea (a C++ library for pocketing and profiling) and OpenCAMLib. While HeeksCNC itself is inactive and HeeksCAD is defunct, the underlying **libarea** library lives on as the pocketing/profiling engine used by FreeCAD's CAM workbench. libarea is the most interesting component here: a focused, BSD-licensed C++ library for exactly the 2D profile and pocket operations needed in a CNC nesting workflow.

- **Primary Languages:** C++ (libarea), Python (HeeksCNC scripts)
- **License:** libarea: New BSD License (permissive); HeeksCNC: GPLv3
- **Source:**
  - https://github.com/Heeks/heekscnc (HeeksCNC - last updated Oct 2021)
  - https://github.com/Heeks/libarea (libarea - last updated Jan 2020)
  - https://github.com/danheeks/HeeksCAM (combined, never released)

## Activity & Maturity
- **HeeksCNC Last Commit:** October 2021 (Claude); around 2021 (Codex). Inactive.
- **libarea Last Commit:** January 2020 (Claude). Inactive.
- **HeeksCNC GitHub Stars:** ~103 / Forks: ~47 / Contributors: ~10 (Claude)
- **libarea GitHub Stars:** ~41 / Forks: ~22 / Contributors: ~20 (Open Hub: 7 with 122 commits) (Claude)
- **Release History:** No formal releases in years. HeeksCAM never reached release state (Claude).
- **Community Size:** Defunct as standalone project. Legacy influence through FreeCAD (Claude). Gemini: "Abandoned around 2015" for HeeksCNC overall.

## Profile Cut Support
**All models agree: Yes, via libarea.**
- Claude: libarea provides profiling operations -- tool offset calculation for following 2D contours. Inside and outside profiling. Same algorithms used by FreeCAD.
- Gemini: Uses OpenCAMLib and Kurve for profiling.
- Codex: Left/outside, right/inside, on; climb/conventional.

## Tab Support
**Models disagree:**
- Claude: "Not clearly documented in libarea itself." HeeksCNC may have handled tabs at the Python script level.
- Gemini: "Had a basic 2D tagging/tab system."
- Codex: Yes -- Tag objects and profile tag-adding mode, largely manual.

**Note:** Tab support appears to exist at the HeeksCNC (Python) level but not in the libarea C++ library itself. This means tabs would need to be implemented separately if using libarea alone.

## Pocket Cut Support
**All models agree: Yes, via libarea.**
- Claude: Pocketing is a core function of libarea. Computes offset-based pocket toolpaths for 2D regions. Used by FreeCAD. Uses Clipper library for polygon operations.
- Gemini: Uses libarea, "an incredibly fast C++ 2D pocketing algorithm library." Based on Voronoi diagrams.
- Codex: Zig-zag options, entry styles including helical/ramp/plunge.

## G-code Output
- Claude: HeeksCNC generated G-code via Python scripts (post.py). Machine-specific formatting through user-editable scripts. Supported EMC2/LinuxCNC and others. But this is through the HeeksCNC Python layer, not libarea.
- Gemini: Python-based G-code output.
- Codex: Post-processor scripts exist (emc2/mach3/centroid/etc.).

## Integration Potential (for Rust application)
**libarea is the key component here -- all models recognize this.**
- Claude: libarea is the most interesting component. Focused C++ library with BSD license for pocketing/profiling. ~23,000 LOC. Uses Clipper (also permissively licensed). Could be called from Rust via FFI. Unmaintained since 2020, would need modernization.
- Gemini: "High (for parts of it)" -- libarea is standalone, brilliant, lightweight. Can compile and generate C-bindings to call directly from Rust.
- Codex: Possible but legacy architecture and aging dependencies are substantial. Rates HeeksCNC 2/5 but notes the component extraction opportunity.

## Strengths
- libarea: BSD license -- permissive, integration-friendly (Claude, Gemini)
- libarea: Small, focused C++ library for exactly profile + pocket operations (Claude, Gemini)
- Proven algorithms -- used by FreeCAD's CAM workbench (Claude, Gemini)
- Could be wrapped via Rust FFI (Claude, Gemini)
- Reasonable code size (~23K LOC) (Claude)
- Uses Clipper library (Rust equivalents available) (Claude)
- Has all three operation classes at HeeksCNC level (Codex)
- Entry style options (helical/ramp/plunge) for pockets (Codex)

## Weaknesses
- Unmaintained since 2020 (libarea) / 2021 (HeeksCNC) (all models)
- No tab support in libarea itself (Claude)
- Limited documentation (Claude)
- Aging C++ code may need modernization (Claude, Codex)
- HeeksCNC (user-facing part) is defunct (all models)
- No active community for bug fixes (Claude)
- Legacy stack and maintenance risk (Codex)
- Must extract math libraries manually from the overall project (Gemini)

## Ratings

| Model | Rating | Key Reasoning |
|-------|--------|---------------|
| Claude Opus 4.6 | 3/5 | libarea is a hidden gem -- BSD C++ library for profile/pocket; no tabs; unmaintained |
| Gemini | 2/5 (HeeksCNC) / 4/5 (libarea extraction) | Overall project dead, but libarea is exactly what a Rust 2D CAM backend needs |
| GPT/Codex | 2/5 | Legacy stack; has needed operations but maintenance risk high |
| **Average** | **2.7/5** | |

## Sources
- [HeeksCNC GitHub](https://github.com/Heeks/heekscnc)
- [libarea GitHub](https://github.com/Heeks/libarea)
- [HeeksCAM GitHub](https://github.com/danheeks/HeeksCAM)
