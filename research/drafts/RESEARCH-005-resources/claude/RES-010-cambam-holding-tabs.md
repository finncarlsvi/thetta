# CamBam Holding Tabs Documentation
> **URL**: http://www.cambam.info/doc/plus/cam/HoldingTabs.htm
> **Type**: Commercial
> **Agent**: Claude
> **Relevance**: High
> **Credibility**: High

## Summary
CamBam's holding tabs documentation provides the most explicit and implementable algorithmic description of automatic tab placement found in this research. While other CAM tools describe their tab features in terms of user-facing parameters, CamBam's documentation reveals the underlying formula: Number of tabs = perimeter ÷ Tab Distance, rounded to the nearest whole number, then clamped between configurable Minimum Tabs and Maximum Tabs values. A Size Threshold parameter skips tab insertion entirely for shapes whose perimeter falls below the threshold.

This formula is elegant and practical. For example, a shape with a 160mm perimeter and a 30mm tab distance would get 160/30 ≈ 5 tabs. The min/max clamp ensures that very small shapes don't get too many tabs (which would be impractical to remove) and very large shapes don't get too few (which would allow movement). The size threshold addresses the common case of small interior cutouts or tiny parts where tabs are unnecessary or impractical.

CamBam also offers configurable Tab Width, Tab Depth, and Tab Style parameters, though the documentation doesn't detail the available styles. This resource is particularly valuable for Thetta because it provides a concrete, implementable baseline algorithm that can be extended with more sophisticated heuristics (corner avoidance, shared-tab alignment, material-aware sizing).

## Key Findings
- **Tab count formula**: tabs = round(perimeter ÷ tab_distance), clamped to [min_tabs, max_tabs]
- **Size threshold**: shapes with perimeter < threshold get no tabs
- **Example**: 160mm perimeter, 30mm distance = 5 tabs
- **Parameters**: Tab Distance, Min Tabs, Max Tabs, Size Threshold, Tab Width, Tab Depth, Tab Style
- Most explicit algorithmic description of automatic tab placement found across all resources

## Technical Details
- **Software**: CamBam (affordable CAM tool, popular in hobbyist community)
- **Algorithm**: Perimeter-based division with min/max clamp
- **Formula**: count = clamp(round(perimeter / tab_distance), min_tabs, max_tabs)
- **Distribution**: Evenly spaced along perimeter (implied by division formula)
- **Parameters**: Tab Distance, Minimum Tabs, Maximum Tabs, Size Threshold, Tab Width, Tab Depth
- **Content access**: FETCH_FAILED (expired certificate) — content from search results

## Strengths
- Clearest algorithmic description found — directly implementable
- Perimeter-based approach naturally scales with part size
- Min/max clamp prevents degenerate cases
- Size threshold is a practical quality-of-life feature
- Simple enough to serve as baseline for more sophisticated algorithms

## Weaknesses & Limitations
- Doesn't specify how tabs are positioned along the perimeter (evenly? avoiding corners?)
- No consideration of part geometry complexity (concavities, narrow sections)
- Doesn't address shared tabs between adjacent parts
- Website certificate expired — documentation may be abandoned
- No material-specific or thickness-dependent sizing guidance

## Connections
- [RES-001 Micro-Joints Genetic Algorithm] — centroid-based heuristic is a more geometry-aware alternative to perimeter division
- [RES-005 MecSoft Bridges & Tabs] — MecSoft offers count OR distance; CamBam unifies them with the division formula
- [RES-009 VCarve Tab Customization] — VCarve's min/max constraint is similar but adds interactive editing
- [RES-021 CNCZone Optimize Tabs] — community heuristic of "minimum 3 tabs" aligns with CamBam's min_tabs parameter
- [RES-011 FreeCAD DressupTag] — FreeCAD's edge-length-proportional distribution is a related but different approach
