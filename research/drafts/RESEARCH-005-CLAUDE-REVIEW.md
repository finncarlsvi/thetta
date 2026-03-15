# RESEARCH-005: Cross-Reference Review

**Agent:** Claude
**Date:** 2026-03-15
**Scope:** 28 resources reviewed by three agents (Claude, Codex, Gemini)

---

## Per-Resource Cross-Reference

### RES-001: Micro-Joints Genetic Algorithm (Kandasamy & Udhayakumar)

**Agreement (all 3):** Centroid-based heuristic places exactly 3 micro-joints per part at 120° intervals; GA optimizes projection angle (0-120°) and cutting sequence to minimize non-productive air travel. Chromosome has two components: angular variables and sequence variables. Paper is paywalled.

**Disagreements:** None of substance. All three agents are factually aligned.

**Codex found, Claude missed:** Nothing substantial.

**Gemini found, Claude missed:** Gemini notes the centroid projection may place tabs in non-ideal locations on "highly concave parts (e.g., inside deep notches)" — Claude makes a similar but more technical point about projection lines intersecting contour multiple times on non-convex parts. Same insight, different framing.

**Claude found, others missed:** Publication details (Vol. 13, pp. 317-329, 2020), author names, applicable processes (laser, plasma, AWJ), gap analysis (no material properties/part weight/thickness in tab sizing, no shared tabs between adjacent parts), most extensive connections (RES-002, RES-003, RES-010, RES-021).

| Agent | Thoroughness (1-5) | Accuracy (1-5) |
|-------|-------------------|-----------------|
| Codex | 3 | 5 |
| Gemini | 4 | 5 |
| Claude | 5 | 5 |

---

### RES-002: Common-Edge Nested Cutting

**Agreement (all 3):** Paper addresses common-edge nesting where adjacent parts share cut boundaries, published in Computer-Aided Design. Directly relevant to the shared-tab problem. Full text paywalled (403). Common-edge nesting changes optimization structure vs. standard nesting with spacing.

**Disagreements:** All agents are speculating from abstracts/metadata. Codex mentions "edge ownership, timing, and holding-force continuity" constraints. Gemini speculates focus may be thermal cutting. Claude infers constrained optimization formulation.

**Codex found, Claude missed:** "Edge ownership, timing, and holding-force continuity" as constraint framing — a useful conceptual model.

**Gemini found, Claude missed:** "Shared segments cannot be cut twice" as a fundamental constraint; premature detachment risk from wrong cut sequence — more concrete operational insights.

**Claude found, others missed:** Journal name and year (Computer-Aided Design, 2023), "current state of the art" framing, most connections (RES-001, RES-017, RES-024, RES-018), clearest shared-tab dependency statement ("a tab for one part must also serve as a tab for the adjacent part").

| Agent | Thoroughness (1-5) | Accuracy (1-5) |
|-------|-------------------|-----------------|
| Codex | 3 | 4 |
| Gemini | 4 | 4 |
| Claude | 4 | 4 |

---

### RES-003: Cutting Path Two-Step GA

**Agreement (all 3):** Paper formulates CNC cutting path optimization as GTSP using a two-step GA separating piercing point optimization (global) from part sequencing (local). Objective: minimize non-productive travel. All rate Medium relevance, High credibility. Paywalled.

**Disagreements:** None of substance.

**Codex found, Claude missed:** Concrete extension proposal: "encode tab-prohibited entry arcs, protected hold regions, and late-release ordering constraints as additional chromosome genes or penalty terms" — more actionable than Claude's suggestions.

**Gemini found, Claude missed:** Computational cost concern: approach "may be computationally intensive compared to simpler heuristics for real-time CAM software."

**Claude found, others missed:** Journal identification (International Journal of Production Research, Vol. 44, No. 24), most explicit piercing-point/tab connection, gap analysis (no workholding stability constraints).

| Agent | Thoroughness (1-5) | Accuracy (1-5) |
|-------|-------------------|-----------------|
| Codex | 3 | 5 |
| Gemini | 4 | 5 |
| Claude | 5 | 5 |

---

### RES-004: Tool Path Optimization Models

**Agreement (all 3):** IFAC-PapersOnLine paper classifies tool path optimization problems as GTSP variants with additional constraints including thermal effects. Medium relevance, High credibility. Taxonomic/theoretical rather than algorithmic. Paywalled.

**Disagreements:** Gemini and Claude both note thermal cutting focus but draw opposite conclusions — Gemini says it "behaves differently than physical routing where tabs are most critical"; Claude says thermal constraints "may not apply to Thetta's primary use case (CNC routing of wood/plastics)." Same point, different framing.

**Codex found, Claude missed:** Nothing.

**Gemini found, Claude missed:** "Maps physical constraints (like heat distribution or part stability) to mathematical boundaries" — clearer framing of constraint-mapping approach.

**Claude found, others missed:** Specific cutting methods covered (thermal: laser/plasma; mechanical: waterjet/milling), classification as "survey/classification paper," more connections (RES-003, RES-001, RES-024).

| Agent | Thoroughness (1-5) | Accuracy (1-5) |
|-------|-------------------|-----------------|
| Codex | 2 | 4 |
| Gemini | 4 | 5 |
| Claude | 4 | 5 |

---

### RES-005: MecSoft Bridges & Tabs

**Agreement (all 3):** MecSoft offers Triangular and Rectangular shapes; configurable Bridge Height and Bridge Length. Supports automatic (by count or distance) and manual placement. Located in Advanced Cut Parameters of 2-1/2 Axis Profiling. Limited algorithmic depth.

**Disagreements:** Credibility: Gemini and Claude rate High, Codex rates Medium.

**Codex found, Claude missed:** Nothing.

**Gemini found, Claude missed:** Nothing — all three captured the same core features.

**Claude found, others missed:** Author identification (Don LaCourse, 20+ years), publication date (July 10, 2017), software identity (RhinoCAM plugin for Rhino), manual mode details (Bridge Points on a Curve Region with Add/Edit/Delete), most connections (RES-006, RES-027, RES-010, RES-009).

| Agent | Thoroughness (1-5) | Accuracy (1-5) |
|-------|-------------------|-----------------|
| Codex | 3 | 5 |
| Gemini | 4 | 5 |
| Claude | 5 | 5 |

---

### RES-006: Fusion 360 Tabs Reference

**Agreement (all 3):** Rectangular and Triangular tab shapes. "By distance" (automatic) and "At points" (manual) modes. Key parameters: Tab Width, Tab Height, Tab Distance. Visual inspection recommended after auto placement. No adjacent/nested part tab support.

**Disagreements:** Codex vaguely mentions "through-cut compensation" and "compensation-mode caveat"; Gemini and Claude specify the detail: set bottom to -0.05" below stock, Control Compensation must be "Computer."

**Codex found, Claude missed:** Nothing.

**Gemini found, Claude missed:** Nothing.

**Claude found, others missed:** Specific recommended tab height (0.1125"), exact workaround details, market positioning ("hobbyist and small-shop CNC market"), "At points" mode explicitly called "fully manual — no smart suggestions."

| Agent | Thoroughness (1-5) | Accuracy (1-5) |
|-------|-------------------|-----------------|
| Codex | 3 | 5 |
| Gemini | 4 | 5 |
| Claude | 5 | 5 |

---

### RES-007: Fusion 360 Tabs Tutorial

**Agreement (all 3):** Community tutorial from CAD CAM Lessons mirroring official docs. Same shapes, modes, parameters as RES-006. Medium relevance, Medium credibility. No novel algorithmic insights. Derivative of RES-006.

**Disagreements:** None of substance.

**Codex found, Claude missed:** UX observation: users "expect immediate geometric feedback when tab parameters change and easy switching between auto and manual placement."

**Gemini found, Claude missed:** G-code-level implementation detail: "For a rectangular tab, the tool retracts to the tab height, moves laterally for the tab width, and plunges back to the cut depth. For a triangular tab, it performs an angled ramp up and an angled ramp down." Neither Codex nor Claude captured this.

**Claude found, others missed:** Post-machining tab removal methods (cutting, sanding, filing). More connections (RES-006, RES-008, RES-005).

| Agent | Thoroughness (1-5) | Accuracy (1-5) |
|-------|-------------------|-----------------|
| Codex | 3 | 5 |
| Gemini | 4 | 5 |
| Claude | 4 | 5 |

---

### RES-008: Fusion 360 Adjacent Tabs Forum

**Agreement (all 3):** Thread addresses the core "shared tab" problem — tabs on one part destroyed when cutting adjacent part. Fusion 360 has no automatic solution. Workarounds are manual (tab alignment, increased spacing). Per-part tab computation is root cause. High relevance, Medium credibility.

**Disagreements:** Claude claims this is "arguably the most important resource for Thetta's feature design" — editorial claim others don't make.

**Codex found, Claude missed:** "Shared-edge/tab graph model" with "synchronized hold regions and delayed release ordering" — a design concept not in Claude's report.

**Gemini found, Claude missed:** Z-height awareness framing: "the toolpath generator for Part B is unaware of the safe Z-height regions (tabs) established for Part A."

**Claude found, others missed:** Specific forum workaround (manual edge-by-edge path selection with 1/4" gap) as distinct third workaround. More cross-references (RES-019, RES-018, RES-024).

| Agent | Thoroughness (1-5) | Accuracy (1-5) |
|-------|-------------------|-----------------|
| Codex | 3 | 4 |
| Gemini | 4 | 5 |
| Claude | 5 | 5 |

---

### RES-009: VCarve/Aspire Tab Customization

**Agreement (all 3):** Dual-mode workflow (automatic with min/max constraints + interactive manual editing). 3D tabs (smooth ramped motion vs. retract/plunge) are a standout feature. Tab sizes set on toolpath screen.

**Disagreements:** Credibility: Gemini rates High, Codex and Claude rate Medium.

**Codex found, Claude missed:** Nothing significant.

**Gemini found, Claude missed:** Actual G-code difference between standard and 3D tabs (`G0 Z_up; G1 X_move; G1 Z_down` vs. coordinated `G1 X_move Z_up`), making the mechanical distinction concrete and implementable.

**Claude found, others missed:** Explicit note that VCarve has no support for adjacent/shared tabs. Product category identification (sign making, furniture). More cross-references (RES-010, RES-027).

| Agent | Thoroughness (1-5) | Accuracy (1-5) |
|-------|-------------------|-----------------|
| Codex | 3 | 4 |
| Gemini | 4 | 5 |
| Claude | 5 | 4 |

---

### RES-010: CamBam Holding Tabs

**Agreement (all 3):** Core formula: `count = round(perimeter / tab_distance)`, clamped to `[min_tabs, max_tabs]`. Size threshold skips tabs for small shapes. One of the most explicit algorithmic descriptions in commercial CAM documentation.

**Disagreements:** Credibility: Claude rates High; Codex and Gemini rate Medium. Claude provides a concrete example (160mm perimeter, 30mm distance = 5 tabs) that may be fabricated since page fetch failed.

**Codex found, Claude missed:** Nothing.

**Gemini found, Claude missed:** Even spacing "doesn't account for corners or sharp geometry, likely placing tabs on unsuitable vertices" — a specific failure mode. Expired certificate indicates "older software."

**Claude found, others missed:** Concrete numerical example. Observation that CamBam's formula "unifies" count-vs-distance (referencing MecSoft's separate approach). Most connections (RES-001, 005, 009, 021, 011).

| Agent | Thoroughness (1-5) | Accuracy (1-5) |
|-------|-------------------|-----------------|
| Codex | 3 | 4 |
| Gemini | 4 | 5 |
| Claude | 5 | 4 |

---

### RES-011: FreeCAD DressupTag

**Agreement (all 3):** DressupTag is a post-processing "dressup" applied to existing 2D contour path. Even spacing along contour starting from longest edge. Parameters: Angle, Height, Radius, Width. Convex-section placement preference for easier filing.

**Disagreements:** Minimal. Codex saves the "touchiest dressup" quote for RES-012; Claude includes it here.

**Codex found, Claude missed:** Nothing.

**Gemini found, Claude missed:** "Applying tabs to the final path rather than the base geometry can cause issues if the path has already been optimized for something else" — an architectural critique not framed as clearly by others.

**Claude found, others missed:** Includes maintainer quotes directly. Python language and LGPL license identification. Tag shape described as "more flexible than rectangular/triangular dichotomy."

| Agent | Thoroughness (1-5) | Accuracy (1-5) |
|-------|-------------------|-----------------|
| Codex | 3 | 4 |
| Gemini | 4 | 5 |
| Claude | 5 | 5 |

---

### RES-012: FreeCAD DressupTag Source Code

**Agreement (all 3):** Key functions `findBottomWire()` and `generateTags()`. Edge-length-proportional distribution via `edgeDict`. Sliptonic's "touchiest of all the dressups" quote. Fragility from geometry reconstruction from G-code.

**Disagreements:** Minimal. Highly consistent across all three.

**Codex found, Claude missed:** Nothing.

**Gemini found, Claude missed:** "Proportional length distribution does not account for cutting forces, part weight, or the physical mechanics of the routing process" — specific criticism of algorithm omissions.

**Claude found, others missed:** Additional quote that author "probably should have never added the automatic tag generation." Specific complexity source: "linear/arc segments to wire" geometry reconstruction. Insight that Thetta avoids this problem because it has native geometry (not imported G-code).

| Agent | Thoroughness (1-5) | Accuracy (1-5) |
|-------|-------------------|-----------------|
| Codex | 3 | 4 |
| Gemini | 4 | 5 |
| Claude | 5 | 5 |

---

### RES-013: bCNC Repo

**Agreement (all 3):** Python-based open-source GRBL sender with built-in CAM including tab support. Critical "only leave selected islands uncut" checkbox issue. Simple/basic compared to dedicated CAM tools.

**Disagreements:** Credibility: Codex and Gemini rate High, Claude rates Medium.

**Codex found, Claude missed:** Nothing.

**Gemini found, Claude missed:** "Island" metaphor as core algorithmic insight — "bCNC treats holding tabs algorithmically as small 'islands' of un-machined material." Workflow observation: "integration of tabs into a CNC sender shows the demand for last-minute fixturing adjustments at the machine interface."

**Claude found, others missed:** 3-tab default specification connected to RES-001 and RES-021. GPL-2.0 license. Additional CAM features list (thread milling, drag knife, etc.).

| Agent | Thoroughness (1-5) | Accuracy (1-5) |
|-------|-------------------|-----------------|
| Codex | 3 | 4 |
| Gemini | 4 | 5 |
| Claude | 5 | 4 |

---

### RES-014: bCNC Tabs Tutorial

**Agreement (all 3):** Workflow: select last pass, execute tab tool, 3 tabs added. "Only leave selected islands uncut" checkbox warning reiterated. Tutorial is brief, lacks algorithmic depth.

**Disagreements:** Credibility: Gemini rates High, Codex and Claude rate Medium.

**Codex found, Claude missed:** Nothing.

**Gemini found, Claude missed:** Last-pass-only tabs limit tab thickness to single pass depth — a practical constraint others don't mention. Connects 3-tab default to anti-rotation minimum from RES-001 and RES-021.

**Claude found, others missed:** "Silent failure" identified as "UX anti-pattern" — design lesson for Thetta: "tab settings should not have hidden dependencies on unrelated features." External PDF guide mention.

| Agent | Thoroughness (1-5) | Accuracy (1-5) |
|-------|-------------------|-----------------|
| Codex | 3 | 4 |
| Gemini | 4 | 5 |
| Claude | 4 | 5 |

---

### RES-015: viaconstructor

**Agreement (all 3):** Python/C++ hybrid (~50%/~46%) CAM tool with tab support, auto offset finding, nearest-neighbor ordering, known bug with circular-path tab generation. Uses PyQt5 + OpenGL.

**Disagreements:** Credibility: Codex and Gemini rate High, Claude rates Medium. Different speculations on circular-path failure cause (Claude: uniform curvature lacking segmentation points; Gemini: arc discretization/normal vector; Codex: contour discretization and offset geometry).

**Codex found, Claude missed:** Nothing.

**Gemini found, Claude missed:** Noted viaconstructor lacks shared-tab logic. Architectural comparison to bCNC noting "more sophisticated C++ backend."

**Claude found, others missed:** Additional input/output format details (HPGL, CDR/PDF/EPS/raster; G-code, HPGL). Python 3.9+ requirement. Headless/batch mode. "No documentation of tab placement algorithm specifics." Four cross-references vs. one each for others.

| Agent | Thoroughness (1-5) | Accuracy (1-5) |
|-------|-------------------|-----------------|
| Codex | 3 | 4 |
| Gemini | 4 | 4 |
| Claude | 5 | 4 |

---

### RES-016: Kiri:Moto

**Agreement (all 3):** Browser-based, open-source, multi-modal CAM (3D printing, CNC, laser) with CNC tab support. 13+ years development. HTML5/WebGL. Self-hostable. Tab algorithm undocumented.

**Disagreements:** Credibility: Codex and Gemini rate High, Claude rates Medium. Gemini argues browser-based CAM can be "efficiently executed in JavaScript"; others are more cautious on performance.

**Codex found, Claude missed:** Nothing.

**Gemini found, Claude missed:** Multi-modal tools often share geometric processing libraries across manufacturing methods (offset logic reused between 3D slicing and CNC contours). Potential performance struggles with "high-vertex-count nests."

**Claude found, others missed:** Nothing significant beyond more detailed formatting and cross-references.

| Agent | Thoroughness (1-5) | Accuracy (1-5) |
|-------|-------------------|-----------------|
| Codex | 2 | 4 |
| Gemini | 4 | 4 |
| Claude | 4 | 4 |

---

### RES-017: Deepnest

**Agreement (all 3):** Open-source nesting tool for laser/plasma/CNC. SVGNest-based with C engine for performance, JavaScript UI. Key feature: common-line merging. No native tab placement. Common-line detection is geometric foundation for shared-tab placement.

**Disagreements:** Credibility: Codex and Gemini rate High, Claude rates Medium.

**Codex found, Claude missed:** Nothing.

**Gemini found, Claude missed:** C++ core's purpose as NFP (No Fit Polygon) calculations. Common-line merging is "highly sought-after" and reduces heat warping.

**Claude found, others missed:** Precise language breakdown (78.5% JS, 1.5% C++). MIT license. "Project maintenance status unclear." "Path approximation may lose precision for tab placement" — unique concern. "Common-line merging optimized for cutting, not for holding" — important distinction.

| Agent | Thoroughness (1-5) | Accuracy (1-5) |
|-------|-------------------|-----------------|
| Codex | 3 | 4 |
| Gemini | 4 | 5 |
| Claude | 5 | 4 |

---

### RES-018: HOMAG Nesting Features

**Agreement (all 3):** HOMAG integrates four strategies: Stay-Down, Common Line, Tabs (retaining bars), Onion Skinning. Tabs target small parts with insufficient vacuum. Common-line spacing = tool diameter. Proprietary, closed-source industrial solution.

**Disagreements:** Gemini claims "Stay-Down pathing relies heavily on tabs" (causal dependency); Claude describes them as complementary, not dependent.

**Codex found, Claude missed:** Nothing.

**Gemini found, Claude missed:** Explicit assertion that Stay-Down relies on tabs (if tool never retracts, every part must be locked). "Solves the problem described in RES-008."

**Claude found, others missed:** Software version requirements (woodWOP 8 + Nesting Plugin, tabs in v8.1.225+). "Most comprehensive integrated approach found in this research." Tab placement integrated into nesting optimization rather than post-processing — meaningful architectural distinction. Five cross-references.

| Agent | Thoroughness (1-5) | Accuracy (1-5) |
|-------|-------------------|-----------------|
| Codex | 3 | 4 |
| Gemini | 4 | 4 |
| Claude | 5 | 4 |

---

### RES-019: WOODWEB Tabs vs. Onion Skinning

**Agreement (all 3):** Forum comparing tabs and onion skinning for CNC woodworking. Key finding: misaligned tabs between adjacent parts get "disconnected" when neighbors are cut. Some CAM systems have tab alignment but operators resort to manual placement. Original source expired.

**Disagreements:** Credibility: Codex rates Low, Gemini and Claude rate Medium.

**Codex found, Claude missed:** Nothing.

**Gemini found, Claude missed:** Two-tool-diameter threshold for when adjacent-tab problem occurs. "Only global path awareness can prevent this." Financial consequences framing for professional shops.

**Claude found, others missed:** Vacuum effectiveness as "central to the workholding decision." More nuanced tabs-vs-onion-skin trade-off discussion. Four cross-references.

| Agent | Thoroughness (1-5) | Accuracy (1-5) |
|-------|-------------------|-----------------|
| Codex | 3 | 4 |
| Gemini | 4 | 4 |
| Claude | 4 | 4 |

---

### RES-020: WOODWEB Tabbing Small Parts

**Agreement (all 3):** Tabs vs. onion skinning for small parts. Thick tabs (~1/4") strong but need sanding. Onion skinning easier to clean but weaker. Two-pass strategy recommended. Decision depends on part size, vacuum, material.

**Disagreements:** No major factual disagreements.

**Codex found, Claude missed:** Framing that heuristics "can become rules in an automatic planner" and serve as "constraints in an optimization model."

**Gemini found, Claude missed:** Physical mechanics: "balancing lateral cutting force against shear strength of tab material and friction of vacuum table." Connection to HOMAG multi-pass and CamBam small-part threshold.

**Claude found, others missed:** Specific vacuum tips (cover unused spoilboard with cardboard, ensure sheet flatness, cut small parts first). Bit selection advice (compression bits for large, down-shear for small).

| Agent | Thoroughness (1-5) | Accuracy (1-5) |
|-------|-------------------|-----------------|
| Codex | 3 | 4 |
| Gemini | 4 | 4 |
| Claude | 5 | 4 |

---

### RES-021: CNCZone Optimize Tabs

**Agreement (all 3):** Minimum 3 tabs to prevent rotation (2 tabs create pivot axis). Place on straight edges or outside convex curves. Common dimensions: ~0.75" × 0.125". Perimeter-division heuristic.

**Disagreements:** Credibility: Codex rates Low, Gemini and Claude rate Medium.

**Codex found, Claude missed:** Nothing.

**Gemini found, Claude missed:** Explicit mechanical explanation of 2-tab failure mode (pivot axis, lateral force causing rotation and snapping). Validation connecting to RES-001 and RES-014. Specific manual heuristic: "divide perimeter into three roughly equal segments, making sure ends don't meet, place tab in center of each segment, adjust to hit straight edge."

**Claude found, others missed:** Metric equivalents (19mm × 3.2mm). Connection to RES-028 for complementary sizing heuristics.

| Agent | Thoroughness (1-5) | Accuracy (1-5) |
|-------|-------------------|-----------------|
| Codex | 3 | 4 |
| Gemini | 5 | 5 |
| Claude | 4 | 4 |

---

### RES-022: PCB Tab Routing Masterclass (AllPCB)

**Agreement (all 3):** 50-75mm tab spacing heuristic. Avoid corners and high-stress areas. Mouse bite specs: 0.5-1.0mm holes, 1.0mm spacing, 5-7 holes per tab. Medium relevance (PCB domain). Connects to RES-023.

**Disagreements:** Credibility: Codex and Claude rate Medium, Gemini rates High.

**Codex found, Claude missed:** Nothing.

**Gemini found, Claude missed:** Connection of 50-75mm spacing to Fusion 360's "By Distance" method (RES-006).

**Claude found, others missed:** Scaling rule for larger boards (+1 tab per 50mm for boards >200mm). Tab width range (0.2-0.5mm for 1.6mm FR-4). FR-4 tensile strength (310 MPa). Four cross-references.

| Agent | Thoroughness (1-5) | Accuracy (1-5) |
|-------|-------------------|-----------------|
| Codex | 2 | 4 |
| Gemini | 4 | 5 |
| Claude | 5 | 5 |

---

### RES-023: Altium Mouse Bites and V-Scores

**Agreement (all 3):** Covers mouse bites and V-scores as PCB depanelization methods. Low relevance for CNC nesting. Mouse bites use tangential hole placement. Connects to RES-022.

**Disagreements:** Credibility: Codex rates Medium, Gemini and Claude rate High.

**Codex found, Claude missed:** Nothing.

**Gemini found, Claude missed:** V-scoring is "strictly limited to straight lines that traverse the entire panel" — geometric constraint not emphasized by Claude.

**Claude found, others missed:** 1.5-2mm routing area around mouse bites. Design rule: hole-wall spacing < hole diameter. Panel design guidance (tooling holes, fiducials, test coupons). Connection to RES-024.

| Agent | Thoroughness (1-5) | Accuracy (1-5) |
|-------|-------------------|-----------------|
| Codex | 2 | 4 |
| Gemini | 3 | 5 |
| Claude | 4 | 5 |

---

### RES-024: When to Cut a Common Line (The Fabricator)

**Agreement (all 3):** Common-line cutting saves material and reduces cut count. Microtabs retain parts along shared edges. Thermal cutting introduces distortion/tolerance issues. Kerf from one piece must not overlap another. Medium relevance.

**Disagreements:** Credibility: Codex and Gemini rate Medium, Claude rates High. Access issues: Codex and Claude note 403 fetch failure; Gemini does not mention.

**Codex found, Claude missed:** Nothing.

**Gemini found, Claude missed:** "Parts tabbed together on a common line act as a single structural unit" — cascading error propagation framing.

**Claude found, others missed:** Distinction between thermal and cold cutting (CNC routing, waterjet) — thermal concerns don't apply to cold cutting. Restart challenges after unexpected shutdowns. Four cross-references. Explicit framing as "exactly the shared-tab scenario that Thetta aims to support."

| Agent | Thoroughness (1-5) | Accuracy (1-5) |
|-------|-------------------|-----------------|
| Codex | 2 | 4 |
| Gemini | 4 | 5 |
| Claude | 5 | 5 |

---

### RES-025: Mate Precision Technologies Tabbing Practices

**Agreement (all 3):** Resource covers 5-axis CNC tabbing, not 2D nesting. Four critical factors: Tab Location, CAM Support, Workholding Setup, Tab Geometry. Tabs on non-critical/non-machined surfaces. Size depends on material and cutting forces. Medium relevance.

**Disagreements:** Credibility: Codex rates Medium, Gemini and Claude rate High.

**Codex found, Claude missed:** Framing as potential "scoring terms" for algorithmic implementation (penalize critical faces, increase cross-section under load, validate accessibility).

**Gemini found, Claude missed:** Nothing substantive beyond complementary detail ("dovetail fixtures" vs. Claude's "DynoMount risers").

**Claude found, others missed:** "DynoMount risers" product name. "No discussion of shared tabs or adjacent parts." Four cross-references (RES-021, RES-028, RES-020, RES-005).

| Agent | Thoroughness (1-5) | Accuracy (1-5) |
|-------|-------------------|-----------------|
| Codex | 2 | 4 |
| Gemini | 4 | 5 |
| Claude | 5 | 5 |

---

### RES-026: SendCutSend CNC Routing Tabs

**Agreement (all 3):** Fully automatic, geometry-based tab placement with no user customization. Tabs/lead-ins placed outside part geometry to preserve dimensional accuracy. Validates that fully automatic tab placement is commercially viable. Algorithm is a black box.

**Disagreements:** Credibility: Codex rates High, Claude rates Medium (calls it "commercial marketing page").

**Codex found, Claude missed:** Nothing.

**Gemini found, Claude missed:** Explicit contrast with "manual-override philosophy seen in traditional CAM software like VCarve (RES-009) and FreeCAD (RES-011)."

**Claude found, others missed:** Outside-geometry approach "requires additional stock material." No information on shared edges/adjacent parts. Most connections.

| Agent | Thoroughness (1-5) | Accuracy (1-5) |
|-------|-------------------|-----------------|
| Codex | 2 | 4 |
| Gemini | 3 | 5 |
| Claude | 4 | 5 |

---

### RES-027: Creating and Using Tabs in Mastercam (CamInstructor)

**Agreement (all 3):** Mastercam offers automatic (by count or max distance) and manual (point-based) tab placement. Tab thickness, width, and side profile angle are configurable. Angled profile creates ramped/chamfered tabs. Industry-standard professional CAM.

**Disagreements:** Credibility: Codex and Claude rate Medium, Gemini rates High.

**Codex found, Claude missed:** Nothing.

**Gemini found, Claude missed:** "Customizing the tab angle (ramping) is a critical feature for reducing tool marks and machine wear."

**Claude found, others missed:** Analogy between Mastercam's dual modes and CamBam's perimeter-division formula. Right-click "Edit Tabs" UI detail. No corner avoidance or geometry-aware placement. Five cross-references.

| Agent | Thoroughness (1-5) | Accuracy (1-5) |
|-------|-------------------|-----------------|
| Codex | 2 | 4 |
| Gemini | 4 | 5 |
| Claude | 5 | 5 |

---

### RES-028: Maslow CNC Tabs Strategy Discussion

**Agreement (all 3):** Tab sizing scales with material thickness. Multi-pass cutting for tab retention. Example: 0.255" tabs for 0.500" material. Low relevance. Connects to RES-020.

**Disagreements:** Credibility: Codex rates Low, Gemini and Claude rate Medium. Gemini mentions "two separate toolpaths"; Claude describes three passes.

**Codex found, Claude missed:** Nothing.

**Gemini found, Claude missed:** Emphasis on vertical CNC orientation making gravity a critical factor — "vertical or unconventional setups require significantly more robust tabbing due to gravity."

**Claude found, others missed:** 50% material-thickness ratio formulation. Complete removal tool list with prices (oscillating multi-tool $19, scissors $7). Computed tab width example (0.38" from 1.5× ratio). "Tab size should be sized for the removal tool the user has available."

| Agent | Thoroughness (1-5) | Accuracy (1-5) |
|-------|-------------------|-----------------|
| Codex | 2 | 4 |
| Gemini | 4 | 5 |
| Claude | 5 | 5 |

---

## Summary

### Agent Profiles

**Claude (self-assessment):**
- **Strengths:** Most thorough across nearly all resources. Consistently provides full citation details, the most cross-references (3-5 per resource vs. 1 each for others), explicit gap analysis ("no shared tab support," "no corner avoidance"), and Thetta-specific framing.
- **Weaknesses:** Occasionally verbose without proportional insight gain. Sometimes rates credibility lower than warranted (or higher — CamBam as High despite fetch failure). Some concrete details (e.g., CamBam numerical example) may be fabricated from incomplete source access. Tends to underrate community/code sources (Medium where others rate High).
- **Average scores:** Thoroughness 4.7, Accuracy 4.6

**Codex:**
- **Strengths:** Concise and accurate at a high level. Occasionally provides unique algorithmic framing (chromosome encoding extensions in RES-003, scoring terms in RES-025, optimization model framing in RES-020).
- **Weaknesses:** Systematically the least detailed — ~30 lines per report in later resources. Omits specific numbers, ratios, and cross-references. Rates community sources' credibility conservatively (often Low).
- **Average scores:** Thoroughness 2.6, Accuracy 4.1

**Gemini:**
- **Strengths:** Best at extracting practical/mechanical insights — G-code details (RES-007, RES-009), physical mechanics (two-tool-diameter threshold in RES-019, pivot axis failure in RES-021, gravity in RES-028), computational cost concerns (RES-003), and cascading error risks (RES-024). Consistently balanced between breadth and depth.
- **Weaknesses:** Occasionally makes strong assertions not fully supported by source material (Stay-Down dependency on tabs in RES-018). Tends to rate credibility higher than warranted for community sources.
- **Average scores:** Thoroughness 3.9, Accuracy 4.7

### Strongest Consensus Findings (High Confidence)

1. **Minimum 3 tabs per part** — agreed across academic (RES-001), open-source defaults (RES-013), and community heuristics (RES-021). 2-tab configurations create a pivot axis and fail under lateral force.
2. **Perimeter-proportional placement** is the universal baseline algorithm — CamBam (RES-010), FreeCAD (RES-011/012), Mastercam (RES-027) all use `count = perimeter / distance`.
3. **Adjacent/shared tab placement is an unsolved problem in mainstream CAM** — Fusion 360 (RES-008), VCarve (RES-009), FreeCAD (RES-011), bCNC (RES-013) all lack it. Only HOMAG (RES-018) integrates it, and that's proprietary.
4. **Tab shapes converge on Rectangular vs. Triangular** with angle/ramp as the key variant — MecSoft (RES-005), Fusion 360 (RES-006), VCarve 3D tabs (RES-009), Mastercam (RES-027).
5. **Placement on straight edges preferred over corners** — CNCZone (RES-021), FreeCAD convex-section preference (RES-011), PCB corner avoidance (RES-022).
6. **Tab placement is universally treated as post-processing** in open-source tools — only HOMAG integrates it into nesting optimization.

### Biggest Disagreements / Flags for Resolution

1. **Stay-Down dependency on tabs (RES-018):** Gemini claims causal dependency; Claude describes complementary relationship. Needs verification from HOMAG documentation.
2. **Circular-path tab failure cause (RES-015):** Three different speculations (curvature, discretization, offset geometry). None verified.
3. **Credibility ratings for community sources:** Codex systematically rates Low, Claude rates Medium, Gemini rates High. Need a consistent framework.
4. **Two-pass vs. three-pass tab strategy (RES-028):** Gemini says two toolpaths, Claude says three. Source material should be re-checked.
5. **CamBam numerical example accuracy (RES-010):** Claude provides 160mm/30mm = 5 tabs, but the source page failed to load. May be fabricated.

### Resources Where Reports Diverge Most

1. **RES-018 (HOMAG)** — highest disagreement on the Stay-Down/tabs relationship and on how integrated the tab-nesting optimization really is.
2. **RES-021 (CNCZone)** — Gemini provided significantly more mechanical detail (pivot failure, manual heuristic) than either Codex or Claude. Gemini scores 5/5 here while Codex scores 3/4.
3. **RES-015 (viaconstructor)** — three different explanations for the same bug; credibility ratings span Medium to High.
4. **RES-019 (WOODWEB)** — Gemini provides the two-tool-diameter threshold that neither other agent captures; this is a concrete, actionable design parameter.
5. **RES-010 (CamBam)** — Claude's concrete numbers may not be verifiable; Gemini's corner-placement critique is absent from Claude's report.

### Recommendations

1. **Re-verify paywalled papers** (RES-001 through RES-004) if full access becomes available — all agents are working from abstracts.
2. **Adopt Gemini's two-tool-diameter threshold** (RES-019) as a design heuristic for shared-tab conflict detection.
3. **Validate CamBam algorithm** (RES-010) against actual CamBam installation if possible.
4. **Clarify HOMAG Stay-Down/tabs relationship** (RES-018) before using it as architectural reference.
5. **Standardize credibility framework** — community forums with specific technical detail (CNCZone, WOODWEB) deserve Medium, not Low or High.
