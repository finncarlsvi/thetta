# CamBam Holding Tabs Documentation
> **URL**: http://www.cambam.info/doc/plus/cam/HoldingTabs.htm
> **Type**: Commercial
> **Agent**: Gemini
> **Relevance**: High
> **Credibility**: Medium

## Summary
CamBam is an established, affordable CAM package known for its straightforward, no-nonsense approach to toolpath generation. Its documentation on Holding Tabs is particularly valuable because it explicitly states the algorithm used for automatic tab placement, unlike the opaque descriptions found in many high-end packages.

The documentation reveals a simple but effective formula: the number of tabs is calculated by dividing the total perimeter of the shape by the user-defined "Tab Distance," rounded to the nearest whole number. This calculated count is then strictly clamped between user-defined "Minimum Tabs" and "Maximum Tabs" limits.

Additionally, CamBam implements a "Size Threshold." If a shape's perimeter is smaller than this threshold, the algorithm automatically skips tab insertion entirely. This is a critical heuristic for avoiding tabs on tiny internal cutouts (like drill holes or small pockets) where tabs are impossible or unnecessary.

## Key Findings
- A standard algorithm for tab count is: `Count = round(Perimeter / Tab_Distance)`.
- Clamping the calculated count with Minimum and Maximum bounds prevents over-tabbing on huge parts and under-tabbing on small ones.
- A "Size Threshold" heuristic is essential to prevent the algorithm from attempting to tab tiny features like small holes or internal cutouts.

## Technical Details
The automatic generation logic is highly explicit:
1. `base_count = round(perimeter / tab_distance)`
2. `final_count = clamp(base_count, min_tabs, max_tabs)`
3. If `perimeter < size_threshold`, then `final_count = 0`
The software also supports configurable Tab Width, Tab Depth, and stylistic options.

## Strengths
- Provides one of the only explicit algorithmic descriptions of automatic tab placement found in commercial documentation.
- The inclusion of a size threshold is a practical, highly implementable heuristic.

## Weaknesses & Limitations
- The spacing logic (dividing the perimeter) is naive; it doesn't account for corners or sharp geometry, likely placing tabs on unsuitable vertices.
- The documentation site suffers from expired certificates, indicating older software.

## Connections
- Contrasts with the centroid-projection approach in RES-001 by using a purely perimeter-based calculation.
- The perimeter threshold aligns with the small-parts discussion in RES-020.
