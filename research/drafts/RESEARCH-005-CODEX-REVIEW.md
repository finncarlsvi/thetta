# RESEARCH-005 Cross-Reference Review (Claude vs Gemini vs Codex)

Compared each `RES-MMM` report across the three agents using reported key findings, limitations, and specificity level.

## RES-001-micro-joints-genetic-algorithm

**Where all three agree (high confidence):**
- All three agree on the core problem framing and practical relevance; differences are mainly in level of detail.

**Where agents disagree (flag for resolution):**
- Different emphasis: Claude/Gemini include extra implementation detail not present in Codex.

**What Claude/Gemini found that Codex missed:**
- Three micro-joints per part is sufficient for stable holding — placed at 120° intervals from the centroid
- Micro-joint count directly correlates with piercing point count, which dominates non-productive tool travel time

**What Codex found that Claude/Gemini missed:**
- The method places three micro-joints by centroid-derived orientation and uses a GA to optimize projection angle plus cutting sequence to reduce non-productive travel.
- The authors state micro-joint count and placement strongly affect piercing points, toolpath length, and process stability (part drop/detachment risk).

**Ratings (1-5):**
- Claude: thoroughness 5/5, accuracy 4/5
- Gemini: thoroughness 4/5, accuracy 4/5
- Codex: thoroughness 3/5, accuracy 4/5

## RES-002-common-edge-nested-cutting

**Where all three agree (high confidence):**
- Common-edge nesting reduces spacing between parts and improves sheet utilization rate compared to traditional nesting
- Common-edge nesting fundamentally alters cutting path optimization by introducing shared segments that cannot be cut twice.

**Where agents disagree (flag for resolution):**
- Different emphasis: Claude/Gemini include extra implementation detail not present in Codex.

**What Claude/Gemini found that Codex missed:**
- Cutting path optimization for shared-edge parts requires consideration of part dependencies (can't fully cut a part before its neighbor if they share an edge)
- The approach optimizes both the sequence of parts and the paths between them

**What Codex found that Claude/Gemini missed:**
- Sequence/path optimization under shared-edge constraints is treated as a specialized problem, not just a direct extension of conventional nesting.

**Ratings (1-5):**
- Claude: thoroughness 5/5, accuracy 4/5
- Gemini: thoroughness 4/5, accuracy 4/5
- Codex: thoroughness 3/5, accuracy 4/5

## RES-003-cutting-path-two-step-ga

**Where all three agree (high confidence):**
- CNC cutting path optimization is formally a generalized TSP with piercing point selection
- CNC cutting path optimization can be effectively modeled as a Generalized Traveling Salesman Problem (GTSP).

**Where agents disagree (flag for resolution):**
- Different emphasis: Claude/Gemini include extra implementation detail not present in Codex.

**What Claude/Gemini found that Codex missed:**
- Two-step decomposition (piercing points + sequencing) is effective for this problem class
- Piercing point location directly relates to micro-joint/tab placement

**What Codex found that Claude/Gemini missed:**
- A two-step GA is used to combine broader search over piercing choices with sequencing improvements for travel reduction.

**Ratings (1-5):**
- Claude: thoroughness 5/5, accuracy 4/5
- Gemini: thoroughness 4/5, accuracy 5/5
- Codex: thoroughness 3/5, accuracy 4/5

## RES-004-tool-path-optimization-models

**Where all three agree (high confidence):**
- All three agree on the core problem framing and practical relevance; differences are mainly in level of detail.

**Where agents disagree (flag for resolution):**
- Different emphasis: Claude/Gemini include extra implementation detail not present in Codex.

**What Claude/Gemini found that Codex missed:**
- Tool path problems for CNC cutting can be classified into distinct mathematical models
- Most formulations reduce to GTSP variants with domain-specific constraints

**What Codex found that Claude/Gemini missed:**
- Toolpath optimization is treated as a family of formal models, including discrete formulations like GTSP variants.

**Ratings (1-5):**
- Claude: thoroughness 5/5, accuracy 4/5
- Gemini: thoroughness 4/5, accuracy 4/5
- Codex: thoroughness 3/5, accuracy 4/5

## RES-005-mecsoft-bridges-tabs

**Where all three agree (high confidence):**
- All three agree on the core problem framing and practical relevance; differences are mainly in level of detail.

**Where agents disagree (flag for resolution):**
- Type label mismatch: Claude `Commercial`, Gemini `Commercial`, Codex `Product`.
- Different emphasis: Claude/Gemini include extra implementation detail not present in Codex.

**What Claude/Gemini found that Codex missed:**
- Two bridge shapes: Triangular (easier removal) and Rectangular (stronger hold)
- Placement by count OR by distance — two distinct automatic distribution methods

**What Codex found that Claude/Gemini missed:**
- MecSoft exposes triangular/rectangular bridges with configurable height/length and either count-based or spacing-based placement.
- The workflow supports both automatic insertion during toolpath generation and explicit manual bridge-point editing.

**Ratings (1-5):**
- Claude: thoroughness 5/5, accuracy 4/5
- Gemini: thoroughness 4/5, accuracy 4/5
- Codex: thoroughness 3/5, accuracy 4/5

## RES-006-fusion360-tabs-reference

**Where all three agree (high confidence):**
- All three agree on the core problem framing and practical relevance; differences are mainly in level of detail.

**Where agents disagree (flag for resolution):**
- Type label mismatch: Claude `Commercial`, Gemini `Commercial`, Codex `Product`.
- Different emphasis: Claude/Gemini include extra implementation detail not present in Codex.

**What Claude/Gemini found that Codex missed:**
- Tab shapes: Rectangular and Triangular
- Recommended tab height for through cuts: 0.1125"

**What Codex found that Claude/Gemini missed:**
- Autodesk guidance includes process-specific recommendations (through-cut depth offsets and compensation settings) that affect tab reliability.

**Ratings (1-5):**
- Claude: thoroughness 5/5, accuracy 4/5
- Gemini: thoroughness 4/5, accuracy 5/5
- Codex: thoroughness 3/5, accuracy 4/5

## RES-007-fusion360-tabs-tutorial

**Where all three agree (high confidence):**
- All three agree on the core problem framing and practical relevance; differences are mainly in level of detail.

**Where agents disagree (flag for resolution):**
- Different emphasis: Claude/Gemini include extra implementation detail not present in Codex.

**What Claude/Gemini found that Codex missed:**
- Tabs work by raising the cutting tool at specified locations, leaving thin material layers
- Two shapes: Rectangular (consistent support) and Triangular (easier removal)

**What Codex found that Claude/Gemini missed:**
- The tutorial presents two shape options and two positioning modes consistent with Fusion’s production workflow.
- It highlights width/height/distance as the key practical tuning triad for retention versus post-cut cleanup.

**Ratings (1-5):**
- Claude: thoroughness 5/5, accuracy 4/5
- Gemini: thoroughness 4/5, accuracy 4/5
- Codex: thoroughness 3/5, accuracy 4/5

## RES-008-fusion360-adjacent-tabs-forum

**Where all three agree (high confidence):**
- Tabs on shared edges between adjacent parts are destroyed when the neighboring part is cut
- No existing CAM software automatically aligns tabs between adjacent parts

**Where agents disagree (flag for resolution):**
- Different emphasis: Claude/Gemini include extra implementation detail not present in Codex.

**What Claude/Gemini found that Codex missed:**
- Tabs must be placed at identical positions on both sides of a shared edge to survive
- The 1/4" gap between parts is a practical minimum for some workaround approaches

**What Codex found that Claude/Gemini missed:**
- No clear unique finding; Codex is mostly a concise restatement.

**Ratings (1-5):**
- Claude: thoroughness 5/5, accuracy 4/5
- Gemini: thoroughness 4/5, accuracy 5/5
- Codex: thoroughness 3/5, accuracy 5/5

## RES-009-vcarve-tab-customization

**Where all three agree (high confidence):**
- All three agree on the core problem framing and practical relevance; differences are mainly in level of detail.

**Where agents disagree (flag for resolution):**
- Different emphasis: Claude/Gemini include extra implementation detail not present in Codex.

**What Claude/Gemini found that Codex missed:**
- Automatic placement with min/max tab count and distance constraints
- Interactive editing: drag to reposition, click to add/delete even on auto-placed tabs

**What Codex found that Claude/Gemini missed:**
- Vectric workflows support automatic tab generation with configurable bounds and interactive manual adjustment.
- 3D tab behavior can reduce abrupt motion artifacts and improve cleanup characteristics versus simple step tabs.

**Ratings (1-5):**
- Claude: thoroughness 5/5, accuracy 4/5
- Gemini: thoroughness 4/5, accuracy 4/5
- Codex: thoroughness 3/5, accuracy 4/5

## RES-010-cambam-holding-tabs

**Where all three agree (high confidence):**
- **Tab count formula**: tabs = round(perimeter ÷ tab_distance), clamped to [min_tabs, max_tabs]
- A standard algorithm for tab count is: `Count = round(Perimeter / Tab_Distance)`.

**Where agents disagree (flag for resolution):**
- Type label mismatch: Claude `Commercial`, Gemini `Commercial`, Codex `Product`.
- Different emphasis: Claude/Gemini include extra implementation detail not present in Codex.

**What Claude/Gemini found that Codex missed:**
- **Example**: 160mm perimeter, 30mm distance = 5 tabs
- **Parameters**: Tab Distance, Min Tabs, Max Tabs, Size Threshold, Tab Width, Tab Depth, Tab Style

**What Codex found that Claude/Gemini missed:**
- No clear unique finding; Codex is mostly a concise restatement.

**Ratings (1-5):**
- Claude: thoroughness 5/5, accuracy 4/5
- Gemini: thoroughness 4/5, accuracy 4/5
- Codex: thoroughness 3/5, accuracy 5/5

## RES-011-freecad-dressuptag

**Where all three agree (high confidence):**
- FreeCAD auto-generates tags with even spacing along the contour and prioritizes placement from the longest edge.

**Where agents disagree (flag for resolution):**
- Different emphasis: Claude/Gemini include extra implementation detail not present in Codex.

**What Claude/Gemini found that Codex missed:**
- Dressup pattern: modify existing toolpath rather than generate from scratch
- Placement preference for convex sections (easier removal)

**What Codex found that Claude/Gemini missed:**
- No clear unique finding; Codex is mostly a concise restatement.

**Ratings (1-5):**
- Claude: thoroughness 5/5, accuracy 4/5
- Gemini: thoroughness 4/5, accuracy 4/5
- Codex: thoroughness 3/5, accuracy 5/5

## RES-012-freecad-dressuptag-source

**Where all three agree (high confidence):**
- All three agree on the core problem framing and practical relevance; differences are mainly in level of detail.

**Where agents disagree (flag for resolution):**
- Different emphasis: Claude/Gemini include extra implementation detail not present in Codex.

**What Claude/Gemini found that Codex missed:**
- `findBottomWire()`: extracts lowest-Z edges, constructs closed wire for tab placement
- `generateTags(obj, count)`: distributes tabs proportional to edge length via `edgeDict`

**What Codex found that Claude/Gemini missed:**
- Automatic tag distribution appears to allocate counts by edge length using an edge dictionary and longest-edge-first logic.
- Maintainer commentary flags significant complexity and reliability issues in automatic generation workflows.

**Ratings (1-5):**
- Claude: thoroughness 5/5, accuracy 4/5
- Gemini: thoroughness 4/5, accuracy 4/5
- Codex: thoroughness 3/5, accuracy 4/5

## RES-013-bcnc-repo

**Where all three agree (high confidence):**
- All three agree on the core problem framing and practical relevance; differences are mainly in level of detail.

**Where agents disagree (flag for resolution):**
- Different emphasis: Claude/Gemini include extra implementation detail not present in Codex.

**What Claude/Gemini found that Codex missed:**
- Default: 3 tabs added to last pass of cutting operation
- Tab feature integrated into the "cut" tool, not standalone

**What Codex found that Claude/Gemini missed:**
- bCNC includes native cutout tab support within its CAM toolchain and defaults to simple tab insertion behavior.

**Ratings (1-5):**
- Claude: thoroughness 5/5, accuracy 4/5
- Gemini: thoroughness 4/5, accuracy 4/5
- Codex: thoroughness 3/5, accuracy 4/5

## RES-014-bcnc-tabs-tutorial

**Where all three agree (high confidence):**
- All three agree on the core problem framing and practical relevance; differences are mainly in level of detail.

**Where agents disagree (flag for resolution):**
- Different emphasis: Claude/Gemini include extra implementation detail not present in Codex.

**What Claude/Gemini found that Codex missed:**
- Critical: must uncheck "only leave selected islands uncut" in cut tool settings
- External PDF guide available for detailed configuration

**What Codex found that Claude/Gemini missed:**
- A specific cut-tool option must be disabled for tabs to apply as intended in many cases.

**Ratings (1-5):**
- Claude: thoroughness 5/5, accuracy 4/5
- Gemini: thoroughness 4/5, accuracy 4/5
- Codex: thoroughness 3/5, accuracy 4/5

## RES-015-viaconstructor

**Where all three agree (high confidence):**
- All three agree on the core problem framing and practical relevance; differences are mainly in level of detail.

**Where agents disagree (flag for resolution):**
- Different emphasis: Claude/Gemini include extra implementation detail not present in Codex.

**What Claude/Gemini found that Codex missed:**
- Automatic offset finding (inside/outside) and nearest-neighbor ordering
- Supports nested part handling with correct sequencing

**What Codex found that Claude/Gemini missed:**
- No clear unique finding; Codex is mostly a concise restatement.

**Ratings (1-5):**
- Claude: thoroughness 5/5, accuracy 4/5
- Gemini: thoroughness 4/5, accuracy 4/5
- Codex: thoroughness 3/5, accuracy 5/5

## RES-016-kirimoto

**Where all three agree (high confidence):**
- Browser-based CAM with CNC tab support, running entirely client-side
- Tabs are a necessary feature even for lightweight, browser-based CNC tools.

**Where agents disagree (flag for resolution):**
- Different emphasis: Claude/Gemini include extra implementation detail not present in Codex.

**What Claude/Gemini found that Codex missed:**
- 13+ years of development — mature project
- Multi-modal: 3D printing, CNC milling, laser cutting in one tool

**What Codex found that Claude/Gemini missed:**
- The project is open-source, self-hostable, and mature, implying extensive field usage.

**Ratings (1-5):**
- Claude: thoroughness 5/5, accuracy 4/5
- Gemini: thoroughness 4/5, accuracy 4/5
- Codex: thoroughness 3/5, accuracy 4/5

## RES-017-deepnest

**Where all three agree (high confidence):**
- Common-line merging automatically identifies and unifies shared cut paths between adjacent parts

**Where agents disagree (flag for resolution):**
- Different emphasis: Claude/Gemini include extra implementation detail not present in Codex.

**What Claude/Gemini found that Codex missed:**
- No explicit tab/bridge placement feature
- Common-line concept is the geometric foundation for shared-tab placement

**What Codex found that Claude/Gemini missed:**
- The tool focuses on material/time efficiency and thermal improvement, but does not document explicit tab algorithms.

**Ratings (1-5):**
- Claude: thoroughness 5/5, accuracy 4/5
- Gemini: thoroughness 4/5, accuracy 4/5
- Codex: thoroughness 3/5, accuracy 4/5

## RES-018-homag-nesting-features

**Where all three agree (high confidence):**
- Tabs specifically target small parts with insufficient vacuum hold-down

**Where agents disagree (flag for resolution):**
- Type label mismatch: Claude `Commercial`, Gemini `Commercial`, Codex `Product`.
- Different emphasis: Claude/Gemini include extra implementation detail not present in Codex.

**What Claude/Gemini found that Codex missed:**
- Tab placement integrated into nesting optimization (not post-processing)
- Common line = single cut separates two parts (spacing = tool diameter)

**What Codex found that Claude/Gemini missed:**
- No clear unique finding; Codex is mostly a concise restatement.

**Ratings (1-5):**
- Claude: thoroughness 5/5, accuracy 4/5
- Gemini: thoroughness 4/5, accuracy 4/5
- Codex: thoroughness 3/5, accuracy 5/5

## RES-019-woodweb-tabs-vs-onionskin

**Where all three agree (high confidence):**
- All three agree on the core problem framing and practical relevance; differences are mainly in level of detail.

**Where agents disagree (flag for resolution):**
- Different emphasis: Claude/Gemini include extra implementation detail not present in Codex.

**What Claude/Gemini found that Codex missed:**
- Some CAD/CAM systems have tabbing options ensuring alignment with adjacent part tabs
- Tabs vs. onion skinning trade-off depends on part size, material, vacuum quality

**What Codex found that Claude/Gemini missed:**
- No clear unique finding; Codex is mostly a concise restatement.

**Ratings (1-5):**
- Claude: thoroughness 5/5, accuracy 4/5
- Gemini: thoroughness 4/5, accuracy 4/5
- Codex: thoroughness 3/5, accuracy 5/5

## RES-020-woodweb-tabbing-small-parts

**Where all three agree (high confidence):**
- All three agree on the core problem framing and practical relevance; differences are mainly in level of detail.

**Where agents disagree (flag for resolution):**
- Different emphasis: Claude/Gemini include extra implementation detail not present in Codex.

**What Claude/Gemini found that Codex missed:**
- Onion skinning: cut to 0.70" in 0.75" sheet, then full depth for final pass
- Cover unused spoilboard with cardboard to concentrate vacuum on work area

**What Codex found that Claude/Gemini missed:**
- Two-pass onion-skin approaches and vacuum-optimization tactics are practical alternatives for instability-prone layouts.

**Ratings (1-5):**
- Claude: thoroughness 5/5, accuracy 4/5
- Gemini: thoroughness 4/5, accuracy 4/5
- Codex: thoroughness 3/5, accuracy 4/5

## RES-021-cnczone-optimize-tabs

**Where all three agree (high confidence):**
- All three agree on the core problem framing and practical relevance; differences are mainly in level of detail.

**Where agents disagree (flag for resolution):**
- Different emphasis: Claude/Gemini include extra implementation detail not present in Codex.

**What Claude/Gemini found that Codex missed:**
- Minimum 3 tabs to prevent rotation (2 tabs allow rotation around connecting line)
- Place tabs on straight edges or outside curves for easy sanding access

**What Codex found that Claude/Gemini missed:**
- Practitioners suggest a minimum of 3-4 tabs for small isolated shapes to reduce rotational instability.
- Common starter sizing is around `0.75 in` length by `0.125 in` height, adjusted by material and project requirements.

**Ratings (1-5):**
- Claude: thoroughness 5/5, accuracy 4/5
- Gemini: thoroughness 4/5, accuracy 4/5
- Codex: thoroughness 3/5, accuracy 4/5

## RES-022-pcb-tab-routing-masterclass

**Where all three agree (high confidence):**
- All three agree on the core problem framing and practical relevance; differences are mainly in level of detail.

**Where agents disagree (flag for resolution):**
- Different emphasis: Claude/Gemini include extra implementation detail not present in Codex.

**What Claude/Gemini found that Codex missed:**
- Tab spacing: one every 50-75mm along board edges
- 3-5 tabs per side for standard boards; +1 tab per 50mm for boards >200mm

**What Codex found that Claude/Gemini missed:**
- Recommended placement includes even distribution (roughly every 50-75 mm), avoidance of corners/stress zones, and sufficient edge clearance.
- Mouse-bite hole geometry (diameter/spacing/count) is treated as a controllable variable affecting breakaway quality and crack risk.

**Ratings (1-5):**
- Claude: thoroughness 5/5, accuracy 4/5
- Gemini: thoroughness 4/5, accuracy 4/5
- Codex: thoroughness 3/5, accuracy 4/5

## RES-023-altium-mouse-bites

**Where all three agree (high confidence):**
- All three agree on the core problem framing and practical relevance; differences are mainly in level of detail.

**Where agents disagree (flag for resolution):**
- Different emphasis: Claude/Gemini include extra implementation detail not present in Codex.

**What Claude/Gemini found that Codex missed:**
- Mouse bites: holes tangent to PCB edge, large diameter, hole spacing < hole diameter
- V-scores: grooves on both sides, requires large clearance from copper

**What Codex found that Claude/Gemini missed:**
- Mouse-bite quality depends on hole geometry and tangent placement relative to board edges plus adequate routed clearance.
- V-score design requires explicit clearance/keepout management and documented scoring parameters.

**Ratings (1-5):**
- Claude: thoroughness 5/5, accuracy 4/5
- Gemini: thoroughness 4/5, accuracy 4/5
- Codex: thoroughness 3/5, accuracy 4/5

## RES-024-fabricator-common-line

**Where all three agree (high confidence):**
- All three agree on the core problem framing and practical relevance; differences are mainly in level of detail.

**Where agents disagree (flag for resolution):**
- Different emphasis: Claude/Gemini include extra implementation detail not present in Codex.

**What Claude/Gemini found that Codex missed:**
- Microtabs on common lines add process stability for sheet metal cutting
- Never let kerf from one piece overlap kerf from adjacent piece

**What Codex found that Claude/Gemini missed:**
- Heat, kerf sequencing, and tolerance requirements can make common-line strategies unsuitable in some geometries.

**Ratings (1-5):**
- Claude: thoroughness 5/5, accuracy 4/5
- Gemini: thoroughness 4/5, accuracy 4/5
- Codex: thoroughness 3/5, accuracy 4/5

## RES-025-mate-tabbing-practices

**Where all three agree (high confidence):**
- Sizing rule: smaller/harder parts + lower forces → thinner tabs; larger/softer parts + higher forces → thicker tabs

**Where agents disagree (flag for resolution):**
- Type label mismatch: Claude `Commercial`, Gemini `Commercial`, Codex `Product`.
- Different emphasis: Claude/Gemini include extra implementation detail not present in Codex.

**What Claude/Gemini found that Codex missed:**
- Tab location: place on face/edge requiring no machining (non-critical surface)
- Tab geometry: balance rigidity (holding force) vs. easy removal

**What Codex found that Claude/Gemini missed:**
- Effective tabbing depends on location, CAM capability, workholding setup, and geometry tuned to expected cutting forces.

**Ratings (1-5):**
- Claude: thoroughness 5/5, accuracy 4/5
- Gemini: thoroughness 4/5, accuracy 4/5
- Codex: thoroughness 3/5, accuracy 4/5

## RES-026-sendcutsend-tabs

**Where all three agree (high confidence):**
- Fully automatic tab placement based on part geometry — no custom placement available
- All tabs and lead-ins are outside part geometry (design integrity preserved)

**Where agents disagree (flag for resolution):**
- Type label mismatch: Claude `Commercial`, Gemini `Commercial`, Codex `Product`.
- Different emphasis: Claude/Gemini include extra implementation detail not present in Codex.

**What Claude/Gemini found that Codex missed:**
- Tabs designed to be as small as possible for easy removal
- Production service processes many parts with automatic placement — validates algorithmic feasibility

**What Codex found that Claude/Gemini missed:**
- No clear unique finding; Codex is mostly a concise restatement.

**Ratings (1-5):**
- Claude: thoroughness 5/5, accuracy 4/5
- Gemini: thoroughness 4/5, accuracy 4/5
- Codex: thoroughness 3/5, accuracy 5/5

## RES-027-mastercam-tabs

**Where all three agree (high confidence):**
- Two automatic modes: by count (N tabs) or by max distance
- The "Number of Tabs" vs "Max Distance" dichotomy is the established industry standard for automatic placement algorithms.

**Where agents disagree (flag for resolution):**
- Different emphasis: Claude/Gemini include extra implementation detail not present in Codex.

**What Claude/Gemini found that Codex missed:**
- Manual placement via Point Position for exact locations
- Integrated into 2D Contour, Window Machining, and 5-axis parting

**What Codex found that Claude/Gemini missed:**
- No clear unique finding; Codex is mostly a concise restatement.

**Ratings (1-5):**
- Claude: thoroughness 5/5, accuracy 4/5
- Gemini: thoroughness 4/5, accuracy 4/5
- Codex: thoroughness 3/5, accuracy 5/5

## RES-028-maslow-tabs-strategy

**Where all three agree (high confidence):**
- Tab size should scale with material thickness
- Suggested rule of thumb: tab dimensions should scale with material thickness, with example values for half-inch stock.

**Where agents disagree (flag for resolution):**
- Different emphasis: Claude/Gemini include extra implementation detail not present in Codex.

**What Claude/Gemini found that Codex missed:**
- Tab thickness: ~50% of material thickness (0.255" for 0.500" material)
- Tab width = 1.5× tab depth

**What Codex found that Claude/Gemini missed:**
- No clear unique finding; Codex is mostly a concise restatement.

**Ratings (1-5):**
- Claude: thoroughness 5/5, accuracy 5/5
- Gemini: thoroughness 4/5, accuracy 4/5
- Codex: thoroughness 3/5, accuracy 5/5

# Summary

**Strongest consensus findings:**
- Across sources, all three agents converge on the same operational pattern: automatic tab placement is useful but still needs manual overrides for edge cases, especially in dense nests.
- Sequencing, shared edges/common-line behavior, and hold-down method (tabs vs onion-skinning/vacuum) are repeatedly linked as a coupled optimization problem.

**Biggest disagreements:**
- Most disagreements are about specificity and interpretation depth, not direct factual contradiction.
- Type-label inconsistency (`Commercial` vs `Product`) appears in multiple vendor-document resources.

**Resources where reports diverge most:**
- RES-005-mecsoft-bridges-tabs
- RES-001-micro-joints-genetic-algorithm
- RES-006-fusion360-tabs-reference
- RES-007-fusion360-tabs-tutorial
- RES-009-vcarve-tab-customization
- RES-012-freecad-dressuptag-source
- RES-021-cnczone-optimize-tabs
