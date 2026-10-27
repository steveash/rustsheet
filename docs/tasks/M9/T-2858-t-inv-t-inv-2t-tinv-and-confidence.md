---
id: T-2858
title: "T.INV, T.INV.2T, TINV and CONFIDENCE.T at one degree of freedom are 1/TAN(pi*p/2) through Excel's TAN (68 cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
With a deg_freedom that truncates to 1 (and CONFIDENCE.T with a size that truncates to 2), T.INV, T.INV.2T, TINV and
CONFIDENCE.T return Excel's doubles from the Cauchy closed form Excel evaluates, `1/TAN(PI()*q/2)` through Excel's own
TAN, instead of statrs' search.

## Context
Gap triage 2026-10-27 (opus refill). The rule and its evidence are in docs/excel-semantics/statistics.md "Student-t
quantile at one degree of freedom is 1/TAN(π·p/2) through Excel's TAN (T-2858)". With `t2(q)` = `1/excel_tan(π·q/2)`:
- T.INV.2T and TINV: `t2(p)` for p <= 1, `-t2(2 - p)` for p in (1, 2) (2 - p is exact there, Sterbenz);
- T.INV: `-1/excel_tan(π·p)` for p < 0.5, `1/excel_tan(π·(1 - p))` for p > 0.5 (1 - p is exact there), 0 at 0.5
  (the 0.5 cell is unprobed: keep today's value);
- CONFIDENCE.T: `t2(alpha) * sd / sqrt(2)` (the existing `t_crit * std_dev / size.sqrt()` with today's
  truncated size; ironcalc CONFIDENCE.xlsx E22 `CONFIDENCE.T(0.05,1,2)` = 8.984643532093761 matches it).

The triage evaluated exactly these forms with the engine's TAN (`=1/TAN(PI()*B1/2)` with B1 holding p) on a scratch case
for all 112 failing df = 1 cells in eval/: all 112 match bit for bit, 68 of them visibly off today. Use the same
products: `PI()*p/2` is `std::f64::consts::PI * p / 2.0`, evaluated left to right, then `1.0 / excel_tan(..)`.
Excel's numbers are not the true quantile near p = 1 (`TINV(1-1E-10,1)` is 6E-7 off the true value); that is the
point: libm's tan or a more accurate form (`tan(π(1-p)/2)`) does not give them.

Code:
- `t_inv_of` (functions/statistical/t_dist.rs) and `confidence_t_of` (functions/statistical/normal.rs): add the
  df == 1 branch before the `StudentsT` path, keeping every argument check and error as it is (p = 0 is `#DIV/0!`
  at df 1, p outside the range `#NUM!`).
- `excel_tan` is `pub(crate)` in functions/math_and_trigonometry/trig.rs; call it, do not copy it.
- The tail path `t_tail_quantile` and the huge-df series do not change for df >= 2.

Failing cells (eval/, 2026-10-27; 30 lines in gated baselines, the rest in the ungated probes-numeric suite). Visibly
off are 68, for example gen-m9/5389dd4d5b5739d2 A299 `TINV(1-1E-10,1)`: Excel 1.570797438519081e-10, ours
1.0536712127723509e-08; gen-m9/334f3dc9ca937f10 A23 `TINV(1E-300,1)`: Excel 6.366197723675814e+299, ours
6.366197723675218e+299; ironcalc T_DIST.xlsx D15 `T.INV.2T(0.5000000000000001,1)`: Excel 0.9999999999999996, ours
1.0000000000000016.
- gen-m9/0072265d385d8ab2.case.json (1): A242
- gen-m9/097af1ca33638670.case.json (1): A175
- gen-m9/329b177b5d5165ad.case.json (1): A203
- gen-m9/334f3dc9ca937f10.case.json (2): A23, A293
- gen-m9/3b7990bd4e3307ba.case.json (1): A58
- gen-m9/43c6227e0ff368c7.case.json (1): A100
- gen-m9/5389dd4d5b5739d2.case.json (1): A299
- gen-m9/5395b8a427764a3f.case.json (1): A394
- gen-m9/873f30d2bf76f8db.case.json (1): A123
- gen-m9/91e167cd8d7eb652.case.json (1): A386
- gen-m9/9bc247733a1ca9fa.case.json (1): A17
- gen-m9/a4090567758520df.case.json (1): A307
- gen-m9/afc00c8b70e78baf.case.json (1): A312
- gen-m9/c2684066d8df263e.case.json (1): A374
- gen-m9/d208bbafc3bc1997.case.json (1): A97
- gen-m9/ede0323c389fc703.case.json (1): A262
- gen-m9/f82473275380882e.case.json (1): A320
- ironcalc-calc/STATISTICAL/CONFIDENCE.xlsx (2): CONFIDENCE.NORM CONFIDENCE.T!E3, E22
- ironcalc-calc/STATISTICAL/T_DIST.xlsx (4): T.DIST T.INV T.DIST.RT!E2, E15, E28, E38
- ironcalc-calc/STATISTICAL/T_DIST.xlsx (6): T.DIST.2T T.INV.2T!D2, D15, D37, D52, D56, D57
- probes-numeric/m9-confidence-t.case.json (10): A1, A6, A11, A16, A21, A26, A31, A36, A41, A51
- probes-numeric/m9-t-edges.case.json (2): A9, A10
- probes-numeric/m9-t-inv.case.json (12): A1, A2, A3, A4, A5, A6, A8, A9, A10, A11, A12, A13
- probes-numeric/m9-t-inv-2t.case.json (13): A1-A13
- probes-numeric/m9-t-nonint-df.case.json (6): A17, A18, A23, A24, A29, A30
- probes-numeric/t1048-tinv.case.json (2): A5, B5
- probes-numeric/t1531-t-tiny-p.case.json (37): A1, A2, A3, A4, A6, A7, A10-A21, A25-A28, A30, A31, A34, A35, A37,
  A38, A40, A41, A43, A44, A121, A123, A125, A127, A129

These cells are part of T-0967's scope, which waits on T-0969 (blocked, time-boxed); this split needs no new CDF.

## Acceptance criteria
- [ ] The 30 gated known-failure lines above (18 gen-m9, 12 ironcalc-calc) disappear from eval/baseline/gen-m9.txt and
      eval/baseline/ironcalc-calc.txt (`make eval-baseline`), with no new known-failure line in any baseline.
- [ ] The 82 probes-numeric cells above pass in `make eval` (ungated suite; name any that do not in the Outcome).
- [ ] Unit tests in t_dist.rs for `T.INV.2T(0.5,1)` = 1, `T.INV.2T(1.5,1)` = -1, `TINV(1-1E-10,1)` =
      1.570797438519081e-10, `TINV(1+2^-52,1)` = -5.053466325954936e-16, `T.INV(1E-16,1)` = -3183098861837907,
      `T.INV(0.9,1)` = 3.0776835371752544 and `CONFIDENCE.T(0.05,1,2)` = 8.984643532093761.
- [ ] statistics.md's T-2858 entry gets an "Implemented in" line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- df >= 2 (T-2859 accuracy, T-2839 above p = 1, T-1542 tiny p and huge df, T-0967 bits).
- T.DIST and friends at df = 1 (the cdf side).

## Visible-parity stage
Decision 0083: done when every cell this task owns is within half a unit of the 15th significant digit of Excel's
value. The closed form is expected to be bit-exact on all of them; any cell left only at the ulp level moves to a
`defer: phase-1b` follow-up named in the Outcome.
