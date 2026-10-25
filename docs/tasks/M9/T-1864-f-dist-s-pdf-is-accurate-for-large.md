---
id: T-1864
title: "F.DIST's pdf is accurate for large degrees of freedom instead of #NUM! (probes-numeric t1530-f-df-cut)"
milestone: M9
lane: functions-stat
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
F.DIST's pdf (`cumulative` FALSE) gives Excel's value for large degrees of freedom instead of `#NUM!`.

## Context
statrs' `FisherSnedecor::pdf` (used by `f_pdf` in functions/statistical/fisher.rs) raises `df2` to the power `df2` and
`df1*x + df2` to the power `df1 + df2`, so it overflows to inf/inf (NaN, then `#NUM!`) whenever a degrees of freedom is
above about 143, although the density is an ordinary number: probes-numeric/t1530-f-df-cut `F.DIST(2,5,1E10,FALSE)` is
0.14167277676179968 in Excel (ours `#NUM!`; the suite is not gated). T-1290 tried a log-space form there and found it
not accurate enough: `F.DIST(1,1000,1000,FALSE)` gave 6.306254544584165 against the true 6.3062545445902005, and the
t1530 cell 0.14167422298660368. So T-1290 kept its logarithmic fallback only for an overflow that comes from x alone
(`df1^df1` and `df2^df2` finite; `F.DIST(1E308,1,3,FALSE)` is 0, probes/stat-overflow-arguments E11), and left the
large-df pdf `#NUM!` (the `TODO(T-1864)` in fisher.rs). An accurate form is needed, for example the binomial-density
form R's `df()` uses (Loader's `dbinom_raw` with `bd0`/`stirlerr`), then checked against every F pdf cell of
probes-numeric and ironcalc-calc `F_DIST.xlsx` without moving any value that passes today.

gen-m9 sweep (gap triage 2026-10-23, T-2255). `F.DIST(WRAPROWS(Data!A1,..),123456792.26459,1,FALSE)` (x 0.1, df
1.2E8 and 1): Excel 0.085003679623665, ours `#NUM!` (1 cell).
Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 1 known-failure lines in eval/baseline/gen-m9.txt, 1 formulas.
Examples:
- gen-m9/285d87cea1aaa73c Calc!A64 `=F.DIST(WRAPROWS(Data!A1,LET(acc,Data!$E$12,w,0.2,100),(2.5*0.2)),(123456789.123+ABS(Data!E6)),1,FALSE)`: Excel 0.085003679623665, ours #NUM!
Cells:
- gen-m9/285d87cea1aaa73c.case.json (1): Calc!A64

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2255): these 1 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] probes-numeric/t1530-f-df-cut's F.DIST pdf cells match Excel (or within the documented ulp tolerance), and no
  passing F.DIST/F.DIST.RT/FDIST cell changes (compare every suite cell by cell with main).
- [x] The rule and the form are recorded in docs/excel-semantics/statistics.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2255): the gen-m9 cells listed under Context are visibly off (a wrong type, error or
text, or a number further than half a unit in the 15th significant digit from Excel's), so the task is in the campaign. Its
target is that every visibly-off cell it owns has Excel's type, error and text, or is within half a unit of the 15th
significant digit of Excel's value (an expected 0 exactly 0); the visible-parity criterion under Acceptance criteria states
it. Bit-level criteria still unmet then move to a follow-up task without the key.

## Outcome
Where statrs' direct F density overflows (a df above about 143, or a huge x whose log form does not underflow),
`f_pdf` now uses Loader's saddle-point binomial form as R's `df` does (`fisher.rs` `f_pdf_loader`; new
`functions/statistical/loader.rs` with `stirlerr`, `bd0`/`bd0_dev` and `dbinom_raw_dev`, meant for T-2507 and T-2585).
Two departures from R keep digits it loses: the binomial deviation is formed without cancellation and passed to bd0, and
ln(2πx(1 - x/n)) is one logarithm with 1 - x/n from the exact n - x (R's form was 4E-8 relative off at
`F.DIST(2,1E10,5,FALSE)` and 9,000 ulp at `F.DIST(57400,50,1,FALSE)`). Unit tests (test_fn_f_pdf_large_df.rs) check 10
large-df cells against the exact density (mpmath): 0-4 ulp, and about 100 ulp on the 2.2E-67 tail of
`F.DIST(100,50,100,FALSE)`. Eval against the run before: 607 F.DIST pdf cells went from `#NUM!` to numbers (13 now
bit-exact, 229 within 15 digits of Excel; ours is a median 1.9E-15 relative off the exact density on 545 of them,
Excel's own value 3.2E-15, so the rest is Excel's own noise, T-2586); no other cell of any suite changed. t1530-f-df-cut
A9 is bit-exact and A10 is 1 ulp from Excel and equal to the exact density. gen-m9 285d87cea1aaa73c A64 is 1 ulp from
Excel (0.08500367962366501 against 0.085003679623665, the exact density's double), so it is no longer visibly off
(visible-parity count 8326 to 8321, left for the steward's lock) but its known-failure line stays: the first criterion
is open at the bit level, which needs the exponent summed in double-double (its rounding is 0.67 ulp of the result
there). Recorded in docs/excel-semantics/statistics.md, "F density at large degrees of freedom (T-1864)".
