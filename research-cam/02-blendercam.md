# BlenderCAM / FabexCNC - CAM Research Summary

## Overview
FabexCNC (formerly BlenderCAM, renamed due to Blender trademark requirements) is an open-source CAM extension for Blender that generates toolpaths and G-code from 3D models or 2D curves. Originally created by Vilem Novak in 2012, it has been maintained by Alain Pelletier and a team of 15+ credited contributors. The project is primarily oriented toward artistic relief carving and 3D machining rather than 2D CNC routing.

- **Primary Language:** Python (Blender addon)
- **License:** GPLv3
- **Source:** https://github.com/vilemduha/blendercam
- **Website:** https://blendercam.com/

## Activity & Maturity
- **Last Commit:** January 2026 (Codex); November 2025 (Claude)
- **GitHub Stars:** ~539 (Claude)
- **Forks:** ~129 (Claude)
- **Contributors:** ~51 (Codex); 15+ credited (Claude)
- **Release History:** Latest release v1.0.68 (Claude). Active development since 2012.
- **Community Size:** Matrix chat room, active GitHub issues, Carbide 3D community discussions (Claude). Gemini notes it was "mostly active ~2016-2020" with sporadic maintenance.

## Profile Cut Support
**Models mostly agree: Basic to Good.**
- Claude: Profile is a core milling strategy; supports 2D, 3D, and multi-axis strategies; contour following of curves and edges.
- Gemini: "Basic" -- generates 2D profiles using offset curves but heavily oriented toward 3D mesh slicing rather than pure 2D CAD paths.
- Codex: Yes, inside/outside/on-line profile strategies with climb/conventional options.

**Note:** Gemini is more pessimistic, viewing this as primarily a 3D tool doing 2D work as a secondary concern.

## Tab Support
**Models disagree significantly:**
- Claude: "Not clearly documented" -- explicit holding tab support not prominently featured; may need manual curve modifications. Notable gap.
- Gemini: "Very weak" -- mostly requires physically modeling bridges in 3D mesh. No sophisticated automatic 2D tab algorithm.
- Codex: Yes, "Bridges/Tabs" supported with automated placement, configurable width/height/min count/distance.

**Note:** Codex found more positive evidence of tab support than Claude or Gemini. This discrepancy may reflect recent improvements or different depth of investigation.

## Pocket Cut Support
**Models agree: Good for 3D, basic for 2D.**
- Claude: Pocket is a core strategy; supports various clearing patterns; includes collision detection and simulation.
- Gemini: "Basic" -- mostly relies on image depth-mapping or simple waterline offsets; lacks high-speed adaptive clearing.
- Codex: Yes, pocket ops with multiple 2D/3D strategies.

## G-code Output
- Claude: Supports G-code export; post-processor customization available but less documented than FreeCAD.
- Gemini: Standard Python exporters for common controllers, somewhat rigid.
- Codex: Multiple post processors (GRBL, Mach3, EMC/LinuxCNC variants).

## Integration Potential (for Rust application)
**All models agree: Poor.** Requires Blender as the host application. The addon is deeply tied to Blender's render engine framework (FABEX_ENGINE) and scene graph (Claude). Blender's `bpy` Python API is the exclusive interface (Gemini). Cannot be used headlessly without Blender's full environment. Even larger dependency than FreeCAD for this use case.

## Strengths
- Mature project with 14+ years of development (Claude)
- Full 3D machining capability -- robust for artistic/relief work (Claude, Gemini)
- Active development and community (Claude, Codex)
- Rich CAM for routing workflows (Codex)
- Good simulation and preview capabilities (Claude)
- Multiple post-processor support (Codex)

## Weaknesses
- Requires Blender as a dependency -- even larger than FreeCAD for 2D needs (all models)
- Deeply coupled to Blender's 3D engine and scene model (all models)
- Tab support unclear/weak for 2D work (Claude, Gemini -- Codex disagrees)
- Not designed as a library; extraction of algorithms very difficult (all models)
- GPLv3 license (all models)
- Overkill for 2D/2.5D operations (Claude, Gemini)
- Primarily oriented toward artistic 3D relief carving (Gemini)

## Ratings

| Model | Rating | Key Reasoning |
|-------|--------|---------------|
| Claude Opus 4.6 | 1/5 | Too tightly coupled to Blender; tab support gap; overkill for 2D |
| Gemini | 1/5 | Requires Blender, poor 2D tab support, overly complex for 2D nesting |
| GPT/Codex | 3/5 | Rich CAM for routing workflows, but Blender dependency is heavy |
| **Average** | **1.7/5** | |

## Sources
- [BlenderCAM GitHub](https://github.com/vilemduha/blendercam)
- [FabexCNC Documentation](https://spectralvectors.github.io/blendercam/index.html)
- [BlenderCAM Website](https://blendercam.com/)
- [BlenderCAM DeepWiki](https://deepwiki.com/vilemduha/blendercam/1-overview)
