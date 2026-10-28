---
id: T-3072
title: "An error or one-cell TAKE/DROP result under arithmetic is a scalar, not a lifted 1x1 array (CHITEST, SUMXMY2, ROWS)"
milestone: M7
lane: functions-lookup
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`ROWS(E9-TAKE(E7,2,42))` is `#VALUE!` and `CHITEST(E9-TAKE(E7,2,42),..)` is `#VALUE!` as in Excel (ours 1 and `#N/A`): a one-cell TAKE/DROP under arithmetic is a scalar.

## Context
Split from T-2519, which made a one-cell TAKE/DROP of a range a reference in value context (`take_drop_reference`); that fixed `+0` and `N()` under OFFSET but not the binary-operator path: `E9-TAKE(E7,2,42)` (E7 text) still gives a 1x1 array (a scratch test showed `ROWS(..)` = 1, and `E9-E7`, `E9-INDEX(..)` and `E9-OFFSET(..)` are fine). TAKE/DROP carry `.array()` in `TAKE_DROP` (functions/lift/kinds/general.rs), the likely cause. T-2519 context holds the T-2436 evidence (probes/t2436/<case>).
Remaining gen-m9 cells (3 known-failure lines, eval/baseline/gen-m9.txt):
- gen-m9/7674c89c824aca43 Calc!A305 `=CHITEST((Data!E$9-TAKE(Data!E$7,2,42)),Data!C8:C10)`: Excel #VALUE!, ours #N/A
- gen-m9/9cca17438a7e6f8a Calc!A171 `=INTERCEPT(Data!B4:B4,DECIMAL(TAKE(Data!$B11,-2),MATCH(Data!E12,Data!A4:C11)))`: Excel #N/A, ours #VALUE!
- gen-m9/7c20351562a2d71d Calc!A277 `=CONFIDENCE.NORM(SUMXMY2(TAKE(Data!C11,-1,1),Data!B6:C10),1E-300,FISHERINV((0.1/Data!D3)))`: Excel #VALUE!, ours #N/A (SUMXMY2 of a one-cell TAKE vs a 5x2 range: `#VALUE!`, not `#N/A`)

## Acceptance criteria
- [ ] The 3 gen-m9 cells above leave eval/baseline/gen-m9.txt (`make eval-baseline`), no new known-failure lines.
- [ ] Also probe `SUMXMY2(<scalar>,<range>)` vs a one-cell TAKE (excel-probe) before changing SUMXMY2.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
