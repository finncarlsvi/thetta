# RESEARCH-005 Synthesis Notes

> **Date**: 2026-03-15
> **Author**: Synthesis agent (Claude Opus 4.6)
> **Scope**: Conflict resolution, interview refinements, remaining gaps

---

## 1. Resolved Conflicts

### 1.1 Stay-Down Dependency on Tabs (RES-018)

**Conflict**: Gemini claims HOMAG's Stay-Down "relies heavily on tabs" (causal dependency). Claude describes them as complementary.

**Resolution**: Both are partially correct. Stay-Down keeps the tool at cutting depth between parts, which means the tool cannot retract to avoid separated parts. If a part is fully separated and shifts, it would collide with the tool. Therefore, Stay-Down *requires* some retention mechanism (tabs, vacuum, or onion skin). Tabs are one option, not the only one. In HOMAG's implementation, all four strategies work together -- Stay-Down requires retention, and tabs are one available retention method. For Thetta's purposes, this distinction is not critical since the immediate goal is per-part tabbing independent of toolpath strategy.

### 1.2 Two-Pass vs. Three-Pass Tab Strategy (RES-028)

**Conflict**: Gemini describes two toolpaths; Claude describes three passes.

**Resolution**: The difference is framing, not substance. Claude breaks the process into three conceptual passes: (1) rough, (2) through-cut except tabs, (3) tab removal. Gemini combines (1) and (2) into a single "heavy" toolpath and treats (3) as the second. The actual machining strategy is the same. For implementation, this is a toolpath generation concern, not a tab placement concern.

### 1.3 Circular-Path Tab Failure (RES-015)

**Conflict**: Three different speculations on the cause. Claude: uniform curvature lacks natural segmentation points. Gemini: arc discretization or normal vector calculation. Codex: contour discretization and offset geometry interaction.

**Resolution**: All three are plausible contributing factors. The root cause is likely that most tab algorithms assume a polygon (sequence of line segments) and place tabs at or between vertices. Circular arcs have no natural vertices, so the algorithm either fails to find placement points or places them incorrectly after discretization introduces numerical artifacts. For Thetta, this is solvable by sampling the contour at regular parameter intervals rather than relying on vertex-based segmentation.

### 1.4 CamBam Numerical Example (RES-010)

**Conflict**: Claude provides a specific example (160mm perimeter, 30mm distance = 5 tabs) that may be inferred since the CamBam page failed to load.

**Resolution**: The formula itself (`count = round(perimeter / tab_distance)`) is confirmed by all three agents and matches what CamBam's search-result summaries describe. The numerical example is mathematically correct given the formula (160/30 = 5.33, rounded to 5). Whether it appears literally on the CamBam page is unverifiable but immaterial -- the formula is the important finding.

### 1.5 Credibility Rating Framework

**Conflict**: Codex systematically rates community sources Low, Claude rates Medium, Gemini rates High.

**Resolution**: For the synthesis, community forums with specific technical detail and practitioner consensus (CNCZone, WOODWEB) are rated Medium. Pure anecdotal posts without technical reasoning are rated Low. Official vendor documentation (even from community blogs) is rated High when content is verifiable. This framework was applied consistently across the synthesis.

---

## 2. Interview Refinements

The interview established three key constraints that shaped the synthesis:

### 2.1 Per-Part First, Shared Tabs Later

The proposed algorithm (Section 4 of RESEARCH-005.md) prioritizes per-part tab placement as a self-contained module. Shared-tab coordination is outlined as a future extension (Section 4.6) but not integrated into the core algorithm. This matches the interview requirement to ship per-part auto-tab first.

### 2.2 Post-Processing Step

The algorithm is designed as a pure function: `contour + material params -> tab positions`. It runs after nesting is complete and is independent of the nesting algorithm. This avoids the HOMAG-style deep integration, which would require modifying jagua-rs. The FreeCAD "dressup" pattern (apply tabs to existing paths) is the correct architectural model, but Thetta should apply tabs to native geometry (not G-code), avoiding FreeCAD's fragility.

### 2.3 CNC Router for Wood/Plastic

Thermal cutting concerns (laser/plasma heat expansion, kerf overlap from RES-024) do not apply. This simplifies the algorithm significantly -- no need for heat-aware sequencing or kerf overlap checking. The tab sizing heuristics from wood community forums (RES-020, RES-021, RES-028) are directly applicable.

---

## 3. Remaining Gaps

### 3.1 Critical for MVP

1. **Tab shape selection**: The algorithm proposes positions and dimensions but does not specify tab shape (rectangular, triangular, ramped). Recommendation: start with rectangular (simplest G-code generation), add triangular as a user option. 3D/ramped tabs can follow later.

2. **Concavity handling**: The edge-scoring system assigns low scores to concave edges, but highly concave parts (C-shaped, L-shaped) may still get suboptimal placement. Needs testing with real part geometries from Thetta's test suite.

3. **Interior cutout tabs**: Some parts have interior holes that need separate tabbing (the hole's island must be held). The size_threshold parameter handles skipping tiny holes, but medium-sized interior cutouts need their own tab set. The algorithm should run independently per contour (outer + each inner).

### 3.2 Important for Quality

4. **Validation against real jobs**: The proposed algorithm needs to be tested against a set of real CNC routing jobs with known good tab placements. No such benchmark dataset was found in this research.

5. **Material-specific sizing calibration**: The 30-50% material thickness rule needs validation for specific materials Thetta targets (plywood, MDF, acrylic, HDPE). Different materials have very different shear strengths at tab cross-sections.

6. **UX for manual override**: All commercial tools offer manual override (add/delete/move tabs after automatic placement). Thetta needs this too. The VCarve drag-to-edit pattern is the gold standard.

### 3.3 Future Research (Post-MVP)

7. **Shared-tab algorithm**: No existing algorithm to reference. Thetta will need to design this from first principles using the adjacency graph approach outlined in Section 4.6.

8. **Force-based optimization**: A physics-aware tab placement system that models lateral cutter force, vacuum suction (if applicable), part weight, and material properties to determine optimal tab count and placement. This is a research contribution rather than an implementation task.

9. **Tab removal prediction**: Estimate post-processing time based on tab count, size, and placement (ease of access for sanding/filing). Could be used as an optimization objective alongside hold-down strength.

---

## 4. Agent Performance Summary

| Agent | Strengths | Weaknesses | Best Use |
|-------|-----------|------------|----------|
| Claude | Most thorough, best citations, strongest cross-references | Occasionally verbose; may infer details from incomplete sources | Primary source for detailed technical analysis |
| Codex | Concise, unique algorithmic framing (scoring terms, constraint encoding) | Systematically least detailed; minimal cross-references | Architecture and optimization design patterns |
| Gemini | Best physical/mechanical insights (forces, G-code, failure modes) | Occasional strong assertions not fully supported | Physical validation of heuristics, UX insights |

The three-agent approach worked well. Each agent contributed unique perspectives that the others missed. Claude provided the foundation, Gemini added physical reasoning, and Codex contributed architectural framing. Cross-reference reviews were essential for identifying potential fabrications (CamBam example) and disagreements (HOMAG Stay-Down).
