# HOMAG Stay-Down, Common Line, Tabs, Onion Skinning
> **URL**: https://docs.homag.cloud/en/news/article/stay-down-common-line-include-tabs-onion-skinning
> **Type**: Product
> **Agent**: Codex
> **Relevance**: High
> **Credibility**: High

## Summary
HOMAG’s documentation describes an integrated nesting strategy combining stay-down motion, common-line cutting, retaining bars (tabs), and onion skinning. This is one of the few sources that explicitly positions tabs as part of a coordinated multi-feature machining strategy.

The resource is valuable because it shows how commercial systems treat tab logic as one lever among several for productivity and stability, rather than a standalone toggle. In particular, common-line cutting and tabs are discussed together, which is directly relevant to adjacent-part coordination.

For the research question, this source supports a systems view: optimal tab placement should account for retraction minimization, shared cuts, vacuum limitations, and multi-pass release timing. That broader coupling is likely necessary for robust behavior in dense nests.

## Key Findings
- HOMAG combines stay-down, common-line, tabs, and onion-skinning in one nested workflow to reduce time and improve part stability.
- Retaining bars are configurable by size and quantity and targeted especially at small parts with weak vacuum hold.

## Technical Details
The documented workflow appears in woodWOP/intelliDivide tooling and requires specific software/plugin versions for tab support. Functionally, this implies a planner that tracks part size/stability risk and applies multiple strategy layers (shared cuts + hold features + staged release) in one operation plan.

## Strengths
- Integrated, production-oriented treatment of tabs alongside adjacent optimization features.

## Weaknesses & Limitations
- Vendor documentation does not disclose underlying optimization algorithms or scoring criteria.

## Connections
- [RES-020-woodweb-tabbing-small-parts.md](/Users/finn.carlsvi/thetta/research/drafts/RESEARCH-005-resources/codex/RES-020-woodweb-tabbing-small-parts.md) — community practice on small-part hold-down mirrors HOMAG’s emphasis on tabs/onion-skinning for vacuum-limited cases.
