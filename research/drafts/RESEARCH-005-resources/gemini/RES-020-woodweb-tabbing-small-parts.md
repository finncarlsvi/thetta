# Tabbing vs Onion-Skinning for Small Parts
> **URL**: https://woodweb.com/knowledge_base/Tabbing_Versus_OnionSkinning_for_Small_Parts.html
> **Type**: Community
> **Agent**: Gemini
> **Relevance**: Medium
> **Credibility**: Medium

## Summary
This knowledge base article from WoodWeb provides a practical comparison between tabbing and onion-skinning, specifically focused on the difficult task of securing small parts on a CNC router. It offers insights into the physical mechanics of workholding that algorithms must attempt to simulate or accommodate.

The article advocates for thick tabs (e.g., 1/4" thick) when absolute rigidity is required, noting that such tabs "totally lock everything" in place. However, the trade-off is significant post-processing time spent sanding. Onion-skinning (leaving a paper-thin layer of material across the entire bottom of the cut) is presented as an alternative that is much easier to clean up but provides less lateral rigidity.

A key strategic takeaway is the "two-pass" method. Even when using tabs or skins, operators recommend cutting the entire sheet down to 0.70" (in 0.75" material) on the first pass, and then doing a final, full-depth pass. This reduces the cutting forces applied to the delicate tabs/skins during the final separation, drastically reducing the chance of parts breaking loose.

## Key Findings
- Thick tabs prioritize machine reliability and safety over post-processing convenience.
- Multi-pass cutting (leaving tabs/skins only for a low-force final pass) is a critical strategy for small parts.
- The necessity of tabs is inversely proportional to the effectiveness of the vacuum hold-down and the size of the part.

## Technical Details
Onion skinning leaves a continuous ~0.05" layer of material along the entire path. Tabbing leaves discrete, thicker bridges. The physics involve balancing the lateral cutting force of the router bit against the shear strength of the tab material and the friction of the vacuum table.

## Strengths
- Provides excellent practical context on why tabs are used and how they physically fail.
- Details the multi-pass cutting strategy, which must be supported by the CAM generator.

## Weaknesses & Limitations
- Highly specific to wood routing with vacuum tables; less applicable to metal or laser cutting.
- No algorithmic placement rules provided.

## Connections
- Provides the practical reasoning behind the multi-pass and onion-skinning features discussed in HOMAG's documentation (RES-018).
- Supports the small-part logic threshold seen in CamBam (RES-010).
