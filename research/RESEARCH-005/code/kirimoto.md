# Kiri:Moto -- Browser-based CAM

> **URL**: https://github.com/GridSpace/grid-apps
> **Type**: Code
> **Agents**: Claude, Codex, Gemini
> **Relevance**: Medium
> **Confidence**: Medium -- tab algorithm details undocumented

## Summary
Kiri:Moto is a browser-based, multi-modal slicer/CAM tool that supports 3D printing, CNC milling, and laser cutting. Developed by GridSpace over 13+ years, it runs entirely in the browser using HTML5 and WebGL, with the option to self-host via Docker or Node.js. The CNC mode includes stock fixing with tabs, making it relevant to this research.

The project demonstrates that CNC toolpath generation including tab placement can be implemented efficiently in JavaScript without requiring C/C++ for performance. However, detailed documentation on the tab placement algorithm is not available in the README or readily accessible repository documentation. The tab feature is mentioned as part of CNC mode capabilities but without specifics on parameters, placement heuristics, or configuration options. The source code (JavaScript) is available for deeper investigation.

## Key Findings
- Browser-based CAM with CNC tab support, running entirely client-side (all agents)
- 13+ years of development -- mature project (all agents)
- Multi-modal tools often share geometric processing libraries across manufacturing methods (Gemini)
- Tab implementation details not documented in README (all agents)
- May struggle with extremely complex, high-vertex-count nests compared to compiled engines (Gemini)

## Technical Details
- **Language**: JavaScript (browser-based)
- **Requirements**: Modern HTML5 browser with WebGL
- **Modes**: 3D printing (FDM/SLA), CNC milling, Laser cutting
- **CNC features**: Includes tabs/stock fixing
- **Deployment**: Browser-based, self-hostable (Docker/Node.js)
- **License**: Open source (GitHub: GridSpace/grid-apps)

## Strengths
- Demonstrates tab placement in JavaScript (accessible language)
- Mature, actively maintained project
- Open source -- code available for algorithm study
- Multi-modal tool provides broad context for tab placement across cutting methods

## Weaknesses & Limitations
- Tab placement algorithm not documented externally
- Would require source code diving to extract algorithmic details
- Browser-based may limit computational complexity for large layouts
- No information on shared-tab or nesting-aware tab placement

## Consensus & Disagreements
All agents agree on maturity, browser-based architecture, and undocumented tab algorithm. Gemini uniquely notes that multi-modal tools reuse offset logic across manufacturing methods. No factual disagreements. Credibility ratings vary (Medium from Claude, High from Codex/Gemini).

## Connections
- [RES-015] viaconstructor -- Python alternative with similar feature scope
- [RES-017] Deepnest -- browser-based nesting tool (could complement Kiri:Moto's CAM)
- [RES-013] bCNC Tabs -- simpler implementation with documented defaults
- [RES-011] FreeCAD DressupTag -- more documented open-source tab implementation
