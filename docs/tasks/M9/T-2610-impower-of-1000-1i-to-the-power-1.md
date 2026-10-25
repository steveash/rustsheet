---
id: T-2610
title: "IMPOWER of 1000+1i to the power 1: polar form must leave the imaginary part inexact (gen-m9 A379)"
milestone: M9
lane: functions-engineering
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`IMPOWER("1000+1i",1)` is "1000+1i" as Excel writes it (gen-m9/74779cff2810ccf4 Calc!A379).

## Context
Split from T-2533. After the text fix, ours still writes "1000+i" because `fn_impower` (complex/mod.rs) computes `radius*sin(angle)` as exactly 1.0 for 1000+1i to the power 1, while Excel's imaginary part is not exactly 1 (engineering.md, "1i" when a part is not exactly 1). Find the Excel kernel (radius or angle) that leaves it inexact; use the excel-probe skill.

## Acceptance criteria
- [x] gen-m9/74779cff2810ccf4 Calc!A379 leaves eval/baseline/gen-m9.txt (`make eval-baseline`), no new known-failure lines.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`fn_impower` (engineering/complex/mod.rs) takes the angle from Excel's ATAN2 kernel `excel_atan2` (`atan(y/x)`, as
IMARGUMENT does) instead of libm `atan2`, falling back to `atan2` where that kernel errs. `atan(0.001)` is one ulp
above `atan2(1,1000)`, so `IMPOWER("1000+1i",1)`'s imaginary part is 1.0000000000000002 and prints "1000+1i".
Unit test `impower_angle_is_excels_atan2_kernel`; engineering.md updated. Eval: gen-m9 4324 -> 4323
(74779cff2810ccf4 Calc!A379), no regressions in any suite.
