# jscut - CAM Research Summary

## Overview
jscut is a browser-based CAM tool that converts SVG files to CNC cutting paths and G-code, focusing on 2.5D operations. It was a popular tool in the maker community around 2014-2018 but has been explicitly abandoned by its author (tbfleming), who stated "I no longer add new features to jscut. LaserWeb4 is the successor." The C++ core is compiled to WASM via Emscripten and uses the Clipper library for geometry operations.

- **Primary Language:** JavaScript + C++ (compiled to WASM via Emscripten)
- **License:** GPLv3
- **Source:** https://github.com/tbfleming/jscut
- **Website:** https://jscut.org/

## Activity & Maturity
**All models agree: Abandoned/Inactive.**
- **Last Commit:** Inactive for several years (Claude). Low recent velocity (Codex).
- **GitHub Stars:** ~324 (Claude)
- **Forks:** ~143 (Claude)
- **Contributors:** Primarily tbfleming (single developer) (Claude, Codex)
- **Release History:** No recent releases. Author explicitly moved on to LaserWeb4 (Claude).
- **Community Size:** Defunct. Open issues remain unresolved (Claude).

## Profile Cut Support
**All models agree: Supported.**
- Supports "Inside" and "Outside" cut operations for profile/contour cutting (Claude)
- SVG-based input (Claude)
- Climb/conventional control (Codex)
- Specifically designed for 2D profile offsets (Gemini)

## Tab Support
**Models partially agree: Present but problematic.**
- Claude: Partially implemented. Tab support was requested (issue #17) and partially added, but with issues: "the tool path unpredictably changes direction on cut-up path segments, sending the cutter through areas that shouldn't be cut."
- Gemini: "Yes, supports basic holding tabs manually placed along SVG paths."
- Codex: Yes, tab UI + tab geometry processing; mostly user/manual driven.

**Note:** Claude found specific evidence of reliability issues in the tab implementation that others did not flag.

## Pocket Cut Support
**All models agree: Basic but present.**
- Claude: "Pocket" is a core operation; C++ implementation includes `hspocket.cpp`; V Pocket operation also supported.
- Gemini: Offset pocketing utilizing the robust Clipper geometry library.
- Codex: Yes, Pocket op.

## G-code Output
**All models agree: Basic/Limited.**
- Basic G-code output with limited post-processor customization (Claude)
- String-replacement post-processor logic (Gemini)
- Simple generic G-code generator, limited post sophistication (Codex)

## Integration Potential (for Rust application)
**Models range from Low to Moderate.**
- Claude: Low. Abandoned project. C++ core could theoretically be studied but codebase is small and dated. SVG-only input limiting. GPLv3.
- Gemini: Moderate. Could be stripped down for headless Node context, but inheriting technical debt from unmaintained project.
- Codex: Good as lightweight JS CAM engine; easier extraction than large GUI systems.

**Note:** Codex is notably more positive, viewing jscut's simplicity as an advantage for extraction. Claude and Gemini emphasize the abandonment risk.

## Strengths
- SVG-based 2D input aligns with 2D nesting output (Claude)
- Simple, focused feature set (Claude, Codex)
- C++ core algorithms (pocket/profile) could be studied (Claude)
- Lightweight -- specifically tailored only to 2D CAM tasks (Gemini)
- Uses Clipper library for geometry (Gemini)
- Simple 2D CAM core matching nesting output geometry (Codex)

## Weaknesses
- Abandoned/unmaintained -- author explicitly moved to LaserWeb4 (Claude, Gemini)
- Tab implementation is buggy with known path direction issues (Claude)
- GPLv3 license (all models)
- Limited post-processor support (all models)
- No API for programmatic use (Claude)
- Small, dated codebase with known issues (Claude)
- Technical debt from years of no maintenance (Gemini)

## Ratings

| Model | Rating | Key Reasoning |
|-------|--------|---------------|
| Claude Opus 4.6 | 1/5 | Abandoned with buggy tabs; not suitable for integration |
| Gemini | 2/5 | Lightweight 2D focus but dead project with JS dependency |
| GPT/Codex | 3/5 | Simple 2D CAM core matching nesting geometry; aging but extractable |
| **Average** | **2.0/5** | |

## Sources
- [jscut GitHub](https://github.com/tbfleming/jscut)
- [jscut Website](https://jscut.org/)
- [jscut Tab Support Issue #17](https://github.com/tbfleming/jscut/issues/17)
