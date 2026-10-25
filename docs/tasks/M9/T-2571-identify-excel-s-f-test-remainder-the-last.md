---
id: T-2571
title: "Identify Excel's F.TEST remainder: the last visibly-off F.TEST cells after T-2361"
milestone: M9
lane: research
status: done
priority: P2
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
F.TEST/FTEST reach visible parity on every F.TEST cell on file: Excel's remainder R (the factor
after exp(a ln x) exp(b ln y), docs/excel-semantics/statistics.md "F.TEST / FTEST: larger variance
on top") is identified, or the cells below are within half a unit of Excel's 15th digit without
losing a cell that is bit-exact.

## Context
Follow-up of T-2361 (its visible-parity criteria were not met; decision 0083). T-2361 fitted the
structure: larger variance over the smaller, upper tail exp(a ln x)*exp(b ln y)*R with R in
double-double (`f_test_upper_tail`, `f_test_remainder` in
crates/rustsheet-engine/src/functions/statistical/fisher.rs). Over 188 distinct inputs that model
is bit-exact on 90, within 4 ulp on 170 and visibly off on 6; Excel's residual is -12..+14 ulps,
so Excel's R is a double computation not yet found. Candidates scored so far are in
statistics.md (Cephes' double continued fraction 63/188, a double 2F1 series 74/188).
The visibly-off cells after T-2361 (excel-graph oracle values):
- `gen-m9/58e3253bbd3a7a82.case.json` Calc!A141 `FTEST(Data!A2:D10,Data!E1:E11)`: Excel 0.5407985695568127, ours 0.540798569556814 (+12 ulp)
- `gen-m9/8f2ac25945af87da.case.json` Calc!A119 `F.TEST(Data!A9:D12,Data!A10:D12)`: Excel 0.657910021992518, ours 0.6579100219925169 (-10 ulp)
- `gen-m9/8f2ac25945af87da.case.json` Calc!A241 `F.TEST({0.02,0.027},Data!D6:D7)`: Excel 8.912676813146147e-18, ours 8.912676813146168e-18 (+14 ulp)
- `gen-m9/a7bf0d8d81d33ba6.case.json` Calc!A81 `FTEST(Data!B5:E8,Data!C2:E4)`: Excel 0.5027827884435259, ours 0.5027827884435264 (+5 ulp)
- `gen-m9/b6593d7dd13f2dac.case.json` Calc!A306 `F.TEST(Data!B6:E12,Data!A1:D10)`: Excel 1.23966888990022e-69, ours 1.2396688899002132e-69 (-26 ulp)
- `gen-m9/bcb24385baafb2f0.case.json` Calc!A377 `FTEST(Data!B7:E9,FILTER(Data!A1:D12,(Data!D1:D12>7)))`: Excel 6.4145381857304026e-93, ours 6.414538185730395e-93 (-9 ulp)
- `gen-m9/c468f5046aadfbbc.case.json` Calc!A192 `F.TEST(Data!C6:D7,Data!A9:C9)`: Excel 6.909871037288267e-08, ours 6.909871037288261e-08 (-4 ulp)
- `gen-m9/caa1eed48b1883fe.case.json` Calc!A380 `F.TEST(Data!A10:C12,Data!A5:C10)`: Excel 0.08405253726549103, ours 0.08405253726549108 (+4 ulp)
- `gen-m9/f34f66fb7f81e673.case.json` Calc!A4 `FTEST(Data!A1:C10,Data!A1:A12)`: Excel 0.8776459265593823, ours 0.8776459265593836 (+11 ulp)
- `gen-m9/fec39efaba9284c3.case.json` Calc!A36 `F.TEST(Data!B9:C11,Data!A1:C10)`: Excel 0.0807011926950238, ours 0.08070119269502367 (-9 ulp)
T-2361 also took decision 0106 for 17 cells that were bit-exact before and are now 1-5 ulp off
(listed in T-2361's Outcome); they are this task's bit-level targets, not visible-parity ones.
Ideas: Excel's R as a forward or reverse sum in doubles of the 2F1 series with the Gamma ratio
applied at a different point; Cephes' incbcf/incbd chosen by a different switch; the remainder of
the A&S 26.6.4-26.6.8 finite series (df even or odd) divided by its own powers; the
`F.TEST({0.02,0.027},..)` cell may instead be VAR.S's one-pass variance (statistics.md "VAR/STDEV
switch"). Score each over the F.TEST cells on file, extracted from the case files as (x, y, Excel p) rows the way T-2361 did.

Unprobed corners (T-2361 review): an overflowing variance ratio, `FTEST({1E-160,2E-160},{1E150,2E150})`
in both orders (the engine says `#NUM!`, as Excel does for an overflowing variance,
gen-m9/7674c89c824aca43 A39); large df where the double-double remainder hands over to
`beta_reg_complement` at a + b = 200 (e.g. df 1 against df 249 or 400). Probe both before fitting.
The T-2361 fit rows were extracted ad hoc and not committed: commit the extraction (tools/fitlab)
with this task so the scores are reproducible.

## Acceptance criteria
- [ ] (not met: 8 remain, follow-up T-2681) Visible parity: the 10 cells above have Excel's value to 15 significant digits, with no new
      known-failure lines in any baseline beyond decision 0083's allowance.
- [x] The candidate scores (bit-exact / within 4 ulp / visibly off) are recorded in
      docs/excel-semantics/statistics.md.
- [x] `make check` green; reviewers approve.

## Out of scope
- F.DIST, F.DIST.RT and BETA.DIST, which use the shared `beta_reg_complement` kernel.

## Progress (2026-10-25, opus run; resumed by a later opus run)
- Committed the extraction the Context asked for: fits `f-test` (candidate remainders R) and `f-test-structure`
  (variance, x/y and power alternatives) in tools/src/rustsheet_tools/fitlab/kernels/ftest.py, over every F.TEST/FTEST
  cell on file with plain-range arguments (291 numeric cells). Scores are in statistics.md under the F.TEST entry.
- Finding: no double-sum R (forward, backward, Gamma-ratio B) beats the correctly rounded R, and every R variant keeps
  the same worst cells, so the visibly-off cells' residual probably lies outside R.
- Found and implemented: df 1 against df 1 is Excel's closed form 2 asin(sqrt x)/pi (13 of 16 such cells exact);
  `f_test_upper_tail` uses it. 6 gen-m9 cells fixed, 2 of them on this task's list (8f2ac25945af87da A241, c468f5046aadfbbc
  A192); 8 visibly-off cells remain. Scored and rejected so far: double 2F1 sums, A&S 26.6.4 positive series for an even
  df_big, the finite sums from the asin base, log1p / log-difference powers, pairwise and one-pass variances (statistics.md).
- Next (was): candidates for the powers' arguments (ln x as log1p(-y), or ln(df_small) - ln(df_big f + df_small)), the
  variance's summation order (pairwise, column-major for 2-D ranges), and the unprobed corners listed in Context.


## Outcome
Partial (decision 0083 follow-up T-2681). Excel's F.TEST with one degree of freedom on each side is the closed form
2 asin(sqrt x)/pi (13 of 16 such cells bit-exact); `f_test_upper_tail` uses it, fixing 6 gen-m9 cells, 2 of this
task's 10 (8f2ac25945af87da A241, c468f5046aadfbbc A192). The F.TEST fit is committed (`rustsheet-fitlab f-test`,
`f-test-structure`, `f-test-finite`) and every candidate's score is in statistics.md. The remaining residual is in the
tail's evaluation, not in the variance ratio (the double f matches Excel; the exact f is further off) and not in R
alone (every R variant, NR `betai` in doubles and the alternative power forms score below the model). Eval delta:
gen-m9 known failures 6 fewer, no regressions. The unprobed corners move to T-2681.
