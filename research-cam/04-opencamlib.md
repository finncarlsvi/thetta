# OpenCAMLib - CAM Research Summary

## Overview
OpenCAMLib (OCL) is a C++ library for creating 3D toolpaths for CNC machines. It provides core algorithms for cutter-surface intersection calculations, focusing on drop-cutter and push-cutter (waterline) operations. Unlike most projects on this list, OCL is designed as a library rather than a GUI application, making it architecturally the most integration-friendly. However, its algorithms focus on 3D surface machining rather than the 2D profile/pocket operations needed for CNC nesting.

- **Primary Language:** C++
- **License:** LGPL v2.1 (since August 2018)
- **Source:** https://github.com/aewallin/opencamlib
- **Bindings:** Python (PyPI), Node.js, browser (WASM)

## Activity & Maturity
- **Last Commit:** Early-to-mid 2024 approximately (Claude)
- **GitHub Stars:** ~473 (Claude)
- **Forks:** ~155 (Claude)
- **Contributors:** ~31 (Codex); ~10 or fewer active (Claude)
- **Release History:** Latest release 2023.01.11 (Claude). Considered potentially discontinued by package health analyzers.
- **Community Size:** Small. LinuxCNC forum discussions, academic references (Claude). Serves as mathematical backbone for several other CAMs (Gemini).

## Profile Cut Support
**All models agree: Limited for 2D use case.**
- Claude: Primarily focused on 3D surface machining (drop-cutter, push-cutter/waterline). For 2D, offers OpenVoronoi for line segment offsetting, but this is experimental.
- Gemini: Provides low-level math (push-cutter, waterline) to calculate profile offsets, but only generates point-cloud/path data -- lead-in/lead-out logic left to user.
- Codex: Not a full 2D profile CAM app; provides geometric toolpath primitives, not a turnkey profiling workflow.

## Tab Support
**All models agree: None.** OCL is a pure geometric algorithm library with no concept of holding tabs. Tab placement would need to be implemented entirely externally.

## Pocket Cut Support
**All models agree: Experimental/None for production use.**
- Claude: Medial-axis pocketing described as "just a demonstration"; terrace roughing "not implemented yet but doable."
- Gemini: Provides offset toolpaths but no native high-speed adaptive clearing.
- Codex: No full 2D pocket workflow out of the box; building-block level only.

## G-code Output
**All models agree: None.** OCL produces toolpath point/coordinate data only. G-code generation must be handled entirely externally.

## Integration Potential (for Rust application)
**All models agree: Architecturally excellent, but wrong algorithms.**
- Claude: Best library architecture of all 12 projects. LGPL license most permissive. C++ with CMake and `find_package()` support. Pre-compiled libraries available. Python/Node.js/WASM bindings. Could be called via Rust FFI.
- Gemini: "Excellent" -- pure C++ library, can generate C bindings and wrap cleanly in Rust using `bindgen` or `cxx`. Lightning fast, no UI bloat.
- Codex: Excellent for Rust FFI to C++ core, but you must build all missing orchestration yourself.

**Consensus:** The architecture is ideal for integration, but you'd be building profile/tab/pocket/G-code logic from scratch on top of 3D surface algorithms -- a mismatch for 2D nesting needs.

## Strengths
- LGPL license -- most integration-friendly of the group (all models)
- Designed as a library, not a GUI application (all models)
- C++ with clean CMake integration, FFI-callable from Rust (all models)
- Lightning fast compiled algorithms (Gemini)
- Academically rigorous algorithms (Claude)
- Pre-compiled libraries and multiple language bindings available (Claude)
- Library-first design, good for algorithm reuse (Codex)

## Weaknesses
- Focused on 3D surface machining, not 2D profile/pocket (all models)
- No tab support whatsoever (all models)
- No G-code generation (all models)
- Experimental/incomplete pocketing (all models)
- Declining maintenance -- no releases since 2023 (Claude)
- Waterline algorithm has known bugs (Claude)
- Requires building entire orchestration layer on top (Codex)

## Ratings

| Model | Rating | Key Reasoning |
|-------|--------|---------------|
| Claude Opus 4.6 | 2/5 | Excellent architecture but wrong algorithms for 2D nesting use case |
| Gemini | 3/5 | Great foundation but requires heavy lifting to build missing features |
| GPT/Codex | 2/5 | Excellent FFI potential but must implement everything yourself |
| **Average** | **2.3/5** | |

## Sources
- [OpenCAMLib GitHub](https://github.com/aewallin/opencamlib)
- [OpenCAMLib Documentation](https://opencamlib.readthedocs.io/)
- [OpenCAMLib Releases](https://github.com/aewallin/opencamlib/releases)
- [Anders Wallin CAM Blog](https://www.anderswallin.net/cam/)
- [OpenCAMLib on Snyk](https://snyk.io/advisor/python/opencamlib)
