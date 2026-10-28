---
id: T-3017
title: "MEDIAN of an even count is a + (b - a)/2, the lower middle value plus half the gap (10 gen-m9 and 22 calipers cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
defer: phase-1b
---

## Goal
MEDIAN of an even number of values returns Excel's bits, `a + (b - a)/2` where a <= b are the two middle values, so
the 32 known-failure lines below leave the baselines.

## Context
Gap triage 2026-10-28 (`make eval` on dea28aa5 with calipers, corpus-bench and poi fetched). Every cell below is
1 or 2 ulp off, never visibly off, so the task is deferred (decision 0114). It is split from T-2476 because the
formula is already settled and needs no fit.
- `fn_median` (crates/rustsheet-engine/src/functions/statistical/count_and_average.rs) returns `(a + b) / 2`, or
  `a/2 + b/2` when `a + b` overflows. AGGREGATE(12, ..) calls the same function (functions/aggregate.rs).
- Fit, done during triage (no new evidence needed). I took every gen-m* cell whose formula is `MEDIAN(<Data ranges>)`
  with an even count, read the numbers of those ranges (text, booleans and blanks skipped) and compared the
  candidates with the oracle value. There are 50 such cells (excel-graph):
  - `a + (b - a)/2` matches all 50;
  - `(a + b)/2`, `a/2 + b/2`, `b - (b - a)/2` and `(a + b)*0.5` each match 44.
  The 6 cells only `a + (b - a)/2` matches are the failing ones below. The other failing cells, whose arguments mix
  constants and arrays, also fit by hand (A336, A239, A286, A117).
- Examples:
  - gen-m9/73f053591a54bc43 Calc!A307 `=MEDIAN(Data!E9:E9,Data!D2:D11)`, middle pair -0.5 and 0.3:
    Excel -0.09999999999999998, ours -0.1.
  - gen-m9/775d053de498b8c7 Calc!A162 `=MEDIAN(Data!C11:C11,Data!C8:C12)`, middle pair -0.0001 and 7:
    Excel 3.4999499999999997, ours 3.49995.
  - calipers/pv_goldens/pv23_051/golden.xlsx WACC!E23 `=MEDIAN(E16:E21)`, middle pair 12.302999999999999 and
    43.90399999999999 (Mac desktop cached): Excel 28.103499999999997, ours 28.103499999999993.
- Edge: when `b - a` overflows (a = -9E+307, b = 9E+307), no Excel cell is known. Keep the result finite the way the
  current halving fallback does. The probe aggregate-overflow (`MEDIAN` over 9E+307 twice is 9E+307, numbers.md) must
  still pass. File a probe task for the opposite-sign overflow; do not guess it.
- docs/excel-semantics/statistics.md "MEDIAN of an even count is a + (b - a)/2" records the rule (added by this triage).

Cells (known-failure lines today):
- gen-m9/201d67f1d59436ed.case.json (1): Calc!A314
- gen-m9/5389dd4d5b5739d2.case.json (1): Calc!A31
- gen-m9/6e18550e9fc9c1b9.case.json (1): Calc!A336
- gen-m9/73f053591a54bc43.case.json (2): Calc!A239, A307
- gen-m9/775d053de498b8c7.case.json (2): Calc!A162, A286
- gen-m9/91e167cd8d7eb652.case.json (1): Calc!A117
- gen-m9/bbab6562e88b70e8.case.json (1): Calc!A170
- gen-m9/eedd629f3d69fdda.case.json (1): Calc!A189
- calipers pv23_046/golden.xlsx, pv23_049/golden.xlsx, pv23_051/golden.xlsx, pv23_052/golden.xlsx, pv23_053/golden.xlsx,
  pv23_054/golden.xlsx, pv23_055/golden.xlsx (3 each): WACC!E23, Public Comps!F17, Public Comps!F33
- calipers pv23_050/golden.xlsx (1): WACC!E23

## Acceptance criteria
- [ ] The 10 gen-m9 lines above disappear from eval/baseline/gen-m9.txt, and the 22 calipers lines from
      eval/baseline/calipers.txt (run `rustsheet-eval fetch --suite calipers` before `make eval-baseline`). No new
      known-failure line appears in any baseline.
- [ ] Unit tests in count_and_average.rs (or its test module) pin the three examples above and the 9E+307 pair.
- [ ] statistics.md's MEDIAN entry is marked implemented.
- [ ] `make check` green; reviewers approve.

## Out of scope
- MEDIAN cells that pass an inner function's last digit through (gen-m9/80586b859b73c87d A264 FDIST,
  gen-m9/cb61169f4c033d4b A168 LN(VARPA)): T-2476 and the inner function's task keep them.
- TRIMMEAN, SKEW, STDEV and the rest of T-2476.
- PERCENTILE and QUARTILE interpolation. Check them for the same form only if a failing cell shows it.
