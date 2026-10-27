---
id: T-2548
title: "BESSELJ, BESSELY and BESSELI last-ulp cells at x = 84 to 1E+5 in the gen-m9 engineering sweep (12 cells)"
milestone: M9
lane: functions-engineering
status: blocked
priority: P3
size: S
tier: opus
campaign: visible
depends_on: [T-2547, T-2881]
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

T-2547 (2026-10-24) gave the asymptotic form Excel's own sine (`excel_sin_cos`): that fixed 5 of the 12 lines above
(12e4aa13a20cc1f8 A257, 408aa11aa923a4d9 A268, 4e180f54ff140abc A236 and A377, 4fffa5addab1f8af A319). It also hands over
`BESSELJ(1E+5,100000)` (gen-m9/2b0324e2de2e7b74 Calc!A295): Excel 0.009636944011336267, ours 0.00963694401133629 (Miller's
recurrence, n = x; visibly off in the 15th digit). Tried without a match: the step as `j*tox*bj`, `bj*(j*tox)`,
`bj*(2j*(1/x))`, `(2j/x)*bj`, `j*(tox*bj)`, and NR's ACC 160 and 100 for the starting order.

## Acceptance criteria
- [ ] These 12 known-failure lines and gen-m9/2b0324e2de2e7b74 Calc!A295 disappear from eval/baseline/gen-m9.txt (`make eval-baseline`), with no new
      known-failure lines in any baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other T-2258 triage families: each has its own task (T-2258 Outcome).

## Visible-parity stage
Decision 0083/0114: this task is done when every visibly-off cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value. Bit-level criteria still unmet then move to a follow-up task filed without the key and with `defer: phase-1b`.
Gap triage 2026-10-25 (eval on origin/main becefc6f3): the task was deferred by decision 0114's adoption, but an eval shows a cell it owns visibly off, so, as its Deferred note asked, the `defer` key gave way to `campaign: visible`. Visibly off: gen-m9/2b0324e2de2e7b74 Calc!A295 `=BESSELJ(1E+5,100000)`: Excel 0.009636944011336267, ours 0.00963694401133629 (4.6 units of the 15th digit).

## Blocked because
Blocked because: needs the T-2881 oracle sweep of `BESSELJ` near the turning point n = x; one cell cannot pick a model.
Findings (2026-10-27, opus run, mpmath at 40 digits; data from a scratch analysis, not committed):
- The true `J_100000(100000)` is 0.0096369440113378623. Ours (0.00963694401133629) and Excel's (0.009636944011336267) are
  both about 1.6E-13 low (relative), so Excel does run Miller's recurrence here. Upward recurrence from J0 and J1 at `ax >= n` gives
  0.009636943998118515, far off.
- Almost all of the shared error comes from `tox = 2/x` rounding to a double (relative error +8.18E-17; near the turning
  point the result is about 2,000 times as sensitive to it). With that double `tox` and exact arithmetic everywhere
  else, the result is 0.0096369440113363024. Excel's value is 3.5E-17 below that, ours 1.2E-17 below it, so what is left is
  per-step rounding noise.
- Tried without a match, each with the NR start order m = 102000:
  - x87 64-bit steps, kept in registers (0.009636944011336303) or stored to double each step (...336333);
  - FMA `fl(j*tox)*bj - bjp` (...336523);
  - `j*fl(tox*bj) - bjp` (...336154);
  - one rounding of `j*tox*bj - bjp` (...336324).
  These scatter by about 2E-16 around Excel's value, so a fit needs the T-2881 points (several n near x).
