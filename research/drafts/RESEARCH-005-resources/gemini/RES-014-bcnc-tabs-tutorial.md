# bCNC Tabs Tool Tutorial
> **URL**: https://github.com/vlachoudis/bCNC/wiki/Tutorials:-Tabs-tool
> **Type**: Community
> **Agent**: Gemini
> **Relevance**: Medium
> **Credibility**: High

## Summary
This wiki tutorial provides the user instructions for generating tabs within the bCNC software (analyzed in RES-013). It outlines a very specific workflow for adding tabs to a toolpath just before sending it to the GRBL controller.

The workflow dictates that the user must select the "last pass block" in the G-code editor, and then use the tabs tool to execute the addition. By default, bCNC attempts to place 3 tabs on this final pass. This confirms that bCNC's approach to tabs is strictly limited to modifying the final depth cut, rather than planning multi-pass 3D tabs.

The tutorial emphatically reiterates the warning found in the repository analysis: users must uncheck the "only leave selected islands uncut" option in the cut tool settings. This reinforces the technical finding that bCNC's CAM engine treats holding tabs as identical to geometric islands.

## Key Findings
- In basic 2.5D CAM, tabs only need to be applied to the final depth pass, significantly simplifying the algorithm compared to full 3D ramped tabs.
- A default of 3 tabs is common across different software (aligning with findings in RES-001 and RES-021) as it is the minimum required to prevent a part from pivoting.
- The "island" approach to tab generation requires careful state management to ensure the tabs aren't accidentally machined away.

## Technical Details
The process modifies the existing toolpath block by injecting Z-retract and Z-plunge commands at three calculated points along the final XY perimeter pass.

## Strengths
- Provides clear insight into the user experience of an open-source, sender-integrated CAM feature.
- Confirms the "3-tab default" heuristic.

## Weaknesses & Limitations
- Very brief; acts more as a troubleshooting note than a comprehensive algorithm description.
- Modifying only the last pass restricts the thickness of the tabs to the depth of a single cutting pass.

## Connections
- Direct companion to RES-013 (bCNC source repository).
