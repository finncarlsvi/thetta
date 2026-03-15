# PyCAM - CAM Research Summary

## Overview
PyCAM is a standalone toolpath generator for 3-axis CNC machining. It loads 3D models (STL) or 2D contour models (DXF, SVG) and generates G-code compatible with LinuxCNC and other controllers. It is a pure Python application that has been in development since the late 2000s, with intermittent periods of activity.

- **Primary Language:** Python
- **License:** GPLv3
- **Source:** https://github.com/SebKuzminsky/pycam
- **Website:** https://pycam.sourceforge.net/

## Activity & Maturity
- **Last Commit:** Sporadic; issues opened in Feb 2025 (Claude). Recently ported to Python 3/GTK3 (Gemini).
- **GitHub Stars:** ~289 (Claude)
- **Forks:** ~85 (Claude)
- **Contributors:** ~23 (Codex); primarily SebKuzminsky (Claude)
- **Release History:** v0.5.1 (2012), v0.6.0 (2017), v0.6.4 (latest on PyPI). 5-year pause between 2012-2017 (Claude). Last major stable release ~2019 (Gemini).
- **Community Size:** Small. SourceForge forums, limited GitHub activity (Claude). Lingering community (Gemini).

## Profile Cut Support
**All models agree: Supported.**
- Contour/follow strategies (marked experimental since v0.4) (Claude)
- Push Cutter mode removes material in layers; Drop Cutter for surface following (Claude)
- Conventional/climb milling options available (Claude)
- Engraving strategy for 2D contour following (Claude)
- Inside/along/around boundary modes (Codex)

## Tab Support
**Models agree: Present, but varying quality assessments.**
- Claude: Yes -- "Support bridges" with automatic distribution along outline (corners or edges); width, height, length configurable. Called this a "notable feature."
- Gemini: "Basic bridges (tabs)" -- somewhat primitive, lacking advanced automatic placement logic of newer software.
- Codex: Yes -- "support_bridges" with grid/distributed layouts.

## Pocket Cut Support
**All models agree: Basic/Limited.**
- Claude: "Very simple pocketing mode for 2D models" -- basic functionality only.
- Gemini: Basic linear/offset clearing (push-cutter style); no modern adaptive/trochoidal clearing.
- Codex: Grid/spiral style patterns; contour/slice/surface strategies.

## G-code Output
**All models agree: Limited.**
- Primarily targets LinuxCNC (all models)
- Limited post-processor customization compared to FreeCAD (Claude)
- Gemini: Basic text-based G-code emission; configurations exist but limited.
- Codex: Narrow dialect support in-tree.

## Integration Potential (for Rust application)
**Moderate across all models.**
- Claude: Master branch allows scripted (non-interactive) usage; pure Python makes algorithm study feasible; multi-processing support for parallel toolpath calculation; code aging and inconsistently maintained.
- Gemini: Could bundle and call as CLI subprocess via `std::process::Command`; processing algorithms notoriously slow for complex nested sheets due to lack of compiled C++ backend.
- Codex: Better than GUI-heavy tools (Python package/code), but still Python-first.

## Strengths
- Has tab/bridge support with automatic placement (Claude, Codex)
- Supports 2D contour models (DXF, SVG) directly (Claude)
- Scripted/headless usage possible (Claude, Gemini)
- Multi-core parallel processing (Claude)
- Pure Python makes algorithm study feasible for reimplementation (Claude, Gemini)
- 3-axis focus, scriptable flow model (Codex)
- Runs standalone, accepts 2D files, CLI accessible (Gemini)

## Weaknesses
- Inconsistent development; appears semi-abandoned (Claude, Gemini)
- Very basic pocketing support (all models)
- Aging codebase with GTK2/3 GUI dependencies (Claude)
- Limited post-processor options (all models)
- Performance likely poor for production use -- pure Python, notoriously slow on complex nests (Claude, Gemini)
- GPLv3 license (all models)
- Older architecture (Codex)
- Requires Python environment on host machine (Gemini)

## Ratings

| Model | Rating | Key Reasoning |
|-------|--------|---------------|
| Claude Opus 4.6 | 2/5 | Tab support relevant but inconsistent maintenance and basic pocketing |
| Gemini | 2/5 | Runs standalone and CLI accessible, but very slow and outdated |
| GPT/Codex | 3/5 | 3-axis focus and scriptable, but narrow post ecosystem |
| **Average** | **2.3/5** | |

## Sources
- [PyCAM GitHub](https://github.com/SebKuzminsky/pycam)
- [PyCAM Website](https://pycam.sourceforge.net/)
- [PyCAM Features](https://pycam.sourceforge.net/features/)
- [PyCAM PyPI](https://pypi.org/project/pycam/)
