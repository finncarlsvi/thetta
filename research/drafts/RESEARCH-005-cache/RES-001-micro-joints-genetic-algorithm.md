# RES-001: Effective location of micro joints and generation of tool path using heuristic and genetic approach
**URL**: https://link.springer.com/article/10.1007/s12289-019-01488-1
**Type**: Paper
**Status**: PARTIAL (paywall — content from search result summaries)
**Authors**: Vijay Anand Kandasamy, S. Udhayakumar
**Published**: International Journal of Material Forming, Vol. 13, pp. 317–329 (2020)

## Abstract (from search results)
Proposes a heuristic and genetic approach for cutting sheet metal parts that identifies effective locations for micro-joints, thereby generating an optimized tool path.

## Methodology
1. **Micro-joint location using centroid**: Effectively locates three micro-joints in a specific orientation by considering the centroid point of the part geometry. This eliminates excessive micro-joints.
2. **Edge segmentation**: Considering the contour of the parts as segments, each one is cut by the tool with a piercing (lead in/entry) point just away from the micro-joint.
3. **Genetic algorithm**: The first component of the genetic string contains the angle for each part (0° to 120°) by which lines are projected from the centroid to the part contour. The second component provides the order of piercing and cutting segments.

## Key Findings
- The approach effectively locates micro-joints and generates near-optimal tool paths that reduce total air travel distance.
- The number of micro-joints is directly proportional to piercing points, which highly influences tool travel path and process time.
- In industrial practice (Laser, Plasma, AWJ), micro-joints are randomly placed by shop floor engineers — this paper automates the process.
- Ineffective location or negligence of micro-joints causes: falling of parts, detaching of parts, disruption to cutting tool.
- The GA suits well for generating near optimal tool paths within considerable iterations.

## Algorithm Details
- Three micro-joints per part (heuristic choice)
- Placement determined by projecting lines from part centroid at varying angles
- GA optimizes both the angle of projection AND the cutting sequence
- Objective function: minimize total air travel distance of tool
