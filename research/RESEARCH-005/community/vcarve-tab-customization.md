# VCarve/Aspire Tab Customization -- Learn Your CNC

> **URL**: https://www.learnyourcnc.com/blog/how-to-customize-design-tabs-in-vectric
> **Type**: Community
> **Agents**: Claude, Codex, Gemini
> **Relevance**: Medium
> **Confidence**: High -- agents agree on feature set

## Summary
This blog post covers tab customization in Vectric's VCarve and Aspire software, popular CAM tools in the woodworking CNC community. VCarve offers a dual-mode workflow: automatic placement with min/max tab count and distance constraints, plus interactive post-placement editing where users can drag tabs to reposition, click to delete, or click to add. This provides the best of both worlds.

The standout feature is 3D Tabs. Instead of the standard retract/plunge approach (leaving rectangular/triangular tabs), 3D tabs create a smooth ramping motion. The tool makes a continuous 3D arc up to the tab thickness and back down, resulting in smoother profiling passes and tabs that are easier to remove with less visible marking. At the G-code level, this means coordinated `G1 X_move Z_up` / `G1 X_move Z_down` rather than discrete `G0 Z_up; G1 X_move; G1 Z_down`.

## Key Findings
- Automatic placement with min/max tab count and distance constraints (all agents)
- Interactive editing: drag to reposition, click to add/delete even on auto-placed tabs (Claude, Gemini)
- 3D Tabs: smooth ramping motion for easier removal and less marking (all agents)
- G-code difference: 3D tabs use coordinated XZ moves, standard tabs use discrete Z moves (Gemini)
- Tab sizes configured on the Toolpath screen separately from placement (Claude)

## Technical Details
- **Software**: VCarve Pro, Aspire (Vectric)
- **Automatic parameters**: Min tabs, Max tabs, Distance between tabs
- **Placement editing**: Drag-to-reposition, click-to-add, click-to-delete
- **Tab types**: Standard (retract/plunge) and 3D (smooth ramp)
- **G-code**: Standard = `G0 Z_up; G1 X_move; G1 Z_down`; 3D = `G1 X_move Z_up; G1 X_move Z_down`
- **Target market**: Woodworking CNC, sign making, furniture

## Strengths
- Best-in-class UX: automatic placement with interactive refinement
- 3D tabs are a unique feature reducing post-processing effort and tool marks
- Min/max clamp on tab count prevents over/under-tabbing

## Weaknesses & Limitations
- No details on the internal automatic placement algorithm
- No support for adjacent/shared tabs between nested parts
- Commercial product -- closed source

## Consensus & Disagreements
All agents agree on the dual-mode workflow and 3D tab innovation. Gemini provides the most concrete G-code-level detail. Claude notes the lack of shared-tab support. No factual disagreements.

## Connections
- [RES-005] MecSoft Bridges & Tabs -- similar auto/manual dual-mode
- [RES-010] CamBam Holding Tabs -- similar min/max constraint model
- [RES-006] Fusion 360 Tabs Reference -- Fusion lacks VCarve's interactive editing and 3D tabs
- [RES-027] Mastercam Tabs -- Mastercam's angle parameter bridges toward 3D tabs
