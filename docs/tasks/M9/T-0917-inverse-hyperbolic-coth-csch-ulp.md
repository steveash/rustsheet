---
id: T-0917
title: "ASINH/ACOSH/ATANH/ACOTH, COTH/CSCH, small-x TANH: finish Excel ulp parity"
milestone: M9
lane: functions-math
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-0914, T-1005]
---

## Goal
Remaining ulp-level mismatches in `ironcalc-calc/MATH_AND_TRIGONOMETRY/TRIGONOMETRIC.xlsx` after T-0914: ASINH 12, ACOTH 14, ATANH 4, ACOSH 3, COTH ~14, CSCH ~9, COT 17, plus TANH at x = 0.00625, 0.123, 0.604.

## Context
Kernel map (T-1522, decision 0056): rank 12 in `docs/research/numeric-kernel-map.md`; fit with the fit lab (T-1523).
T-0914 found SINH/COSH/TANH fit exp formulas (see `docs/excel-semantics/numbers.md`). COTH/CSCH did NOT fit the same building blocks: exp-based COTH/CSCH regressed COTH_CSCH_SECH!B10, B47, C54 which libm matches, so Excel's formulation there differs per range. Use `/tmp`-style throwaway fitting over the cached cells (parse the xlsx sheets), try log1p/sqrt based inverse forms, and keep only per-function variants with 0 regressions.

**Needs an oracle sweep (T-0018):** fitting this formulation needs more Excel points than the
cached workbooks give. Fit against the T-1005 sweep (`probes-numeric/m9-hyperbolic`, landed with findings in numbers.md), never blind.

## Acceptance criteria
- [ ] At least 40 F lines for the listed functions leave the baseline, no new ones.
- [ ] Oracle-probe (T-0004) the fitted SINH 0.5 / TANH 1.0 switchover thresholds, the 20 TANH cutoff and the 700 sinh/cosh overflow guard, and confirm the formulas between cached points.
- [ ] Findings (including negative results) in `docs/excel-semantics/numbers.md`.
- [ ] `make check` green.
