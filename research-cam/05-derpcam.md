# DerpCAM - CAM Research Summary

## Overview
DerpCAM is a 2.5D CAM package and toolpath generator specifically designed for hobby 3-axis CNC routers and mills. It targets LinuxCNC and GRBL controllers. Created and primarily maintained by Krzysztof Foltman, it is purpose-built for exactly the kind of operations needed in a CNC nesting workflow: profile cuts with tabs, pocket clearing, and basic G-code generation. Despite its small community, it has the best feature-to-complexity ratio for 2D/2.5D CNC routing among the projects surveyed.

- **Primary Language:** Python 3
- **License:** GPLv3
- **Source:** https://github.com/kfoltman/DerpCAM
- **GUI:** Qt5

## Activity & Maturity
**Models disagree on activity level:**
- Claude: Last commit April 28, 2025; ~26 stars; active development.
- Gemini: "Essentially dead" -- a personal hobbyist project with very low activity.
- Codex: Smaller project, low contributor count, primarily single maintainer.

**Note:** Claude's data is more specific (dated commit), while Gemini appears to have underestimated activity. The project is small but not dead.

- **Contributors:** Primarily Krzysztof Foltman (single developer) (all models)
- **Release History:** No formal releases; run from source directory (Claude)
- **Community Size:** Very small. V1E forum mentions (Claude).

## Profile Cut Support
**All models agree: Good/Strong.**
- Outside/inside slot milling is a core feature (Claude, Codex)
- Supports wide slots (Claude)
- DXF import from LibreCAD (Claude)
- Side milling, climb/conventional control (Codex)
- Gemini: "Very basic 2D profiling" (more pessimistic than others)

## Tab Support
**Claude and Codex agree: Yes, strong. Gemini disagrees.**
- Claude: "Outside/inside slot milling with optional tabs" is a key feature; tabs are part of core slot milling operation.
- Codex: Strong -- automatic + manual placement.
- Gemini: "Unlikely to have robust, production-ready tab generation."

**Note:** Claude and Codex both found specific evidence of tab support in the README/codebase. Gemini appears to have missed this.

## Pocket Cut Support
**Claude and Codex agree: Good with advanced features. Gemini disagrees.**
- Claude: Supports trochoidal paths for efficient clearing; rest machining (refining coarsely-milled pocket with finer tool) -- a sophisticated feature typically found in commercial CAM.
- Codex: Strong for 2D/2.5D with islands, multiple strategies including HSM/trochoidal notes.
- Gemini: "Simple linear offsets."

## G-code Output
**All models agree: Limited dialect support.**
- Generates G-code for LinuxCNC and GRBL (all models)
- Does NOT support commercial machine vendor dialects (Claude)
- Limited post-processor customization (Claude)
- Hardcoded and rigid (Gemini -- though this may be overstated)

## Integration Potential (for Rust application)
- Claude: Moderate-to-good. Has a Python API for generating G-code programmatically (not just GUI). Wall profile support via Python API only.
- Gemini: "Poor" -- collection of hobby scripts lacking robustness for production nesting app.
- Codex: Good practical fit via Python API; Rust integration via embedded Python/subprocess likely straightforward.

**Note:** Claude and Codex found the Python API and view it as a practical integration path. Gemini's assessment appears to undervalue this.

## Strengths
- Purpose-built for exactly the 2.5D use case: profile cuts, tabs, and pockets (Claude, Codex)
- Trochoidal milling and rest machining -- advanced features rare in open-source (Claude)
- Python API for programmatic use, not just GUI (Claude, Codex)
- Simple, focused codebase -- excellent study reference (Claude)
- Directly aligned with 2D nesting post-process needs (Codex)
- Active development as of April 2025 (Claude)

## Weaknesses
- Very small community -- essentially single developer (all models)
- Not meant for professional use (author's own disclaimer) (Claude)
- Limited G-code dialect support (LinuxCNC/GRBL only) (all models)
- GPLv3 license (all models)
- No formal releases or versioning (Claude)
- DXF import limited to LibreCAD output (Claude)
- Small bus factor -- limited real-world validation (Claude, Codex)
- Small star count (~26) (Claude)

## Ratings

| Model | Rating | Key Reasoning |
|-------|--------|---------------|
| Claude Opus 4.6 | 3/5 | Best feature-to-complexity ratio; excellent study reference; single-dev risk |
| Gemini | 1/5 | Considered essentially dead hobby scripts |
| GPT/Codex | 4/5 | Best direct feature fit + Python API for nesting pipeline |
| **Average** | **2.7/5** | |

## Sources
- [DerpCAM GitHub](https://github.com/kfoltman/DerpCAM)
- [DerpCAM V1E Forum Discussion](https://forum.v1e.com/t/derpcam-open-source-2-5d-cam/47525)
