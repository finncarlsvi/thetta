# Research 005 Brief: CNC Tab/Bridge Placement Optimization

## Research Question

What algorithms, heuristics, and tools exist for automatically determining optimal placement of holding tabs (bridges/micro-joints) in CNC profile cutting, and how can tabs between adjacent nested pieces be aligned to share a single tab?

## Search Dimensions

Ranked by priority:

1. **Code / OSS** — Open-source CAM libraries, nesting tools, or standalone tab placement implementations (GitHub repos, Rust/Python/C++ libraries). Look for algorithmic approaches embedded in CNC toolpath generators.
2. **Academic** — Papers on tab placement optimization, CNC workholding strategies, nesting with shared tabs, computational geometry approaches to bridge placement.
3. **Commercial** — How commercial CAM software (Fusion 360, VCarve, SheetCAM, Aspire, Mastercam, SolidCAM) handles automatic tab placement. Feature comparisons and documentation.
4. **Community** — CNC forum discussions, blog posts, YouTube tutorials, maker community knowledge on tab placement best practices and heuristics.

## Scope Boundaries

**In scope:**
- All CNC profile cutting methods: routing, milling, laser, plasma, waterjet
- All materials: wood, metal, plastics, composites
- Tab/bridge placement algorithms and heuristics
- Alternative terms: tabs, bridges, holding tabs, onion skin, micro-joints, tie-ins, break-away tabs, skin tabs
- Shared/aligned tabs between adjacent nested pieces
- Tab sizing (width, height/depth) optimization
- Workholding strategies that interact with tab placement

**Out of scope:**
- General CNC nesting algorithms (unless they specifically address tab placement)
- Vacuum hold-down systems (different approach entirely, but mention if relevant as alternative)
- Manual tab placement workflows (unless they document heuristics that could be automated)
- CNC machine hardware/firmware details

## Known Starting Points

- No specific URLs provided by user
- The Thetta project (current working directory) is a Rust-based nesting tool that could benefit from tab placement features
- jagua-rs crate (part of workspace) handles nesting geometry

## Success Criteria

- Catalog of all known terminology across CNC domains
- At least 2-3 algorithmic approaches to automatic tab placement
- Understanding of what factors affect optimal tab placement (piece weight, perimeter length, corner proximity, material properties)
- Any existing open-source implementations that could be adapted
- Clear understanding of the "shared tab" problem for adjacent nested pieces
- Enough detail to inform implementation of a tab placement feature in the Thetta nesting tool

## Open Questions

1. What is the mathematical formulation for optimal tab placement (if any formal treatment exists)?
2. How do you determine the minimum number of tabs needed for a given piece?
3. What heuristics work best for placing tabs away from visible/critical edges?
4. How can tabs between adjacent nested pieces be aligned/shared?
5. Is there academic literature on this, or is it primarily practitioner knowledge?
6. What role does material type and thickness play in tab sizing and count?
7. Are there any patents covering automatic tab placement algorithms?
