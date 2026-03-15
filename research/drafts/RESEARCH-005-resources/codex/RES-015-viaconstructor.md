# viaconstructor — Python CAM tool with tab support
> **URL**: https://github.com/multigcs/viaconstructor
> **Type**: Code
> **Agent**: Codex
> **Relevance**: Medium
> **Credibility**: High

## Summary
viaconstructor is an open-source Python/C++ CAM tool that converts vector and related inputs to G-code, with explicit support for tabs, pockets, and islands. It is relevant as a modern lightweight codebase with practical CNC path features.

Its feature set includes automatic offset determination and nearest-neighbor ordering, indicating that some geometric optimization infrastructure already exists. This makes the project potentially useful as a reference for integrating tab placement with sequencing logic.

The documented limitation on circular-path tab generation is particularly informative. It points to a common difficulty in robust tab algorithms: handling non-linear geometry consistently across all contour types.

## Key Findings
- viaconstructor supports tabs in an end-to-end CAM pipeline with mixed Python/C++ implementation.
- A known issue reports tab generation failures on circular paths, showing a concrete geometric edge case.

## Technical Details
The stack is roughly split between Python and C++ and uses PyQt/OpenGL for interface and visualization. Path planning includes automatic ordering and offset logic. Tab insertion likely interacts with contour discretization and offset geometry, which may explain circular-shape instability.

## Strengths
- Open implementation with modern pipeline features and explicit known limitations.

## Weaknesses & Limitations
- Reported circular-path tab failures indicate reliability gaps for universal geometry handling.

## Connections
- [RES-011-freecad-dressuptag.md](/Users/finn.carlsvi/thetta/research/drafts/RESEARCH-005-resources/codex/RES-011-freecad-dressuptag.md) — both are open CAM systems with tag/tab capabilities, useful for comparative implementation strategies.
