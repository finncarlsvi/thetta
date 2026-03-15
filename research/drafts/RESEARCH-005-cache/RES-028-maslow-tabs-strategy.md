# RES-028: Maslow CNC Tabs Strategy Discussion
**URL**: https://forums.maslowcnc.com/t/help-with-tabs-strategy/432
**Type**: Community
**Status**: CACHED

## Content
### Tab Sizing
- 0.255" thick tabs for 0.500" material
- Width = 1.5× depth value
- Tab size should scale with material thickness

### Multi-pass Strategy
1. Shallow cut stopping just short of full depth with normal step-downs
2. Identical overlaying path cutting completely through except at tabs
3. Final pass cuts remaining thin tabs bottom-to-top

### Removal Methods
- Oscillating multi-tools (~$19)
- Sharp chisels (small tabs)
- Japanese flush-cut pull saws (straight edges)
- Dremels (small tabs)
- Heavy-duty scissors (~$7)
