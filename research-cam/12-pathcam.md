# PathCAM - CAM Research Summary

## Overview
PathCAM is a simple tool for generating 2.5D toolpaths to cut shapes from stock material using a CNC router. It supports direct robot connection via GRBL. Written in C# with an OpenGL-based GUI (OpenTK), it is a small hobby project by Benjamin R. Porter. It uses the Clipper library for polygon offsetting -- a useful data point since Clipper has Rust bindings available.

- **Primary Language:** C#
- **License:** GPLv3+ (Claude); GPL-3.0 (Codex); GPL/MIT per Gemini (unclear)
- **Source:** https://github.com/xenovacivus/PathCAM
- **GUI:** OpenTK (OpenGL-based)

## Activity & Maturity
- **Last Commit:** Unknown/sporadic (all models)
- **GitHub Stars:** ~237 (Claude)
- **Forks:** ~63 (Claude)
- **Contributors:** Primarily Benjamin R. Porter (single developer) (Claude)
- **Release History:** MSI installer available. No formal version numbering visible (Claude).
- **Community Size:** Very small. Occasional issues filed (Claude). Low-maintainer project (Codex).

## Profile Cut Support
**All models agree: Supported.**
- Claude: "Add Perimeter Paths" follows edges of objects. Paths divided into layers based on max cut depth. Two passes per edge: rough cut (bulk removal) and clean cut (exact dimension). Uses Clipper library for polygon offsetting.
- Gemini: Generates profiles mostly by "slicing" 3D STL files (waterline passes) rather than natively routing 2D vectors.
- Codex: Perimeter toolpaths with inside/outside handling in planner.

**Note:** Claude and Codex describe functional 2D profiling. Gemini views it as more 3D-mesh-oriented.

## Tab Support
**Models disagree:**
- Claude: Yes -- tabs appear as orange elements in GUI that can be moved around. Manual placement.
- Gemini: "Poor/Non-existent for 2D routing" -- geared toward 3D volume slicing.
- Codex: Yes -- Tab class with auto spacing/min tabs + avoidance in path generation.

**Note:** Claude and Codex found evidence of tab support in the codebase. Codex specifically identified a Tab class with auto-spacing logic.

## Pocket Cut Support
**All models agree: Limited to None.**
- Claude: No explicit pocket support documented. Focuses on perimeter/profile cutting with layered depth passes.
- Gemini: Very basic clearing.
- Codex: Limited; more perimeter/rout-area behavior than rich pocket strategy set.

## G-code Output
**All models agree: Basic.**
- Claude: Exports .gcode files; supports GRBL 8.0c+ for direct machine connection. Basic output without extensive post-processor customization.
- Gemini: Hardcoded and simple.
- Codex: Simple export with some direct machine connectivity.

## Integration Potential (for Rust application)
**All models agree: Poor.**
- Claude: C# on .NET/Mono. Building on Linux requires Mono. Uses OpenTK, Clipper, STLdotNET, Triangle.NET. GUI application, not a library. Extracting algorithms would require porting C# to Rust.
- Gemini: "Terrible" -- C#/.NET requires shipping .NET runtime or using heavy interop layers, highly inefficient for a tool with basic capabilities.
- Codex: Potentially extractable C# planning code, but not Rust-native and limited ecosystem.

**Note:** The Clipper usage is the most transferable insight -- Clipper is available as a Rust crate (`clipper2`).

## Strengths
- Simple 2.5D focus matches the use case (Claude)
- Tab support included with auto-spacing logic (Claude, Codex)
- Uses Clipper for polygon offsetting -- Clipper available in Rust (Claude)
- Clean, understandable codebase (Claude)
- Built-in tab-aware path planning (Codex)
- Fast if running on Windows (Gemini)

## Weaknesses
- C# / .NET dependency -- poor Rust interop (all models)
- No or very limited pocket support (all models)
- Single developer, uncertain maintenance status (all models)
- GPLv3+ license (all models)
- GUI application, not a library (Claude)
- Basic G-code output only (GRBL) (Claude)
- No headless/API mode (Claude)
- Relies on 3D meshes rather than 2D geometry in some workflows (Gemini)

## Ratings

| Model | Rating | Key Reasoning |
|-------|--------|---------------|
| Claude Opus 4.6 | 1/5 | Too limited (no pocketing) and wrong tech stack (C#) |
| Gemini | 1/5 | C# dependency, abandoned, relies on 3D meshes |
| GPT/Codex | 2/5 | Built-in tab-aware planning, but narrow scope and GPL |
| **Average** | **1.3/5** | |

## Sources
- [PathCAM GitHub](https://github.com/xenovacivus/PathCAM)
