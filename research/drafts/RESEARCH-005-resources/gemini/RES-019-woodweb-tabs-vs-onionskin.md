# Tabs vs Onion Skinning (WOODWEB Forum)
> **URL**: https://www.woodweb.com/cgi-bin/forums/cnc.pl?read=818617
> **Type**: Community
> **Agent**: Gemini
> **Relevance**: Medium
> **Credibility**: Medium

## Summary
This discussion from the WOODWEB CNC forum revolves around the practical challenges of cutting small parts and the debate between using holding tabs versus onion skinning. The context is professional woodworking where vacuum tables are standard, but often insufficient for small surface areas.

A critical point raised by the community is the failure mode of unaligned tabs. Operators note that if CAM software places tabs on adjacent parts but fails to align them perfectly across the shared gap, the tool will cut through the first part's tab while profiling the second part. This results in the first part becoming "disconnected" and potentially flying loose, damaging the bit or the part.

The discussion indicates that while some advanced CAD/CAM systems have specific options to ensure tab alignment between adjacent parts, many operators resort to manual placement or manual CAD workarounds to guarantee stability, especially for repeated nesting layouts where the manual effort pays off over multiple runs.

## Key Findings
- Misaligned tabs between closely nested parts are a major source of job failure and tool damage in professional shops.
- Automatic tab alignment across gaps is a premium feature not found in all CAM software.
- Manual tab placement is often preferred for high-volume, repeated nests because operators cannot trust naive automatic algorithms.

## Technical Details
The problem occurs when the gap between parts is less than or equal to two tool diameters. The toolpath offset for Part B extends into the safe Z-zone established by the tab on Part A. Only global path awareness can prevent this.

## Strengths
- Highlights real-world, financial consequences of poor tab placement algorithms.
- Confirms that the shared-tab problem is widespread in the industry.

## Weaknesses & Limitations
- Anecdotal forum discussion without technical solutions.
- Original source message has expired, limiting deep analysis.

## Connections
- Echoes the exact frustrations found in the Autodesk forum (RES-008).
- Highlights the value of the HOMAG common-line tab solution (RES-018).
