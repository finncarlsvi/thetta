# RESEARCH-005: Cross-Reference Review

This document synthesizes the findings of Claude, Codex, and Gemini across 28 research resources concerning CNC workholding, tab placement algorithms, and nesting strategies.

## Individual Resource Analysis

### RES-001: Micro-Joints Genetic Algorithm
- **Consensus**: High confidence that the centroid-projection heuristic (3 tabs at 120° intervals) coupled with a Genetic Algorithm effectively optimizes tab placement and piercing sequences.
- **Disagreements**: None.
- **Missed by Gemini**: Claude explicitly detailed the 120-degree projection angle mechanism.
- **Found by Gemini**: Identified the weakness that exactly 3 micro-joints might not scale safely to very large or highly complex concave parts.
- **Ratings**: Claude (5/5), Codex (4/5), Gemini (4.5/5)

### RES-002: Common-Edge Nested Cutting
- **Consensus**: Common-edge nesting is critical for material efficiency but severely complicates path optimization due to shared boundaries.
- **Disagreements**: Differences in interpreting paywalled content; Gemini focused more on physical routing translation, while Codex focused on structural planning.
- **Missed by Gemini**: Claude made explicit connections to Deepnest and HOMAG.
- **Found by Gemini**: Stated the exact geometric foundation connecting common-edge optimization to the shared-tab problem.
- **Ratings**: Claude (4/5), Codex (3/5), Gemini (4/5)

### RES-003: Cutting Path Two-Step GA
- **Consensus**: A two-step GA (Generalized TSP) effectively separates piercing point optimization from part sequencing.
- **Disagreements**: None.
- **Missed by Gemini**: Claude detailed the specific "city cluster" GTSP formulation.
- **Found by Gemini**: Explicitly mapped piercing points to tab endpoints, providing a direct connection to hold-down strategies.
- **Ratings**: Claude (4/5), Codex (3/5), Gemini (4/5)

### RES-004: Tool Path Optimization Models
- **Consensus**: CNC tool paths can be rigorously modeled as GTSP variants with thermal/mechanical constraints.
- **Disagreements**: Agents diverge on applicability; Claude and Gemini note thermal constraints don't strictly apply to cold routing, whereas Codex treats them as generic constraints.
- **Missed by Gemini**: Claude highlighted the taxonomic/classification value of the paper.
- **Found by Gemini**: Emphasized mapping physical constraints directly to mathematical boundaries in the model.
- **Ratings**: Claude (4/5), Codex (3/5), Gemini (4/5)

### RES-005: MecSoft Bridges & Tabs
- **Consensus**: Commercial standard involves Triangular vs Rectangular shapes, and placement via either fixed count or fixed distance, with manual override.
- **Disagreements**: None.
- **Missed by Gemini**: Claude provided the exact UI pathway (Advanced Cut Parameters tab).
- **Found by Gemini**: Highlighted the "Count vs Distance" dichotomy as the dominant industry heuristic.
- **Ratings**: Claude (5/5), Codex (4/5), Gemini (4/5)

### RES-006: Fusion 360 Tabs Reference
- **Consensus**: Fusion 360 mirrors MecSoft's feature set (By distance / At points, Rectangular/Triangular).
- **Disagreements**: None.
- **Missed by Gemini**: Claude extracted exact dimensional recommendations (0.1125" height) and the compensation bug workaround.
- **Found by Gemini**: Highlighted the vendor's own recommendation for visual inspection, implicitly admitting algorithmic naivety.
- **Ratings**: Claude (5/5), Codex (4/5), Gemini (4/5)

### RES-007: Fusion 360 Tabs Tutorial
- **Consensus**: Practical confirmation of how tabs work physically (tool retracts/plunges).
- **Disagreements**: None.
- **Missed by Gemini**: Claude included post-machining removal context.
- **Found by Gemini**: Broke down the exact G-code movement required for rectangular vs triangular tabs.
- **Ratings**: Claude (4/5), Codex (3/5), Gemini (4/5)

### RES-008: Fusion 360 Adjacent Tabs Forum
- **Consensus**: Current CAM auto-tab features destroy tabs on adjacent nested parts due to a lack of shared-edge awareness.
- **Disagreements**: None.
- **Missed by Gemini**: Claude detailed the specific community workarounds (e.g., 1/4" gap minimum).
- **Found by Gemini**: Diagnosed the root cause as per-contour path generation lacking global spatial awareness.
- **Ratings**: Claude (5/5), Codex (4/5), Gemini (5/5)

### RES-009: VCarve Tab Customization
- **Consensus**: VCarve provides excellent UX with drag-to-edit auto-tabs and introduces "3D tabs" (ramped cuts) for smoother motion.
- **Disagreements**: None.
- **Missed by Gemini**: Claude noted the min/max tab count clamp feature.
- **Found by Gemini**: Detailed how 3D tabs alter G-code generation to coordinated multi-axis moves, preventing tool marks.
- **Ratings**: Claude (5/5), Codex (4/5), Gemini (4/5)

### RES-010: CamBam Holding Tabs
- **Consensus**: The standard algorithm is `count = perimeter / distance`, bounded by min/max limits, with a small-part size threshold skip.
- **Disagreements**: None.
- **Missed by Gemini**: Claude wrote out the exact clamp formula explicitly.
- **Found by Gemini**: Pointed out that this naive perimeter division often places tabs on unsuitable vertices (corners).
- **Ratings**: Claude (5/5), Codex (4/5), Gemini (5/5)

### RES-011: FreeCAD DressupTag
- **Consensus**: FreeCAD applies tabs as a post-processing "dressup," places them on the longest edges first, and prefers convex sections.
- **Disagreements**: None.
- **Missed by Gemini**: Claude extracted the full parameter list (Angle, Height, Radius, Width, Segmentation Factor).
- **Found by Gemini**: Emphasized the physical heuristic: convex edges are much easier to hand-file later.
- **Ratings**: Claude (4/5), Codex (4/5), Gemini (4/5)

### RES-012: FreeCAD DressupTag Source
- **Consensus**: The source code uses edge-length proportional distribution but is highly fragile due to parsing imported G-code paths.
- **Disagreements**: None.
- **Missed by Gemini**: Claude identified the exact Python function names and dictionary structures.
- **Found by Gemini**: Concluded that applying tabs to G-code rather than native CAD geometry is an architectural pitfall.
- **Ratings**: Claude (5/5), Codex (4/5), Gemini (4/5)

### RES-013: bCNC Repo
- **Consensus**: bCNC is a Python sender that uses a simple last-pass tab implementation and relies heavily on island-detection toggles.
- **Disagreements**: None.
- **Missed by Gemini**: Claude highlighted the cross-platform nature and 3-tab default behavior clearly.
- **Found by Gemini**: Emphasized the conceptual approach of treating tabs algorithmically as un-machined islands.
- **Ratings**: Claude (4/5), Codex (3/5), Gemini (4/5)

### RES-014: bCNC Tabs Tutorial
- **Consensus**: Users must select the final pass block and explicitly uncheck island settings to generate the default 3 tabs.
- **Disagreements**: None.
- **Missed by Gemini**: Claude provided a literal step-by-step workflow.
- **Found by Gemini**: Noted that this limits tab thickness strictly to the depth of the final pass.
- **Ratings**: Claude (4/5), Codex (3/5), Gemini (4/5)

### RES-015: viaconstructor
- **Consensus**: A Python/C++ CAM tool with robust features but a documented bug where tab generation fails on circular paths.
- **Disagreements**: None.
- **Missed by Gemini**: Claude highlighted the exact language split (50/46) and input format support.
- **Found by Gemini**: Connected the circular bug to the universal difficulty of offsetting and discretizing arcs in computational geometry.
- **Ratings**: Claude (4/5), Codex (4/5), Gemini (5/5)

### RES-016: Kiri:Moto
- **Consensus**: Browser-based JavaScript CAM is fully capable of handling CNC tab placement and toolpath generation.
- **Disagreements**: None.
- **Missed by Gemini**: Claude mentioned local hosting options via Docker.
- **Found by Gemini**: Noted that multi-modal tools often share offset/perimeter logic between 3D slicing and CNC contouring.
- **Ratings**: Claude (4/5), Codex (3/5), Gemini (4/5)

### RES-017: Deepnest
- **Consensus**: Deepnest's "common-line merging" algorithm solves the exact geometric prerequisite needed for shared-tab placement.
- **Disagreements**: None.
- **Missed by Gemini**: Claude noted the specific SVGNest underpinnings.
- **Found by Gemini**: Highlighted the use of No Fit Polygons (NFP) and path approximation.
- **Ratings**: Claude (4/5), Codex (4/5), Gemini (4/5)

### RES-018: HOMAG Nesting Features
- **Consensus**: HOMAG provides the ultimate industrial solution: integrating common-line cutting with retaining bars (tabs) and onion skinning.
- **Disagreements**: None.
- **Missed by Gemini**: Claude identified the exact woodWOP versions required.
- **Found by Gemini**: Highlighted that tabs are strictly utilized for small parts that cannot generate sufficient vacuum hold-down.
- **Ratings**: Claude (5/5), Codex (4/5), Gemini (5/5)

### RES-019: WOODWEB Tabs vs Onion Skinning
- **Consensus**: Real-world validation of the shared-tab problem: misaligned tabs get "disconnected" by adjacent cuts.
- **Disagreements**: None.
- **Missed by Gemini**: Claude noted that some high-end CAM does align tabs automatically.
- **Found by Gemini**: Defined the failure state occurring when part gaps are <= 2 tool diameters.
- **Ratings**: Claude (4/5), Codex (4/5), Gemini (4/5)

### RES-020: WOODWEB Tabbing Small Parts
- **Consensus**: Thick tabs (1/4") are highly rigid but costly to clean; two-pass cutting (roughing + final skin/tab pass) is standard practice.
- **Disagreements**: None.
- **Missed by Gemini**: Claude extracted specific operational tips (e.g., covering spoilboard with cardboard).
- **Found by Gemini**: Emphasized the physics of balancing lateral cutter force against the shear strength of the tab.
- **Ratings**: Claude (5/5), Codex (4/5), Gemini (4/5)

### RES-021: CNCZone Optimize Tabs
- **Consensus**: Minimum of 3 tabs prevents part rotation; tabs should be placed on straight edges or outside convex curves, never inside corners.
- **Disagreements**: None.
- **Missed by Gemini**: Claude noted the common physical dimension of 0.75" x 0.125".
- **Found by Gemini**: Explained the mechanical "pivot-point" physics of why 2-tab setups fail.
- **Ratings**: Claude (4/5), Codex (3/5), Gemini (5/5)

### RES-022: PCB Tab Routing Masterclass
- **Consensus**: PCB "mouse bites" (perforated tabs) are placed via distance intervals (50-75mm) and strictly avoid high-stress corners.
- **Disagreements**: Applicability to CNC. Claude and Gemini see it as cross-domain inspiration; Codex views it as highly relevant structural logic.
- **Missed by Gemini**: Claude extracted the specific FR-4 tensile strength specs.
- **Found by Gemini**: Suggested adopting the perforation concept into CNC by drilling relief holes in thick wood/metal tabs.
- **Ratings**: Claude (5/5), Codex (4/5), Gemini (4/5)

### RES-023: Altium Mouse Bites
- **Consensus**: Tangential placement of perforations ensures clean breaks. V-scores are an alternative but limited to straight lines.
- **Disagreements**: None.
- **Missed by Gemini**: Claude noted specific design rules (spacing < hole diameter).
- **Found by Gemini**: Emphasized that tangential placement keeps broken edges flush with the final boundary.
- **Ratings**: Claude (4/5), Codex (3/5), Gemini (4/5)

### RES-024: Fabricator Common Line
- **Consensus**: Microtabs on common lines provide massive process stability, keeping parts in the skeleton until the end.
- **Disagreements**: None.
- **Missed by Gemini**: Claude highlighted the strict rule against kerf overlap.
- **Found by Gemini**: Noted that thermal expansion can propagate errors across an entire sheet if parts are rigidly tabbed together.
- **Ratings**: Claude (4/5), Codex (3/5), Gemini (4/5)

### RES-025: Mate Tabbing Practices
- **Consensus**: 5-axis tabbing principles apply to 2D: place tabs on non-critical faces, and scale thickness based on material hardness/cutting force.
- **Disagreements**: None.
- **Missed by Gemini**: Claude explicitly listed the four critical factors from the article.
- **Found by Gemini**: Emphasized that tabbing is fundamentally a workholding strategy, not just a toolpath manipulation.
- **Ratings**: Claude (4/5), Codex (3/5), Gemini (4/5)

### RES-026: SendCutSend Tabs
- **Consensus**: Automated service scales using geometry-based, auto-placed tabs located entirely outside the part boundary.
- **Disagreements**: None.
- **Missed by Gemini**: Claude emphasized this as proof of automation viability at scale.
- **Found by Gemini**: Highlighted that "outside" placement prevents accidental undercutting of critical dimensions.
- **Ratings**: Claude (4/5), Codex (4/5), Gemini (4/5)

### RES-027: Mastercam Tabs
- **Consensus**: Confirms industry standard UI: Count vs Distance, and the ability to customize side profile angles (ramps).
- **Disagreements**: None.
- **Missed by Gemini**: Claude noted integration into 5-axis parting and window machining.
- **Found by Gemini**: Emphasized how ramping/angled tabs bridge the gap to 3D continuous motion.
- **Ratings**: Claude (4/5), Codex (4/5), Gemini (4/5)

### RES-028: Maslow Tabs Strategy
- **Consensus**: Ratio-based heuristics work well: Tab thickness = 50% material thickness; Tab width = 1.5x depth. Multi-pass cuts are vital for low-rigidity setups.
- **Disagreements**: None.
- **Missed by Gemini**: Claude listed specific manual removal tools (oscillating saws, etc.).
- **Found by Gemini**: Highlighted that vertical CNC orientations increase reliance on tabs due to gravity.
- **Ratings**: Claude (4/5), Codex (3/5), Gemini (4/5)

---

## Final Summary

**Strongest Consensus Findings:**
1. **The Shared-Tab Problem is Critical:** Across academic, commercial, and community sources, the destruction of misaligned tabs on adjacent parts is recognized as a major failure mode in modern nesting. HOMAG's industrial solution and Deepnest's common-line algorithms prove that inter-part geometric awareness is the required solution.
2. **The Minimum 3-Tab Rule:** Supported by both genetic algorithm papers and machinist heuristics, parts require a minimum of 3 tabs to prevent pivoting and mechanical snapping.
3. **Placement Heuristics:** Tabs should be placed on straight lines or convex curves. Placing tabs inside concave corners is universally discouraged due to post-processing difficulty and stress concentration.
4. **Distance vs. Count:** The standard commercial approach to automatic placement gives users the choice of specifying a fixed number of tabs OR a maximum distance interval, supplemented by size thresholds and manual override.
5. **Multi-Pass Mitigation:** To protect tabs from breaking under heavy lateral cutting forces, a two-pass strategy (leaving an onion skin or thick tabs until a final, light finishing pass) is standard practice.

**Biggest Disagreements:**
The agents did not disagree on the facts, but diverged on the applicability of cross-domain resources. Claude and Gemini noted that thermal expansion (laser) and FR-4 material limits (PCB) do not directly apply to cold routing of wood/plastics, whereas Codex often treated these constraints as generalized architectural requirements.

**Where Agent Reports Diverge Most:**
- **Claude** excelled at extracting exact numerical data, UI pathways, and formulas (e.g., CamBam's clamp formula, exact FR-4 tensile strengths, specific software versions).
- **Codex** focused heavily on software architecture, generalizing the findings into high-level system requirements and optimization strategies (e.g., treating features as nodes in a constraints graph).
- **Gemini** uniquely focused on the physical mechanics (pivot points, lateral forces, Z-axis G-code moves) and identified the structural implications of geometric choices (e.g., offset bugs on arcs, geometry vs. pathing pipelines).
