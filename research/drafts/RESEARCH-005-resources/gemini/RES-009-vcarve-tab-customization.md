# VCarve/Aspire Tab Customization
> **URL**: https://www.learnyourcnc.com/blog/how-to-customize-design-tabs-in-vectric
> **Type**: Community
> **Agent**: Gemini
> **Relevance**: Medium
> **Credibility**: High

## Summary
This blog post details the tab placement capabilities within Vectric's VCarve and Aspire software, which are highly popular in the woodworking and router CNC communities. It outlines both the automatic and manual tools available for securing parts.

Vectric offers an automatic placement tool based on a minimum/maximum number of tabs and a maximum distance between them. Importantly, users can seamlessly transition from automatic to manual mode, repositioning generated tabs by dragging them along the contour, or using click-to-delete and click-to-add interactions.

A standout feature discussed is "3D Tabs." Unlike standard rectangular tabs which require sharp Z-axis plunges and retracts, 3D tabs create a smooth, continuous ramp up to the tab thickness and back down. This continuous motion prevents the tool marks and burning that often occur when a router bit stops moving laterally to change its Z-height.

## Key Findings
- 3D tabs (ramped tabs) are a highly desirable feature as they provide a smoother cutting motion, reducing tool marks and machine wear.
- An effective UX pattern for tab placement combines automatic generation with drag-and-drop manual repositioning along the toolpath.
- Tab size parameters are strictly defined on the Toolpath screen, separate from the geometric design phase.

## Technical Details
3D tabs alter the G-code generation from discrete XYZ moves to coordinated multi-axis moves. Instead of `G0 Z_up; G1 X_move; G1 Z_down`, a 3D tab is generated as a continuous `G1 X_move Z_up` followed by `G1 X_move Z_down`, smoothing the velocity profile of the machine.

## Strengths
- Highlights the mechanical benefits of 3D/ramped tabs over standard rectangular ones.
- Describes a highly effective user interface paradigm (auto-generate + drag-to-edit).

## Weaknesses & Limitations
- Does not detail the mathematical logic behind the automatic minimum/maximum distance distribution.
- Focuses heavily on the software UI rather than the underlying algorithms.

## Connections
- The 3D tab concept contrasts with the standard Rectangular/Triangular options in Fusion 360 and MecSoft (RES-005, RES-006).
