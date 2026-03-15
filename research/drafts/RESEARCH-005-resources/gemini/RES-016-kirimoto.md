# Kiri:Moto Browser-based CAM
> **URL**: https://github.com/GridSpace/grid-apps
> **Type**: Code
> **Agent**: Gemini
> **Relevance**: Medium
> **Credibility**: High

## Summary
Kiri:Moto is a unique, browser-based CAM and slicing application built entirely in JavaScript. It is a multi-modal tool, meaning it supports workflows for 3D printing (FDM/SLA), laser cutting, and CNC milling within a single unified interface.

With over 13 years of continuous development, it represents a highly mature open-source codebase. In its CNC milling mode, Kiri:Moto explicitly supports "stock fixing with tabs." Because it operates in the browser, all geometric processing, toolpath generation, and tab calculations are executed client-side via JavaScript and WebGL.

While the specific algorithmic details of its tab placement are not heavily documented in the README, the existence of robust CNC tab support in a JavaScript environment proves that heavy C++ or Python backends are not strictly necessary for computational geometry in CAM, provided modern browser APIs are utilized.

## Key Findings
- Complex CAM operations, including toolpath generation and automatic tab placement, can be efficiently executed in JavaScript running in a web browser.
- Multi-modal tools often share geometric processing libraries across different manufacturing methods (e.g., sharing offset logic between 3D slicing perimeters and CNC contours).
- Tabs are a necessary feature even for lightweight, browser-based CNC tools.

## Technical Details
The application relies heavily on HTML5 canvas and WebGL for rendering and geometric operations. It can be self-hosted via Docker or Node.js. The JavaScript engine must handle vector math, path offsetting, and Z-height manipulations to generate the tabbed G-code.

## Strengths
- Proves the viability of JavaScript/TypeScript for complex CAM geometry.
- Highly accessible and easily testable without local installation.

## Weaknesses & Limitations
- Algorithm details for tab placement are opaque without deep-diving into the undocumented JavaScript source code.
- May struggle with extremely complex, high-vertex-count nests compared to compiled C++/Rust engines.

## Connections
- Contrasts with the Python/C++ approaches of viaconstructor (RES-015) and FreeCAD (RES-012), showing a web-first architecture.
- Shares the web-tech foundation with Deepnest (RES-017).
