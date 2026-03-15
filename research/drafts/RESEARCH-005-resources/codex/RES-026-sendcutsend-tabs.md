# SendCutSend CNC Routing Tabs Documentation
> **URL**: https://sendcutsend.com/services/cnc-routing/tabs/
> **Type**: Product
> **Agent**: Codex
> **Relevance**: Medium
> **Credibility**: High

## Summary
SendCutSend’s documentation describes a fully automatic, service-side tab placement model for routed parts. Unlike interactive CAM tools, the user does not control tab positions directly.

This model is relevant because it reflects a production service constraint: automation must be robust enough to apply across large job diversity with minimal human intervention. The policy that tabs and lead-ins are kept outside part geometry prioritizes dimensional fidelity of delivered parts.

For research, this source highlights a pragmatic deployment stance: default-safe auto placement can be commercially viable even without user-customizable tab location, provided geometry preservation and easy post-removal are maintained.

## Key Findings
- Tab and lead-in placement is fully automatic and not user-configurable.
- Tabs are intentionally kept outside part geometry to preserve part shape fidelity.

## Technical Details
The system likely applies geometry-driven heuristics with manufacturability constraints optimized for throughput. From an algorithm perspective, this implies robust default rules, conservative placement boundaries, and deterministic behavior over broad part classes.

## Strengths
- Clear, production-proven automation policy with explicit geometry-integrity constraints.

## Weaknesses & Limitations
- No transparency into underlying algorithms; no user override path for special retention needs.

## Connections
- [RES-010-cambam-holding-tabs.md](/Users/finn.carlsvi/thetta/research/drafts/RESEARCH-005-resources/codex/RES-010-cambam-holding-tabs.md) — both represent deterministic auto-tab paradigms, but CamBam exposes knobs while SendCutSend abstracts them away.
