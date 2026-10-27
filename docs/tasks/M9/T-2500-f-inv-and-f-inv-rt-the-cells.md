---
id: T-2500
title: "F.INV and F.INV.RT: the cells still within a unit or two of the 15th digit after T-2465 (Excel's own error)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
campaign: visible
depends_on: [T-2465]
---

## Goal
The F quantile cells T-2465 left a unit or so from Excel's 15th digit display Excel's 15 digits, with no new
known-failure lines.

## Context
T-2465 fixed the large-shape prefactor (`incomplete_beta.rs` `ln_near_one`). After that, these gen-m9 cells (excel-graph)
are within about a unit of the 15th significant digit but still display differently (units of the 15th digit in
brackets, measured 2026-10-23):
- 27601819397dd56e A81 `F.INV.RT(0.05,99999,9)`: Excel 2.7067247920355353, ours 2.7067247920355326 (0.27). The 50-digit
  root is 2.70672479203553401, so ours is the correctly rounded side and Excel is 5 ulp high.
- c18dbf21f11093f3 A190 `F.INV.RT(0.001,99999,1)`: Excel 636616.2559081278, ours 636616.2559081267 (1.05). The true root is
  636616.25590812706: Excel is 6 ulp high and ours 4 ulp low.
- 320d51765af9b6b7 A379 `F.INV(0.5,3,9)` (1.11), 4ab2ba98d26c8a5e A390 `F.INV(0.999999,3,1.999)` (0.98), 589f42e67f4b4b8d
  A365 `F.INV(0.5,1,3)` (0.78), 60b0b6e69ad35a45 A197 `F.INV(0.975,6,1)` (0.80), a61e547a1cab994c A297
  `F.INV.RT(Data!D$2,7,1)` (0.71), c2974c061384fdd9 A267 `F.INV(SQRT(ACOTH(123456789.123)),2,9)` (0.68).
- probes/stat-quantile-lift E6 `F.INV(1E-50,6000,4)`: Excel 0.0004924127886173762, ours 0.016348828 (no statrs start,
  bisection of our cdf; T-1871 territory).
- Gap triage 2026-10-25: ten more gen-m9 cells of the same kind that no task named (excel-graph; Excel, ours):
  320d51765af9b6b7 A15 `F.INV.RT(POWER(1/3,0.2),(COUNTIFS(Data!B2:E8,"<>")+1),2)` 0.5769171267631172, ...166;
  40425341c5034eed A108 `FINV(CHITEST(VSTACK(Data!D10,Data!C12),Data!C11:D11),1,1)` 0.5866909757268702, ...692;
  60b0b6e69ad35a45 A150 `F.INV.RT(LET(..),9,10)` 0.9923205895406533, ...524; 7674c89c824aca43 A212 `FINV(0.025,10,6)`
  5.461323718731779, ...796; b6593d7dd13f2dac A188 `FINV(Data!D10,3.14159,10)` 0.8450805765917135, ...142;
  c18dbf21f11093f3 A92 `F.INV(0.5,1,3)` and caa1eed48b1883fe A359 (same value) 0.5850602740518204, ...211;
  e6cf3b20a014e6a3 A135 `F.INV.RT(0.025,6,1.999)` 937.1110834482022, ...029; eced4bb6a0234c9e A202 `F.INV.RT(0.5,5,7)`
  0.9602597305806843, ...833; eced4bb6a0234c9e A289 `FINV(0.975,100,DOLLARFR(3.14159,13))` 0.3077283973670394, ...3883.
These are crossings of a cdf a few ulp from Excel's, so matching them means matching Excel's F.DIST/F.DIST.RT kernel to
the last ulp near each crossing (excel-probe dense runs of F.DIST.RT around these x), or Excel's quantile search's own
stopping rule (T-1745 found BETA.INV to be a crossing of Excel's cdf).

More visibly-off gen-m9 cells (gap triage 2026-10-26, `make eval` on 1c8b3d34, excel-graph), the same crossing problem;
no task listed them. Their 1-5 ulp neighbours on the sweep are T-2767's (deferred):
- gen-m9/eced4bb6a0234c9e Calc!A202 `=F.INV.RT(0.5,5,7)`: Excel 0.9602597305806843, ours 0.9602597305806833 (9 ulp)
- gen-m9/eced4bb6a0234c9e Calc!A289 `=FINV(0.975,100,DOLLARFR(3.14159,13))`: Excel 0.3077283973670394, ours 0.30772839736703883 (10 ulp)
- gen-m9/caa1eed48b1883fe Calc!A359 `=F.INV(((0.3-Data!A9)-SORT(Data!$E1)),IF((Data!C7<=Data!B1),ISNUMBER(1/0),"1"),3)`: Excel
  0.5850602740518204, ours 0.5850602740518211 (7 ulp)
Cells:
- gen-m9/caa1eed48b1883fe.case.json (1): Calc!A359
- gen-m9/eced4bb6a0234c9e.case.json (2): Calc!A202, A289

## Acceptance criteria
- [ ] The cells above display Excel's 15 significant digits, or the Outcome shows from probes why Excel's value cannot
      be reached without a bit-level model of its kernel (then a follow-up without the campaign key).
- [ ] No new known-failure lines in any `eval/baseline/` file.
- [ ] `make check` green; reviewers approve.

## Out of scope
- BETA.INV (T-2119); bit-exact F quantiles beyond the cells above.
