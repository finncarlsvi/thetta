# DXF2GCODE - CAM Research Summary

## Overview
DXF2GCODE converts 2D drawings (DXF, PDF, PS) to CNC-compatible G-code. It is a focused tool for 2D contour cutting with support for milling, lathe, and drag knife operations. Purpose-built for the 2D-to-G-code conversion pipeline, it includes useful features like cutter compensation (G41/G42) and route optimization for minimizing rapid moves. A notable headless fork exists for CLI/batch processing.

- **Primary Language:** Python 3
- **License:** GPLv3
- **Source:** https://sourceforge.net/projects/dxf2gcode/ (primary), various GitHub forks
- **Dependencies:** Python 3, PyQt5, PyOpenGL, configobj

## Activity & Maturity
**Models disagree on activity level (likely evaluating different forks):**
- Claude: Development build DXF2GCODE-2024.11.2 available. SourceForge shows last update 2025-12-31. Latest stable: 2022.2.26.
- Gemini: Actively used in production environments via community forks (Python 3 ports on GitHub).
- Codex: Very low activity; last commit around 2015 (appears to have evaluated the `workflo/dxf2gcode` fork rather than the primary SourceForge project).

**Note:** The primary SourceForge project appears more active than what Codex found. Claude's data from SourceForge is likely most accurate.

- **Contributors:** ~7 (Christian Kohloffel, Jean-Paul Schouwstra, John Bradshaw, Vinzenz Schulz, Robert Lichtenberger, Damian Wrobel, Daniel Luginbuhl) (Claude)
- **Community Size:** Small but active. Available in Fedora/Debian repositories (Claude).

## Profile Cut Support
**All models agree: Good/Excellent -- this is the primary function.**
- Claude: Contour cutting with cutter compensation (G41 left, G42 right, G40 no compensation). Shape enabling/disabling/reversing. Start point shifting. Route optimization for minimizing G0 moves.
- Gemini: "Exceptional" -- flawless inside/outside cutter compensation, climb/conventional milling, multi-pass cutting.
- Codex: Yes, contour/offset-style cutting.

## Tab Support
**Models disagree:**
- Claude: Yes -- supported through a "breaking layer" mechanism. Draw rectangles on a special DXF layer; where they intersect a cutting contour exactly twice, a tab is created. Manual-but-CAD-integrated approach.
- Gemini: Yes, supports "bridges/tabs."
- Codex: "No meaningful holding-tab workflow found."

**Note:** Claude found specific documentation of the breaking-layer tab mechanism. Codex may have evaluated a different fork that lacks this feature. The primary project does support tabs, but through a non-standard manual approach.

## Pocket Cut Support
**All models agree: Basic/Weak.**
- Claude: "Pocket Mill" option available for inside compensation paths. Not sophisticated -- limited to simple offset-based pocketing.
- Gemini: "Weak" -- focused on following shapes; filling/clearing large pockets not its strong suit.
- Codex: Not a modern pocket-clearing CAM system.

## G-code Output
**Claude and Gemini agree: Good/Configurable. Codex less positive.**
- Claude: Fully adjustable post-processor. EMC2/LinuxCNC integration. Supports milling, lathe, and drag knife machine types. Route optimization for efficient G0 moves.
- Gemini: "Highly robust and configurable via standard text-based configuration files."
- Codex: Postprocessor config system exists; focused DXF conversion pipeline.

## Integration Potential (for Rust application)
- Claude: Moderate. A headless fork exists (holoujak/headless-dxf2gcode) with CLI operation, YAML configuration, automatic contour ordering (inner shapes first), and G-code optimization. Still Python-based and GPLv3.
- Gemini: "High (via CLI Subprocess)" -- designed to run headlessly from command line. Rust app can output DXF, shell out to dxf2gcode, and get production-ready G-code instantly.
- Codex: Legacy GUI-oriented code; extraction effort high vs payoff.

**Note:** The headless fork and CLI capability make this more practical than Codex suggests. Gemini and Claude identify a viable subprocess integration path.

## Strengths
- Purpose-built for 2D contour cutting -- the primary use case (Claude, Gemini)
- Cutter compensation (G41/G42) -- important for production quality (Claude, Gemini)
- Route optimization for efficient rapid moves (Claude)
- Tab support via breaking layers (Claude, Gemini)
- headless-dxf2gcode fork adds CLI/batch capability (Claude)
- DXF input aligns with common CAD output formats (Claude)
- Highly configurable post-processor (Claude, Gemini)
- Available in Linux distribution repositories (Claude)
- Tailor-made for 2D workflows, native CLI support (Gemini)

## Weaknesses
- Tab placement is manual (drawn in DXF), not automatic (Claude)
- Basic pocketing only (all models)
- GPLv3 license (all models)
- Python-based -- performance concerns, not easily embedded in Rust (Claude, Gemini)
- SourceForge-based development, less accessible than GitHub (Claude)
- Small community (Claude)
- Codex found the project stale (may have evaluated wrong fork)

## Ratings

| Model | Rating | Key Reasoning |
|-------|--------|---------------|
| Claude Opus 4.6 | 2/5 | Good for studying 2D contour algorithms; manual tabs and basic pocketing limit value |
| Gemini | 3.5/5 | Very practical low-effort CLI integration; tailor-made for 2D workflows |
| GPT/Codex | 2/5 | Stale, limited for modern nesting CAM needs |
| **Average** | **2.5/5** | |

## Sources
- [DXF2GCODE SourceForge](https://sourceforge.net/projects/dxf2gcode/)
- [DXF2GCODE User Guide](https://sourceforge.net/p/dxf2gcode/wiki/User%20Guide%20DXF2GCODE/)
- [headless-dxf2gcode GitHub](https://github.com/holoujak/headless-dxf2gcode)
- [DXF2GCODE GitHub fork](https://github.com/workflo/dxf2gcode)
