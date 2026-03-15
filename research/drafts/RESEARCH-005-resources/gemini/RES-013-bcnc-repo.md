# bCNC GRBL CNC Sender with Cutout Tabs
> **URL**: https://github.com/vlachoudis/bCNC
> **Type**: Code
> **Agent**: Gemini
> **Relevance**: Medium
> **Credibility**: High

## Summary
bCNC is a popular, open-source GRBL CNC command sender written in Python that includes lightweight built-in CAM functionalities. Unlike heavy CAD/CAM suites, bCNC is designed for quick operations directly at the machine, including the addition of cutout tabs.

The software supports profiling, pocketing, and cutout tabs directly within its "cut" tool. The implementation of tabs here is very straightforward, intended for quick 2D routing jobs (often wood or acrylic) rather than complex 3D machining.

A critical operational quirk in bCNC is that the tab functionality is deeply tied to the island-detection logic. By default, the software has a setting called "only leave selected islands uncut" checked. For tabs to generate properly, this setting must be explicitly unchecked, revealing how the software internally treats tabs as small, un-machined islands along the toolpath.

## Key Findings
- bCNC treats holding tabs algorithmically as small "islands" of un-machined material along a contour path.
- The integration of tabs into a CNC sender (rather than upstream CAM) shows the demand for last-minute fixturing adjustments at the machine interface.
- Python is highly viable for implementing functional CAM and tab-generation tools.

## Technical Details
bCNC is built on Python and uses basic geometric offsetting and island detection to generate paths. When tabs are requested, the software interrupts the final depth pass, treating the tab locations as protected islands that the tool must lift over.

## Strengths
- Highly accessible, open-source Python codebase.
- Demonstrates a simplified "island" approach to tab generation.

## Weaknesses & Limitations
- Tab generation is basic and sometimes counter-intuitive (requiring specific checkbox toggles to work).
- Designed for simple 2.5D routing, lacking the sophisticated geometric analysis needed for complex nests.

## Connections
- The tutorial for using this specific feature is detailed in RES-014.
- Shares a Python-based CAM approach with viaconstructor (RES-015).
