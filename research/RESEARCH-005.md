# RESEARCH-005: CNC Tab/Bridge Placement Optimization

> **Research question**: How should holding tabs be automatically placed on CNC-routed parts to maximize hold-down effectiveness while minimizing post-processing effort?
> **Target domain**: CNC Router for wood/plastic
> **Priority**: Per-part tab placement first, shared tabs between adjacent parts later
> **Integration**: Post-processing step (runs after nesting, independent)
> **Date**: 2026-03-15
> **Resources analyzed**: 28 (4 papers, 6 code, 6 commercial, 12 community)
> **Agents**: Claude, Codex, Gemini (3 independent analyses + 3 cross-reference reviews)

---

## 1. Recommended Resources (Ranked)

Ranked by fit to interview requirements (CNC router wood/plastic, per-part auto-tab, post-process architecture) multiplied by confidence and relevance.

| Rank | Resource | Type | Relevance | Confidence | Why |
|------|----------|------|-----------|------------|-----|
| 1 | [CamBam Holding Tabs](RESEARCH-005/commercial/cambam-holding-tabs.md) | Commercial | High | High | Only explicit, implementable formula for automatic tab count |
| 2 | [Micro-Joints GA (RES-001)](RESEARCH-005/papers/micro-joints-genetic-algorithm.md) | Paper | High | High | Only formal academic treatment of automatic tab placement |
| 3 | [FreeCAD DressupTag Source (RES-012)](RESEARCH-005/code/freecad-dressuptag-source.md) | Code | High | High | Open-source implementation with honest pitfall assessment |
| 4 | [FreeCAD DressupTag (RES-011)](RESEARCH-005/code/freecad-dressuptag.md) | Code | High | High | Edge-length-weighted distribution, convex preference |
| 5 | [Fusion 360 Adjacent Tabs Forum (RES-008)](RESEARCH-005/community/fusion360-adjacent-tabs-forum.md) | Community | High | High | Defines the shared-tab problem Thetta must solve |
| 6 | [HOMAG Nesting (RES-018)](RESEARCH-005/commercial/homag-nesting.md) | Commercial | High | High | State-of-the-art integrated nesting + tabs |
| 7 | [MecSoft Bridges & Tabs (RES-005)](RESEARCH-005/commercial/mecsoft-bridges-tabs.md) | Commercial | High | High | Industry-standard parameter model |
| 8 | [Fusion 360 Tabs Reference (RES-006)](RESEARCH-005/commercial/fusion360-tabs.md) | Commercial | High | High | De facto standard for user expectations |
| 9 | [CNCZone Optimize Tabs (RES-021)](RESEARCH-005/community/cnczone-optimize-tabs.md) | Community | Medium | High | 3-tab minimum rule, straight-edge preference |
| 10 | [VCarve Tab Customization (RES-009)](RESEARCH-005/community/vcarve-tab-customization.md) | Community | Medium | High | Best-in-class UX pattern, 3D tabs |
| 11 | [Common-Edge Nested Cutting (RES-002)](RESEARCH-005/papers/common-edge-nested-cutting.md) | Paper | High | Medium | Theoretical foundation for shared-tab sequencing |
| 12 | [WOODWEB Tabs vs Onion Skinning (RES-019)](RESEARCH-005/community/woodweb-tabs-vs-onionskin.md) | Community | Medium | Medium | Practitioner confirmation of shared-tab problem |
| 13 | [Deepnest (RES-017)](RESEARCH-005/code/deepnest.md) | Code | Medium | High | Common-line detection as shared-tab prerequisite |
| 14 | [The Fabricator Common Line (RES-024)](RESEARCH-005/community/fabricator-common-line.md) | Community | Medium | High | Microtab + common-line combination |
| 15 | [WOODWEB Tabbing Small Parts (RES-020)](RESEARCH-005/community/woodweb-tabbing-small-parts.md) | Community | Medium | High | Tab sizing relative to material thickness |
| 16 | [PCB Tab Routing (RES-022)](RESEARCH-005/community/pcb-tab-routing.md) | Community | Medium | High | Most quantitative spacing heuristics (50-75mm) |
| 17 | [SendCutSend Tabs (RES-026)](RESEARCH-005/commercial/sendcutsend-tabs.md) | Commercial | Medium | High | Validates fully automatic placement at scale |
| 18 | [Mate Tabbing (RES-025)](RESEARCH-005/commercial/mate-tabbing.md) | Commercial | Medium | High | Material/force-dependent sizing principle |
| 19 | [Mastercam Tabs (RES-027)](RESEARCH-005/community/mastercam-tabs.md) | Community | Medium | High | Professional CAM standard, angle parameter |
| 20 | [Maslow Tabs Strategy (RES-028)](RESEARCH-005/community/maslow-tabs-strategy.md) | Community | Low | Medium | Material-proportional sizing ratios |
| 21 | [Two-Step GA (RES-003)](RESEARCH-005/papers/cutting-path-two-step-ga.md) | Paper | Medium | High | GTSP formulation, piercing-point connection |
| 22 | [Tool Path Models (RES-004)](RESEARCH-005/papers/tool-path-optimization-models.md) | Paper | Medium | High | Taxonomy of optimization models |
| 23 | [bCNC (RES-013)](RESEARCH-005/code/bcnc.md) | Code | Medium | High | Simple reference implementation, 3-tab default |
| 24 | [Fusion 360 Tutorial (RES-007)](RESEARCH-005/community/fusion360-tabs-tutorial.md) | Community | Medium | High | G-code-level tab implementation detail |
| 25 | [bCNC Tutorial (RES-014)](RESEARCH-005/community/bcnc-tabs-tutorial.md) | Community | Medium | High | UX anti-pattern (hidden dependencies) |
| 26 | [viaconstructor (RES-015)](RESEARCH-005/code/viaconstructor.md) | Code | Medium | Medium | Known circular-path failure case |
| 27 | [Kiri:Moto (RES-016)](RESEARCH-005/code/kirimoto.md) | Code | Medium | Medium | Browser-based CAM proves JS viability |
| 28 | [Altium Mouse Bites (RES-023)](RESEARCH-005/community/altium-mouse-bites.md) | Community | Low | High | PCB-specific, low transfer to CNC routing |

---

## 2. Executive Summary

This research surveyed 28 resources across academic papers, open-source code, commercial CAM documentation, and community forums to understand the state of the art in automatic CNC tab/bridge placement.

**The field is surprisingly under-researched.** Only one academic paper (Kandasamy & Udhayakumar, 2020) directly addresses automatic tab placement, proposing a centroid-based heuristic with 3 tabs per part at 120-degree intervals. Commercial CAM tools universally offer tab features but their automatic placement algorithms are simplistic (distance-based spacing) and undocumented. Only one commercial tool (CamBam) explicitly reveals its formula: `count = clamp(round(perimeter / tab_distance), min_tabs, max_tabs)`.

**The shared-tab problem is universally recognized but unsolved in mainstream software.** Users of Fusion 360, VCarve, Mastercam, and other CAM tools consistently report that tabs placed on one part are destroyed when cutting an adjacent part. Only HOMAG's industrial intelliDivide Nesting integrates tab placement with nesting optimization, but it is proprietary and specific to their machinery ecosystem.

**Strong consensus exists on fundamental heuristics.** All sources agree on: minimum 3 tabs per part (to prevent rotation), placement on straight edges or convex curves (for easier post-processing), perimeter-proportional tab count, and the need for both automatic and manual placement modes. Tab sizing should scale with material thickness (approximately 30-50% of material thickness for height, with width at 1.5x height).

**Thetta has a clear opportunity.** No existing open-source or affordable CAM tool offers geometry-aware automatic tab placement with shared-tab coordination. The combination of CamBam's perimeter formula, FreeCAD's edge-length weighting, the academic centroid heuristic, and community-derived corner-avoidance rules provides a strong foundation for a superior algorithm.

---

## 3. Key Findings by Theme

### 3.1 Terminology Across Domains

The same concept is named differently across industries:
- **CNC Routing/Milling**: Tabs, holding tabs, bridges
- **Sheet Metal Cutting**: Micro-joints, microtabs
- **PCB Manufacturing**: Breakaway tabs, mouse bites, V-scores
- **Industrial Woodworking (HOMAG)**: Retaining bars
- **5-Axis Machining**: Tabs (workholding context)
- **FreeCAD**: Tags, holding tags, DressupTag
- **MecSoft**: Bridges

All refer to the same concept: intentionally uncut material left to connect a part to its surrounding stock during cutting, removed post-machining.

### 3.2 Algorithmic Approaches to Automatic Tab Placement

Three distinct approaches were identified:

1. **Perimeter-based division (CamBam)**: `count = clamp(round(perimeter / tab_distance), min_tabs, max_tabs)`. Simple, deterministic, directly implementable. Does not consider geometry quality (corners, concavities).

2. **Edge-length-proportional distribution (FreeCAD)**: Allocate tabs to edges proportional to their length, starting from the longest edge. More geometry-aware than perimeter division but still lacks corner avoidance.

3. **Centroid-based projection (Kandasamy & Udhayakumar)**: Project lines from part centroid to contour at 120-degree intervals. Elegant for convex parts but fails on highly concave geometry where projection lines may intersect the contour multiple times.

No existing implementation combines these approaches with corner avoidance, concavity handling, or shared-tab coordination.

### 3.3 Heuristics and Rules of Thumb

**Tab count:**
- Minimum 3 tabs per part (prevents rotation; 2 tabs create a pivot axis)
- 3-4 tabs sufficient for most typical parts
- Scale with perimeter: approximately one tab per 30-75mm of perimeter
- Size threshold: skip tabs for parts with perimeter below threshold

**Tab sizing (wood, 3/4 inch / 19mm sheet):**
- Height: 0.125 inch (3.2mm) for light hold; 0.250 inch (6.35mm) for strong lock-down
- Width: 0.75 inch (19mm) typical; 1.5x height as a ratio
- Height should be 30-50% of material thickness
- Fusion 360 recommended through-cut height: 0.1125 inch

**Tab placement preferences:**
- Straight edges preferred (easiest sanding/filing)
- Outside convex curves acceptable
- Inside corners: worst location (hardest post-processing, stress concentration)
- Avoid high-stress areas and narrow sections
- Prefer non-critical surfaces (where tab mark is least visible)

**Tab shapes:**
- Rectangular: strongest hold, hardest removal
- Triangular: easier removal, less material to sand
- 3D/Ramped (VCarve): smoothest motion, least tool marks, easiest removal
- Angle-parameterized (FreeCAD, Mastercam): continuous from rectangular to fully ramped

### 3.4 Commercial CAM Tool Comparison

| Tool | Auto Modes | Manual Mode | Shapes | Adjacent Tab Support | Notable Feature |
|------|-----------|-------------|--------|---------------------|-----------------|
| Fusion 360 | By distance | At points | Rect, Tri | No | Industry standard |
| MecSoft/RhinoCAM | By count, By distance | Bridge points | Rect, Tri | No | Dual auto modes |
| VCarve/Aspire | Min/max count + distance | Drag-to-edit | Rect, Tri, 3D | No | Interactive editing |
| Mastercam | By count, By max distance | Point Position | Angle-parameterized | No | Side profile angle |
| CamBam | Perimeter / distance | Manual | Configurable | No | Explicit formula |
| HOMAG intelliDivide | Integrated with nesting | Configurable | Retaining bars | Yes (integrated) | Only tool with adjacent support |
| SendCutSend | Fully automatic | None | Minimal | Unknown | Zero-touch production |

### 3.5 Open-Source Implementations

| Tool | Language | Algorithm | Quality | Key Lesson |
|------|----------|-----------|---------|------------|
| FreeCAD DressupTag | Python | Edge-length proportional | Fragile | Do not apply tabs to G-code; work from native geometry |
| bCNC | Python | Simple 3-tab insertion | Basic | Tabs as islands; avoid hidden checkbox dependencies |
| viaconstructor | Python/C++ | Unknown | Buggy | Circular paths break naive tab algorithms |
| Kiri:Moto | JavaScript | Undocumented | Unknown | Browser-based CAM is viable |
| Deepnest | JS/C++ | No tabs (common-line only) | Good | Common-line detection is the shared-tab prerequisite |

### 3.6 The Shared/Adjacent Tab Problem

The shared-tab problem occurs when:
1. Two parts are nested close together (gap <= 2 tool diameters)
2. Each part's tabs are computed independently
3. Cutting Part B's contour destroys Part A's tabs on the shared edge

All three agents independently confirmed this is unsolved in mainstream CAM software. HOMAG is the only known solution, and it is proprietary. The failure mode has real financial consequences: tool damage, ruined parts, lost production time.

**Solution requirements for Thetta:**
- Detect shared/adjacent edges (gap <= 2 tool diameters)
- Coordinate tab placement across neighbors: if Part A has a tab at position X on a shared edge, Part B must have a corresponding tab
- Model tabs as protected Z-height zones across all adjacent toolpaths
- Consider cutting sequence: which part is cut first affects which tabs survive

### 3.7 Tab vs. Onion Skin Trade-offs

| Factor | Tabs | Onion Skin |
|--------|------|------------|
| Hold strength | Strong (discrete anchors) | Moderate (continuous skin) |
| Post-processing | Sanding/filing at discrete points | Light sanding across bottom |
| Algorithm complexity | Must decide count + position | Simpler (just reduce depth) |
| Multi-pass compatibility | Yes | Yes (complementary) |
| Material waste | Minimal | Thin layer across entire cut |
| Best for | Standard routing, small parts | Large parts with good vacuum |

HOMAG combines both: use tabs for small parts with insufficient vacuum, onion skinning for additional stability, and multi-pass cutting to reduce forces during final separation.

---

## 4. Proposed Algorithm for Thetta

Based on all findings, the following algorithm is proposed for per-part automatic tab placement as a post-processing step after nesting.

### 4.1 Input

- `contour`: Part contour as a polygon (list of edges with type: line/arc)
- `material_thickness`: Material thickness in mm
- `material_type`: Enum (wood_soft, wood_hard, plastic, composite)

### 4.2 Output

- `tabs`: List of `{ position: Point, edge_index: usize, t: f64, width: f64, height: f64 }`

### 4.3 Parameters (User-configurable with defaults)

```
tab_distance: f64 = 50.0      // mm, target spacing between tabs
min_tabs: usize = 3            // absolute minimum (prevents rotation)
max_tabs: usize = 20           // absolute maximum
tab_width: f64 = 10.0          // mm, along cutting direction
tab_height: f64 = 0.4 * material_thickness  // mm, from cut bottom
size_threshold: f64 = 30.0     // mm, minimum perimeter for tabbing
corner_exclusion: f64 = 5.0    // mm, minimum distance from any corner/vertex
```

### 4.4 Core Algorithm

**Step 1: Perimeter-based count**
```
perimeter = sum(edge.length for edge in contour.edges)
if perimeter < size_threshold:
    return []  // no tabs for tiny parts

base_count = round(perimeter / tab_distance)
count = clamp(base_count, min_tabs, max_tabs)
```

**Step 2: Build candidate zones**

For each edge in the contour:
1. Compute edge length
2. Classify edge type: straight, convex_curve, concave_curve, corner_adjacent
3. Compute edge score:
   - Straight edges: score = 1.0
   - Convex curves (outside): score = 0.8
   - Concave curves (inside): score = 0.3
   - Edges shorter than `tab_width + 2 * corner_exclusion`: score = 0.0 (too short)
4. Exclude zones within `corner_exclusion` distance of any vertex

**Step 3: Edge-length-proportional distribution**

Allocate tabs to edges proportional to `edge.length * edge.score`, starting from the highest-scoring longest edge. This combines CamBam's perimeter approach with FreeCAD's edge-length weighting and community-derived corner avoidance.

```
total_weighted_length = sum(edge.length * edge.score for edge in contour.edges)
for edge in contour.edges sorted by (length * score) descending:
    edge_tabs = round(count * (edge.length * edge.score) / total_weighted_length)
    distribute edge_tabs evenly along edge, excluding corner_exclusion zones
```

**Step 4: Validate minimum spacing**

Ensure no two tabs are closer than `tab_width * 2`. If tabs overlap, remove the one on the lower-scored edge.

**Step 5: Validate minimum count**

If final count < `min_tabs`, force-place additional tabs on the best available edges, relaxing the corner exclusion if necessary.

### 4.5 Tab Sizing

```
tab_height = match material_type {
    wood_soft  => 0.40 * material_thickness,
    wood_hard  => 0.33 * material_thickness,
    plastic    => 0.33 * material_thickness,
    composite  => 0.50 * material_thickness,
}

tab_width = 1.5 * tab_height  // cross-section ratio from Maslow community
tab_width = clamp(tab_width, 5.0, 25.0)  // practical bounds in mm
```

### 4.6 Future Extension: Shared Tabs

For Phase 2 (shared tabs between adjacent parts):
1. After nesting, build adjacency graph: detect edge pairs where gap <= 2 * tool_diameter
2. For each shared edge pair, synchronize tab positions: if Part A has a tab at parameter `t` on the shared edge, Part B must have a matching tab at the corresponding position
3. Resolve conflicts: if tab positions cannot be aligned, increase gap or adjust positions
4. Model tabs as protected Z-height zones in the toolpath generator

---

## 5. Confidence Assessment Table

| Finding | Sources | Agent Agreement | Confidence |
|---------|---------|-----------------|------------|
| Minimum 3 tabs per part | RES-001, 013, 014, 021 | All 3 agents | Very High |
| Perimeter-proportional count formula | RES-010, 011, 012, 027 | All 3 agents | Very High |
| Adjacent/shared tabs unsolved in mainstream CAM | RES-008, 019 | All 3 agents | Very High |
| Tab shapes: Rect vs Tri as standard | RES-005, 006, 009, 027 | All 3 agents | Very High |
| Straight edge preference for placement | RES-011, 021, 022 | All 3 agents | Very High |
| Corner avoidance | RES-021, 022 | Gemini, Claude | High |
| HOMAG is only integrated solution | RES-018 | All 3 agents | High |
| Tab height ~33-50% of material thickness | RES-020, 028 | Claude, Gemini | High |
| Tab width = 1.5x height | RES-028 | Claude, Gemini | Medium |
| 50-75mm spacing heuristic (from PCB) | RES-022 | All 3 agents | Medium (cross-domain) |
| Two-tool-diameter threshold for shared-tab conflicts | RES-019 | Gemini only | Medium |
| Centroid projection works for convex parts | RES-001 | All 3 agents | High |
| Centroid projection fails for concave parts | RES-001 | Claude, Gemini | High |
| FreeCAD tab code is fragile/needs rewrite | RES-012 | All 3 agents | Very High |
| Circular paths break naive tab algorithms | RES-015 | All 3 agents | High |
| 3D/ramped tabs reduce tool marks | RES-009 | All 3 agents | High |

---

## 6. Index Table

| # | File | Type | Relevance | Confidence | One-line Summary |
|---|------|------|-----------|------------|------------------|
| RES-001 | [papers/micro-joints-genetic-algorithm.md](RESEARCH-005/papers/micro-joints-genetic-algorithm.md) | Paper | High | High | Centroid-based 3-tab heuristic + GA optimization |
| RES-002 | [papers/common-edge-nested-cutting.md](RESEARCH-005/papers/common-edge-nested-cutting.md) | Paper | High | Medium | Shared-edge cutting path optimization (paywalled) |
| RES-003 | [papers/cutting-path-two-step-ga.md](RESEARCH-005/papers/cutting-path-two-step-ga.md) | Paper | Medium | High | GTSP formulation linking piercing points to tabs |
| RES-004 | [papers/tool-path-optimization-models.md](RESEARCH-005/papers/tool-path-optimization-models.md) | Paper | Medium | High | Taxonomy of GTSP-based tool path models |
| RES-005 | [commercial/mecsoft-bridges-tabs.md](RESEARCH-005/commercial/mecsoft-bridges-tabs.md) | Commercial | High | High | Industry-standard Rect/Tri shapes, count/distance modes |
| RES-006 | [commercial/fusion360-tabs.md](RESEARCH-005/commercial/fusion360-tabs.md) | Commercial | High | High | De facto standard tab parameters and UX |
| RES-007 | [community/fusion360-tabs-tutorial.md](RESEARCH-005/community/fusion360-tabs-tutorial.md) | Community | Medium | High | G-code-level tab implementation mechanics |
| RES-008 | [community/fusion360-adjacent-tabs-forum.md](RESEARCH-005/community/fusion360-adjacent-tabs-forum.md) | Community | High | High | Shared-tab problem definition and workarounds |
| RES-009 | [community/vcarve-tab-customization.md](RESEARCH-005/community/vcarve-tab-customization.md) | Community | Medium | High | 3D/ramped tabs, best-in-class drag-to-edit UX |
| RES-010 | [commercial/cambam-holding-tabs.md](RESEARCH-005/commercial/cambam-holding-tabs.md) | Commercial | High | High | Explicit perimeter/distance formula with clamp |
| RES-011 | [code/freecad-dressuptag.md](RESEARCH-005/code/freecad-dressuptag.md) | Code | High | High | Edge-length-weighted dressup pattern, convex preference |
| RES-012 | [code/freecad-dressuptag-source.md](RESEARCH-005/code/freecad-dressuptag-source.md) | Code | High | High | Python source: edgeDict, findBottomWire, fragility |
| RES-013 | [code/bcnc.md](RESEARCH-005/code/bcnc.md) | Code | Medium | High | Simple 3-tab island-based implementation |
| RES-014 | [community/bcnc-tabs-tutorial.md](RESEARCH-005/community/bcnc-tabs-tutorial.md) | Community | Medium | High | Last-pass workflow, silent-failure UX anti-pattern |
| RES-015 | [code/viaconstructor.md](RESEARCH-005/code/viaconstructor.md) | Code | Medium | Medium | Python/C++ CAM with circular-path tab bug |
| RES-016 | [code/kirimoto.md](RESEARCH-005/code/kirimoto.md) | Code | Medium | Medium | Browser-based JS CAM with undocumented tabs |
| RES-017 | [code/deepnest.md](RESEARCH-005/code/deepnest.md) | Code | Medium | High | Common-line detection as shared-tab prerequisite |
| RES-018 | [commercial/homag-nesting.md](RESEARCH-005/commercial/homag-nesting.md) | Commercial | High | High | Only integrated nesting + tabs + common-line solution |
| RES-019 | [community/woodweb-tabs-vs-onionskin.md](RESEARCH-005/community/woodweb-tabs-vs-onionskin.md) | Community | Medium | Medium | Practitioner confirmation of shared-tab problem |
| RES-020 | [community/woodweb-tabbing-small-parts.md](RESEARCH-005/community/woodweb-tabbing-small-parts.md) | Community | Medium | High | Tab sizing for small parts, multi-pass strategy |
| RES-021 | [community/cnczone-optimize-tabs.md](RESEARCH-005/community/cnczone-optimize-tabs.md) | Community | Medium | High | Minimum-3-tabs rule with mechanical reasoning |
| RES-022 | [community/pcb-tab-routing.md](RESEARCH-005/community/pcb-tab-routing.md) | Community | Medium | High | Most quantitative spacing heuristics (50-75mm) |
| RES-023 | [community/altium-mouse-bites.md](RESEARCH-005/community/altium-mouse-bites.md) | Community | Low | High | PCB depanelization design rules |
| RES-024 | [community/fabricator-common-line.md](RESEARCH-005/community/fabricator-common-line.md) | Community | Medium | High | Microtab + common-line combination in metal fab |
| RES-025 | [commercial/mate-tabbing.md](RESEARCH-005/commercial/mate-tabbing.md) | Commercial | Medium | High | Material/force-dependent sizing, 4-factor framework |
| RES-026 | [commercial/sendcutsend-tabs.md](RESEARCH-005/commercial/sendcutsend-tabs.md) | Commercial | Medium | High | Fully automatic placement viable at production scale |
| RES-027 | [community/mastercam-tabs.md](RESEARCH-005/community/mastercam-tabs.md) | Community | Medium | High | Professional standard: count/distance + angle param |
| RES-028 | [community/maslow-tabs-strategy.md](RESEARCH-005/community/maslow-tabs-strategy.md) | Community | Low | Medium | Material-proportional sizing ratios (50%, 1.5x) |

---

## 7. Gaps and Limitations

### 7.1 Research Gaps

1. **No experimental validation of tab placement algorithms.** All sources are either theoretical (paywalled papers), anecdotal (forums), or implementation descriptions (code/docs). No published study compares different automatic placement strategies by measuring actual hold-down failure rates, post-processing time, or surface quality.

2. **Material-specific tab sizing is purely heuristic.** The 30-50% material thickness rule comes from community forums, not engineering analysis. No source provides analytically derived tab sizing based on material shear strength, cutting forces, and part weight.

3. **Shared-tab algorithms are entirely unexplored in open literature.** The only solution (HOMAG) is proprietary. No academic paper, open-source implementation, or documented algorithm addresses the problem of coordinating tabs between adjacent nested parts.

4. **Curved contour handling is poorly understood.** The viaconstructor circular-path bug and FreeCAD's fragility suggest that tab placement on arcs, splines, and complex curves is a non-trivial problem that existing tools handle poorly.

5. **No force-based placement optimization.** No source models the actual physical forces during cutting (lateral cutter force, vacuum suction, part weight, material stiffness) to determine optimal tab placement. All approaches are purely geometric.

### 7.2 Limitations of This Research

1. **Paywalled papers (RES-001 through RES-004):** All four academic papers were behind paywalls. Agents worked from abstracts and metadata. Full algorithm details are unavailable.

2. **Expired/inaccessible sources:** CamBam documentation (expired certificate), WOODWEB forum thread (expired message), CNCZone thread (403 error). Content reconstructed from search summaries.

3. **Agent speculation:** Where source material was limited, agents sometimes speculated. The Claude review flagged CamBam numerical examples that may be inferred rather than sourced. Cross-reference reviews helped identify and bound these cases.

4. **Domain transfer uncertainty:** PCB tab heuristics (50-75mm spacing) come from a domain with very different materials, scales, and forces. Transfer to CNC routing of wood/plastic requires validation.

5. **No user testing:** The proposed algorithm has not been validated against real CNC routing jobs. The heuristics are derived from literature synthesis, not empirical testing.
