# Kiri:Moto — Browser-based CAM
> **URL**: https://github.com/GridSpace/grid-apps
> **Type**: Code
> **Agent**: Codex
> **Relevance**: Medium
> **Credibility**: High

## Summary
Kiri:Moto is a long-running browser-based CAM platform that supports CNC, laser, and 3D printing workflows. It is relevant because it demonstrates tab-capable CAM in a web-native architecture rather than desktop-only software.

The cache indicates CNC tab support exists, but algorithm specifics are not well documented in accessible summaries. Even so, the project’s maturity and multi-modal scope suggest practical heuristics and production-tested path transformations are present in code.

For the research brief, Kiri:Moto is useful as a potential source of implementation patterns for interactive tab editing and cross-platform deployment. It is less directly useful for formal tab optimization methods without deeper source inspection.

## Key Findings
- Kiri:Moto provides CNC operations in-browser and includes tab-based stock-fixing functionality.
- The project is open-source, self-hostable, and mature, implying extensive field usage.

## Technical Details
The application runs in modern HTML5/WebGL environments and can be deployed via Docker/Node. Tab behavior likely sits in toolpath generation modules with UI-driven parameter inputs; however, the cache does not expose detailed placement formulas or shared-tab logic.

## Strengths
- Mature open-source CAM platform with accessible deployment model and broad machine-mode support.

## Weaknesses & Limitations
- Insufficient documented detail in cache on automatic tab placement algorithms.

## Connections
- [RES-013-bcnc-repo.md](/Users/finn.carlsvi/thetta/research/drafts/RESEARCH-005-resources/codex/RES-013-bcnc-repo.md) — both are open CAM tools with tab support but different architectural choices (browser vs desktop Python).
