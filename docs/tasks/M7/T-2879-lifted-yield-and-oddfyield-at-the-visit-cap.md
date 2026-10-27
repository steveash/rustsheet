---
id: T-2879
title: "Lifted YIELD and ODDFYIELD at the visit cap run about 17 s: refit the solver weight"
milestone: M7
lane: perf
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
One lifted YIELD or ODDFYIELD call at the per-call visit cap (`limits::MAX_LIFTED_CELL_VISITS`, 2^27) runs in at most
about 5 s of a release build, as SPEC-0011 "Security rule for kernel work" intends, without turning any eval cell into
`#ERROR!`.

## Context
Found by T-2814 (release build, 2026-10-27). `YIELD_SOLVER_WEIGHT` (2^24, functions/lift/kinds/financial.rs) assumes a
price term costs 17 ns (a 32,400-coupon price took 0.55 ms), but where the discount's `pow` takes its slow path near
overflow a term costs about 340 ns, so a Newton run to its 100-step cap costs about 67 us per coupon. Measured, one call
at the cap: 17.3 s for 644 elements of `YIELD(43831,80355,0.05,1E-100,100,4)` (27 ms each), 13.7 s at 30 years
quarterly, 10.9 s at 10 years semiannual (`YIELD(43831,47484,0.05,1E-100,100,2)`, 12,329 elements); ODDFYIELD about
17 s (`ODDFYIELD(43831,DATE(2200,1,1),43800,DATE(2020,4,1),0,1E-100,100,4)`, 47 ms an element, 360 elements). With
the weight at 2^26 those measured 4.26 s (161 elements), 2.47 s (3,085) and 4.27 s (89), but `make eval-check` had two
regressions:
- gen-m9/79d87fc527927626 Calc!A270, `YIELD((OFFSET(..)+INT(2^53+1)),45641,..)`: an off-calendar settlement is charged
  a full walk, though `Model::bond_args` reads all seven arguments (errors, logicals and off-calendar dates refused)
  before `CouponSchedule::new`; setting YIELD's `reads_all_first: true` (as the ODD* entries, T-2815) fixes it.
- gen-m9/0a957119475760c5 Calc!A116, `YIELD(TOCOL(Data!A1:C5,0,..),..)`: four non-date texts (`a*`, `ABC`, `x~*`,
  `abc`) in the settlement are each charged the full bound, since `ArgValue::Text` may read as any date. Charging a
  text by what `Model::cast_number` reads it as (a text that reads as no number is `#VALUE!` before any walk) would
  fix it; `kernel_units` has no model today, so this needs the text's reading carried into the charge.
Raise the weight only together with those two charge refinements. ODDLYIELD, PRICE, IRR and XIRR were timed by T-2814
and are within their weights.

## Acceptance criteria
- [x] YIELD's and ODDFYIELD's charge is fitted so the slowest inputs above take at most about 5 s at the cap; the PR
      records the timings.
- [x] No baseline gains a line (`make eval-check`: 0 regressions), including the two cells above.
- [x] docs/excel-semantics/financial.md "Yield and odd-coupon work cap" and SPEC-0011's as-built note give the new numbers.
- [x] `make check` green; reviewers approve.

## Out of scope
- RATE (T-2814).

## Outcome
`YIELD_SOLVER_WEIGHT` is 2^26 (from 2^24). Two charge refinements come with it, so no eval cell turns into `#ERROR!`:
- YIELD is `reads_all_first` (`Model::bond_args` reads all seven arguments before `CouponSchedule::new`).
- `kernel_units` now takes the model's `cast_number`. For a `CouponWalks` kernel, a text is charged as the number it reads as, or as an error (`#VALUE!` before any walk) when it reads as no number. A scalar is read once, and a text over 1,020 bytes is not parsed.

Release-build timings, one call at the cap:
- 161 elements of `YIELD(43831,80355,0.05,1E-100,100,4)`: 4.34 s.
- 535 of `YIELD(43831,54788,0.05,1E-100,100,4)`: 0.75 s.
- 3,085 of `YIELD(43831,47484,0.05,1E-100,100,2)`: 2.60 s.
- 89 of `ODDFYIELD(43831,DATE(2200,1,1),43800,DATE(2020,4,1),0,1E-100,100,4)`: 4.25 s.
- One element more is `#ERROR!` in 0.01 s.

Eval delta: none (`make eval-check`: 0 regressions in every suite, including gen-m9 79d87fc527927626 A270 and 0a957119475760c5 A116). financial.md and SPEC-0011 give the new numbers.
