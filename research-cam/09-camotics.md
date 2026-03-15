# CAMotics - CAM Research Summary

## Overview
CAMotics is an open-source 3-axis CNC G-code simulator with TPL (Tool Path Language) scripting for programmatic toolpath generation. All three models agree on the fundamental point: CAMotics is primarily a simulation/verification tool, not a CAM toolpath generator. It reads G-code and simulates the cutting result, which makes it potentially useful as a verification step but not as a replacement for actual CAM generation.

- **Primary Language:** C++
- **License:** GPLv2+
- **Source:** https://github.com/CauldronDevelopmentLLC/CAMotics
- **Website:** https://camotics.org/

## Activity & Maturity
- **Last Commit:** Approximately early 2025; issues filed April 2025 (Claude)
- **GitHub Stars:** Not explicitly determined
- **Contributors:** Small team; maintained by CauldronDevelopmentLLC (Claude)
- **Release History:** Latest release v1.2.0. Estimated 24 person-years of effort by COCOMO (Claude).
- **Community Size:** Moderate. Used with Buildbotics CNC controller, V1E community, OpenBuilds (Claude). Standard in open-source community for G-code simulation (Gemini).

## Profile Cut Support
**All models agree: Not applicable for generation.**
- Claude: No native CAM toolpath generation. CAMotics reads G-code and simulates cutting. TPL provides JavaScript-based scripting for G-code generation, but it is a programming interface, not automatic CAM.
- Gemini: "N/A for generation" -- designed to read and simulate, not generate.
- Codex: Not intended as primary profile toolpath generator from 2D nesting geometry.

## Tab Support
**All models agree: Minimal/None for practical use.**
- Claude: TPL added tab support for 2D profile cuts in the DXF TPL library, but this is script-based, not automatic.
- Gemini: N/A -- not a fully-fledged CAM generator.
- Codex: No practical holding-tab CAM workflow.

## Pocket Cut Support
**All models agree: Not available.**
- Claude: CAMotics' roadmap lists "CAM routines (pocketing, offsetting, arrays, etc.)" as planned features, meaning they do not exist yet.
- Gemini: Not a production pocket CAM planner.
- Codex: Not a production pocket CAM planner for this use case.

## G-code Output
- Claude: TPL outputs G-code and implements LinuxCNC O-codes. The simulation engine parses and verifies G-code comprehensively. But output is only through TPL scripting, not automated CAM.
- Gemini: Excellent parsing/simulation of G-code, not post-focused CAM authoring.
- Codex: Excellent parsing/simulation; not post-focused CAM authoring.

## Integration Potential (for Rust application)
**All models agree: Useful as verifier, not as generator.**
- Claude: Moderate as simulation/verification tool. C++ codebase could verify G-code after generation by another tool. TPL is JavaScript-based for manual scripted generation.
- Gemini: If your Rust app needs to simulate the cut and export a 3D model of the finished sheet, CAMotics is perfect. For generating toolpaths, it is the wrong tool.
- Codex: Good as verifier/simulator stage, poor as CAM generation stage.

## Strengths
- Excellent G-code simulation and verification (all models)
- Could be used as a post-processing verification step (all models)
- C++ codebase (Claude)
- TPL scripting for programmatic G-code generation (Claude)
- Export cut workpiece to STL for visual verification (Claude)
- Unmatched open-source G-code simulation (Gemini)
- Validation and collision/risk reduction (Codex)

## Weaknesses
- Not a CAM tool -- no automatic toolpath generation from geometry (all models)
- No profile, pocket, or tab algorithms (all models)
- TPL is manual scripting, not automatic CAM (Claude)
- GPLv2+ license (Claude)
- Would only be useful for verification, not generation (all models)
- Does not solve core post-processing generation requirements (Codex)

## Ratings

| Model | Rating | Key Reasoning |
|-------|--------|---------------|
| Claude Opus 4.6 | 1/5 | Simulator, not CAM tool; misaligned with core requirements |
| Gemini | 1/5 (generation) / 5/5 (simulation) | Not a toolpath generator; unmatched for simulation |
| GPT/Codex | 1/5 | Useful adjunct for verification but doesn't solve CAM generation |
| **Average** | **1.0/5** (for CAM generation) | |

## Sources
- [CAMotics GitHub](https://github.com/CauldronDevelopmentLLC/CAMotics)
- [CAMotics Website](https://camotics.org/)
- [TPL Language](https://tplang.org/)
- [CAMotics Manual](https://camotics.org/manual.html)
