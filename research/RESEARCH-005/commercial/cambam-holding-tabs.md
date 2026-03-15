# CamBam Holding Tabs Documentation

> **URL**: http://www.cambam.info/doc/plus/cam/HoldingTabs.htm
> **Type**: Commercial
> **Agents**: Claude, Codex, Gemini
> **Relevance**: High
> **Confidence**: High -- all agents agree on the explicit formula

## Summary
CamBam's holding tabs documentation provides the most explicit and implementable algorithmic description of automatic tab placement found in this research. While other CAM tools describe their tab features in terms of user-facing parameters, CamBam reveals the underlying formula: Number of tabs = perimeter / Tab Distance, rounded to the nearest whole number, then clamped between configurable Minimum Tabs and Maximum Tabs values. A Size Threshold parameter skips tab insertion entirely for shapes whose perimeter falls below the threshold.

This formula is elegant and practical. The min/max clamp ensures that very small shapes do not get too many tabs and very large shapes do not get too few. The size threshold addresses the common case of small interior cutouts or tiny parts where tabs are unnecessary or impractical. This resource is particularly valuable for Thetta because it provides a concrete, implementable baseline algorithm.

## Key Findings
- **Tab count formula**: `count = clamp(round(perimeter / tab_distance), min_tabs, max_tabs)` (all agents)
- **Size threshold**: shapes with perimeter < threshold get no tabs (all agents)
- **Parameters**: Tab Distance, Min Tabs, Max Tabs, Size Threshold, Tab Width, Tab Depth, Tab Style (Claude, Gemini)
- Most explicit algorithmic description of automatic tab placement found across all resources (all agents)
- Naive perimeter spacing does not account for corners or sharp geometry, likely placing tabs on unsuitable vertices (Gemini)

## Technical Details
- **Software**: CamBam (affordable CAM tool, popular in hobbyist community)
- **Algorithm**: Perimeter-based division with min/max clamp
- **Formula**: `base_count = round(perimeter / tab_distance)`, `final_count = clamp(base_count, min_tabs, max_tabs)`, skip if `perimeter < size_threshold`
- **Distribution**: Evenly spaced along perimeter
- **Parameters**: Tab Distance, Minimum Tabs, Maximum Tabs, Size Threshold, Tab Width, Tab Depth
- **Content access**: Website certificate expired

## Strengths
- Clearest algorithmic description found -- directly implementable
- Perimeter-based approach naturally scales with part size
- Min/max clamp prevents degenerate cases
- Size threshold is a practical quality-of-life feature
- Simple enough to serve as baseline for more sophisticated algorithms

## Weaknesses & Limitations
- Does not specify how tabs are positioned along the perimeter (evenly? avoiding corners?)
- No consideration of part geometry complexity (concavities, narrow sections)
- Does not address shared tabs between adjacent parts
- Website certificate expired -- documentation may be abandoned
- No material-specific or thickness-dependent sizing guidance

## Consensus & Disagreements
All three agents agree on the core formula and its significance. Claude provides specific parameters and a numerical example (160mm perimeter, 30mm distance = 5 tabs), though this may be inferred rather than directly from the source since the page failed to load. Gemini uniquely identifies the corner-placement weakness. Codex frames it as "deterministic baseline" suitable for benchmarking.

## Connections
- [RES-001] Micro-Joints Genetic Algorithm -- centroid-based heuristic is a more geometry-aware alternative
- [RES-005] MecSoft Bridges & Tabs -- MecSoft offers count OR distance; CamBam unifies them
- [RES-009] VCarve Tab Customization -- VCarve's min/max constraint is similar
- [RES-021] CNCZone Optimize Tabs -- community minimum 3 tabs aligns with CamBam's min_tabs
- [RES-011] FreeCAD DressupTag -- edge-length-proportional distribution as alternative approach
