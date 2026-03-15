# Kiri:Moto — Browser-based CAM
> **URL**: https://github.com/GridSpace/grid-apps
> **Type**: Code
> **Agent**: Claude
> **Relevance**: Medium
> **Credibility**: Medium

## Summary
Kiri:Moto is a browser-based, multi-modal slicer/CAM tool that supports 3D printing, CNC milling, and laser cutting. Developed by GridSpace over 13+ years, it runs entirely in the browser using HTML5 and WebGL, with the option to self-host via Docker or Node.js. The CNC mode includes stock fixing with tabs, making it relevant to this research.

The project represents a unique architectural approach — a full CAM tool running client-side in JavaScript. This is noteworthy for demonstrating that CNC toolpath generation (including tab placement) can be implemented efficiently in a higher-level language without requiring C/C++ for performance. The 13+ year development history suggests a mature codebase, though the browser-based architecture may limit computational complexity for large nesting layouts.

However, detailed documentation on the tab placement algorithm is not available in the README or readily accessible repository documentation. The tab feature is mentioned as part of the CNC mode's capabilities but without specifics on parameters, placement heuristics, or configuration options. This limits the resource's direct utility for understanding tab placement algorithms, though the source code (JavaScript) is available for deeper investigation.

## Key Findings
- Browser-based CAM with CNC tab support, running entirely client-side
- 13+ years of development — mature project
- Multi-modal: 3D printing, CNC milling, laser cutting in one tool
- Self-hostable via Docker or Node.js
- Tab implementation details not documented in README

## Technical Details
- **Language**: JavaScript (browser-based)
- **Requirements**: Modern HTML5 browser with WebGL
- **Modes**: 3D printing (FDM/SLA), CNC milling, Laser cutting
- **CNC features**: Includes tabs/stock fixing
- **Deployment**: Browser-based, self-hostable (Docker/Node.js)
- **License**: Open source (GitHub: GridSpace/grid-apps)
- **History**: 13+ years development

## Strengths
- Demonstrates tab placement implementation in JavaScript (accessible language)
- Mature, actively maintained project
- Open source — code available for algorithm study
- Multi-modal tool provides broad context for tab placement across cutting methods
- Browser-based architecture enables easy experimentation

## Weaknesses & Limitations
- Tab placement algorithm not documented externally
- Would require source code diving to extract algorithmic details
- Browser-based may limit computational complexity for large layouts
- Less focused than dedicated CNC CAM tools
- No information on shared-tab or nesting-aware tab placement

## Connections
- [RES-015 viaconstructor] — Python alternative with similar feature scope
- [RES-017 Deepnest] — browser-based nesting tool (could complement Kiri:Moto's CAM)
- [RES-013 bCNC Tabs] — simpler implementation with documented defaults
- [RES-011 FreeCAD DressupTag] — more documented open-source tab implementation
