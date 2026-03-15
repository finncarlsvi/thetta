# Kiri:Moto - CAM Research Summary

## Overview
Kiri:Moto is a browser-based slicer and toolpath generator for 3D Printing, CNC milling, and laser cutting. It runs entirely in the browser using JavaScript, WebWorkers, and WASM. Developed by Stewart Allen (GridSpace founder) over 13+ years, it has grown into one of the most actively maintained and API-friendly open-source CAM solutions. Its MIT license and multiple programmatic interfaces make it one of the strongest candidates for integration, despite the JavaScript-to-Rust bridge complexity.

- **Primary Language:** JavaScript
- **License:** MIT
- **Source:** https://github.com/GridSpace/grid-apps
- **Website:** https://grid.space/kiri/

## Activity & Maturity
All three models agree this is very active:
- **Last Commit:** March 2026 (Claude, Codex)
- **GitHub Stars:** ~817 (Claude)
- **Forks:** ~183 (Claude)
- **Contributors:** ~38 (Codex); primarily Stewart Allen with community contributions (Claude)
- **Release History:** Over 13 years of development since 2013; regular releases on GitHub (Claude). Frequent releases (Codex).
- **Community Size:** Active forum (forum.grid.space), Discord, used in classrooms and makerspaces. Integrated with Onshape (Claude).

## Profile Cut Support
**All models agree: Good/Excellent.**
- Supports "Outline" operations for profile/contour cutting (Claude)
- Rough and contour operations available (Claude)
- Inside/outside contours, climb/conventional milling, multiple step-downs, finishing passes (Gemini)
- Contour/path-inside/path-outside operations (Codex)
- Multiple operation types can be combined per job (Claude)

## Tab Support
**Models agree: Present, with some limitations.**
- Claude: Tab feature exists; however, users report tabs only work on outside contours, not inside contours.
- Gemini: "Excellent" -- allows user to specify tab width, height, and spacing; engine automatically calculates placement along 2D contour.
- Codex: Yes, tab generation and tab-hopping behavior present.

**Note:** Gemini is the most positive; Claude flags a specific limitation (outside contours only) that others did not mention.

## Pocket Cut Support
**Models agree: Supported, with some concerns.**
- Claude: Pocket operations supported with contour option; forum reports indicate issues with toolpaths extending beyond pocket boundaries.
- Gemini: Strong standard pocketing (linear and offset strategies); doesn't have deep adaptive/trochoidal clearing but reliable for 3-axis flat pockets.
- Codex: Yes, pocket operations with ongoing improvements.

## G-code Output
**All models agree: Good/Configurable.**
- Customizable G-code post-processors (Claude)
- Supports GRBL and other controllers; G-code macros for start, tool change, end (Claude)
- Highly configurable JSON-based post-processor architecture handling GRBL, Marlin, LinuxCNC, and more (Gemini)
- Device/profile-driven generation; broad practical machine coverage via profiles (Codex)
- RML dialect support; file extension customizable (Claude)

## Integration Potential (for Rust application)
**All models agree: Good, especially via subprocess.**
- Claude: Multiple integration approaches -- JavaScript Engine API (async via web workers), IFrame Messaging API, CLI for headless operation. MIT license very favorable.
- Gemini: "Very Good (via Subprocess)" -- can run entirely headlessly via Node.js; serialize nested sheets, pass to headless Kiri CLI, receive G-code back. Requires Node.js runtime.
- Codex: Good if you accept JS runtime; exposes slicing/G-code APIs (engine.html, frame API).

**Consensus:** The most practical subprocess integration option. MIT license is a major advantage. Requires Node.js runtime alongside the Rust binary.

## Strengths
- MIT license -- most integration-friendly along with OCL's LGPL (all models)
- Multiple programmatic APIs: Engine, CLI, IFrame (Claude)
- Active development through March 2026 (all models)
- Written from scratch with no dependencies on other CAM codebases (Claude)
- Browser-based = potentially embeddable via Node.js/WASM (Claude)
- Modern architecture, API-oriented (Codex)
- Natively understands 2D CAM operations (Gemini)
- Robust tabs and pockets (Gemini)
- Good community and documentation (Claude)
- Can run entirely headlessly via Node.js (Gemini)

## Weaknesses
- JavaScript-based -- not ideal for Rust FFI; needs Node.js runtime or WASM bridge (all models)
- Pocket operations have reported bugs (Claude)
- Tab support limited to outside contours only (Claude)
- Primarily designed for STL/3D model input, not 2D contours (Claude)
- CNC mode is secondary to 3D printing mode (Claude)
- Browser-oriented architecture may not suit server-side/CLI batch processing well (Claude)

## Ratings

| Model | Rating | Key Reasoning |
|-------|--------|---------------|
| Claude Opus 4.6 | 3/5 | MIT license and APIs attractive; tab/pocket limitations; JS-to-Rust bridge complexity |
| Gemini | 4.5/5 | One of the most feature-complete, decoupled engines; headless Node.js integration |
| GPT/Codex | 4/5 | Active, capable, API-friendly; JS/web stack overhead in Rust backend |
| **Average** | **3.8/5** | |

## Sources
- [Kiri:Moto GitHub (grid-apps)](https://github.com/GridSpace/grid-apps)
- [Kiri:Moto Website](https://grid.space/kiri/)
- [Kiri:Moto API Documentation](https://docs.grid.space/kiri-moto/apis/)
- [Kiri:Moto CAM Overview](https://docs.grid.space/kiri-moto/CAM/)
- [V1E Kiri:Moto Basics](https://docs.v1e.com/software/kiri/)
