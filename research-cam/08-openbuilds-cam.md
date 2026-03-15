# OpenBuilds CAM - CAM Research Summary

## Overview
OpenBuilds CAM is a web-based CAM application for converting SVG, DXF, and Bitmap files to G-code for CNC, Laser, Plasma, and Drag Knife machines. It is part of the OpenBuilds ecosystem, described as "the world's largest online community" for CNC. The tool supports the three core operations (profile, tabs, pocket) but is tightly coupled to its web GUI and the GRBL ecosystem, with an AGPL-3.0 license that poses significant integration constraints.

- **Primary Language:** JavaScript
- **License:** AGPL-3.0 (Claude) / MIT per Gemini (disagreement -- see note)
- **Source:** https://github.com/OpenBuilds/OpenBuilds-CAM
- **Website:** https://cam.openbuilds.com/

**License note:** Claude and Codex identify AGPL-3.0; Gemini identifies MIT. The GitHub repository itself declares AGPL-3.0, so Claude and Codex appear correct.

## Activity & Maturity
- **Last Commit:** March 4, 2025 (Claude)
- **GitHub Stars:** ~775 (Claude)
- **Forks:** ~337 (Claude)
- **Contributors:** OpenBuilds team (Claude); small maintainer set (Codex)
- **Release History:** Regular updates; latest changelog entry December 2024 (Claude).
- **Community Size:** Large -- part of the OpenBuilds ecosystem (Claude). Moderate activity (Codex).

## Profile Cut Support
**All models agree: Good.**
- Supports vector outside/inside paths for profile cutting (Claude)
- CNC: Vector with no offset / inside / outside (Codex)
- Good 2D profile generation from DXF/SVG (Gemini)

## Tab Support
**All models agree: Supported.**
- Tab support documented with community tutorials (Claude)
- Tab depth/width/spacing and automatic placement logic in worker (Codex)
- Basic tab support for cutouts (Gemini)

## Pocket Cut Support
**All models agree: Basic.**
- Pocketing supported with bug fixes noted for Z-depth issues on unmerged geometry (Claude)
- CNC: Pocket op (Codex)
- Basic 2D offset pocketing (Gemini)

## G-code Output
**All models agree: Limited to GRBL ecosystem.**
- Primarily for GRBL controllers (all models)
- Pairs with OpenBuilds CONTROL desktop app for machine communication (Claude)
- Highly tailored for GRBL (Gemini)
- Practical CNC router output (Codex)

## Integration Potential (for Rust application)
**All models agree: Poor, primarily due to licensing and architecture.**
- Claude: AGPL-3.0 is the most restrictive license -- requires source disclosure for network-interacting software. No documented API for headless use. JavaScript GUI application, not a library.
- Gemini: "Poor" -- logic heavily intertwined with UI (jQuery/frontend frameworks); stripping CAM math from web UI would be a massive refactor.
- Codex: Possible by lifting worker logic, but app-centric codebase and AGPL constraints matter. Rates 4/5 technically but 3/5 if AGPL is a blocker.

## Strengths
- Large community and well-tested for hobby CNC (Claude)
- Tab support included with automatic placement (Claude, Codex)
- Active development (Claude)
- Good user experience for end users (Claude)
- Directly supports all three needed operations (Codex)
- Nice user interface for simple 2D work (Gemini)

## Weaknesses
- AGPL-3.0 license is very restrictive for integration into commercial/proprietary software (Claude, Codex)
- No API or headless mode (Claude)
- JavaScript GUI application, not a library (Claude, Gemini)
- Limited to GRBL ecosystem (all models)
- Web-based architecture not suited for embedding (Claude)
- Logic heavily intertwined with UI/frontend frameworks (Gemini)
- Not designed as a clean embeddable library (Codex)

## Ratings

| Model | Rating | Key Reasoning |
|-------|--------|---------------|
| Claude Opus 4.6 | 1/5 | AGPL license unsuitable; no API; purely a web GUI application |
| Gemini | 1.5/5 | UI-coupled, not designed as API or standalone engine |
| GPT/Codex | 3-4/5 | Feature fit strong technically; 3/5 if AGPL is a blocker |
| **Average** | **2.0/5** | |

## Sources
- [OpenBuilds CAM GitHub](https://github.com/OpenBuilds/OpenBuilds-CAM)
- [OpenBuilds CAM Website](https://cam.openbuilds.com/)
- [OpenBuilds Software](https://software.openbuilds.com/)
- [OpenBuilds Documentation](https://docs.openbuilds.com/doku.php?id=docs:software:cam)
