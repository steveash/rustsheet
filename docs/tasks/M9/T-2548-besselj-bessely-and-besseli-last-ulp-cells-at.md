---
id: T-2548
title: "BESSELJ, BESSELY and BESSELI last-ulp cells at x = 84 to 1E+5 in the gen-m9 engineering sweep (12 cells)"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-2547]
---

## Goal
The BESSELJ, BESSELY and BESSELI last-ulp cells of the gen-m9 sweep at x = 84 to 1E+5 match Excel to the bit.

## Context
Gap triage of the T-2258 gen-m9 engineering sweep. None is visibly off (each within half a unit of the 15th digit), so no campaign key: `BESSELJ(1000,3)`, `BESSELJ(1000,5)`, `BESSELJ(700,2)`, `BESSELJ(84,2)`, `BESSELJ(1E+5,0)`, `BESSELJ(1E+5,4)`, `BESSELY(1000,0)`, `BESSELY(1000,2)`, `BESSELY(714,4)`, `BESSELY(1E+5,1)`, `BESSELY(3299.36..,4)` (1 to 3 ulps) and `BESSELI(100.218..,5)` 1.177055823943151E+42 (ours ...507E+42). Fit after T-2547 (which fixes the model above 100 on T-2140's sweep); `BESSELJ(84,2)` is inside the 8..100 range the model is said to match, so check it first.

Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 12 known-failure lines in eval/baseline/gen-m9.txt, 12 formulas, 0 visibly off.
Examples:
- gen-m9/12e4aa13a20cc1f8 Calc!A257 `=BESSELY(714,4)`: Excel -0.0025082601200159195, ours -0.00250826012001592
- gen-m9/408aa11aa923a4d9 Calc!A268 `=BESSELY(1000,2)`: Excel -0.004765486623916302, ours -0.0047654866239163025
- gen-m9/4e180f54ff140abc Calc!A236 `=BESSELJ(1E+5,4)`: Excel -0.0017193488511191089, ours -0.001719348851119112
- gen-m9/4e180f54ff140abc Calc!A377 `=BESSELJ(1000,3)`: Excel -0.004827420803793426, ours -0.004827420803793427
- gen-m9/4fffa5addab1f8af Calc!A319 `=BESSELY(1E+5,1)`: Excel 0.0017192103480940782, ours 0.0017192103480940812
- gen-m9/59a16b5ab566ee5c Calc!A299 `=BESSELI(((Data!$B11+(-2.5))+EXP(Data!D3)),5)`: Excel 1.177055823943151e+42, ours 1.1770558239431507e+42
Cells:
- gen-m9/12e4aa13a20cc1f8.case.json (1): Calc!A257
- gen-m9/408aa11aa923a4d9.case.json (1): Calc!A268
- gen-m9/4e180f54ff140abc.case.json (2): Calc!A236, A377
- gen-m9/4fffa5addab1f8af.case.json (1): Calc!A319
- gen-m9/59a16b5ab566ee5c.case.json (1): Calc!A299
- gen-m9/65dc4e4da669f98a.case.json (1): Calc!A131
- gen-m9/7dc52cfcbc8b4448.case.json (1): Calc!A199
- gen-m9/895a33aa8352864a.case.json (1): Calc!A171
- gen-m9/8c018ae063ac131b.case.json (1): Calc!A263
- gen-m9/ca182d96dedee546.case.json (1): Calc!A126
- gen-m9/df083d85b436b49a.case.json (1): Calc!A98

## Acceptance criteria
- [ ] These 12 known-failure lines disappear from eval/baseline/gen-m9.txt (`make eval-baseline`), with no new
      known-failure lines in any baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
