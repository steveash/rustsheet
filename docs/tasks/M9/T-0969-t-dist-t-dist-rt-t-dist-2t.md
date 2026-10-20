---
id: T-0969
title: "T.DIST/T.DIST.RT/T.DIST.2T/T.TEST: accurate Student-t CDF with direct upper tail (55 known failures)"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
campaign: visible
depends_on: [T-1744, T-2088]
---

## Goal
T.DIST (cumulative and density), T.DIST.RT, T.DIST.2T and T.TEST return Excel's values from one
Student-t CDF kernel that computes the upper tail directly instead of as `1 - CDF`.

## Context
Kernel map (T-1522, decision 0056): rank 3 in `docs/research/numeric-kernel-map.md`; fit with the fit lab (T-1523).
Triage T-0903. Kernel: `statrs::distribution::StudentsT::cdf`, i.e. `0.5 * beta_reg(df/2, 0.5, df/(df+x^2))`
(statrs' Numerical-Recipes continued fraction, ~1e-14..1e-10 relative error) and `StudentsT::pdf`
(statrs Lanczos `ln_gamma`). Code: `functions/statistical/t_dist.rs` (`fn_t_dist`, `fn_t_dist_2t`
and `fn_t_dist_rt` compute `1.0 - dist.cdf(x)`; `fn_t_test` computes `1.0 - cdf` too).
55 baseline lines in `eval/baseline/ironcalc-calc.txt` (40 NumericClose, 14 NumericUlp, 1 NumericMismatch):
- `ironcalc-calc/STATISTICAL/T_DIST.xlsx` `T.DIST T.INV T.DIST.RT`: D2, F2(C), D3(C), F3(C), F4(C), D5, D6, F6(C), D7, F7(C), D8, F8(C), D10(C), D12, F13(C), D14(C), D15, F15, D16(C), F16(C), D17(C), F17(C), F18(M), D19(C), D20(C), F20(C), D21(C), F21(C), D28, F28, D32(C), D38, F38, D40(C), D52(C)
- `ironcalc-calc/STATISTICAL/T_DIST.xlsx` `T.DIST.2T T.INV.2T`: C2(C), C3(C), C4(C), C6(C), C7(C), C8(C), C13(C), C15, C16(C), C17(C), C18(C), C20(C), C37
- `ironcalc-calc/STATISTICAL/T_TEST.xlsx` `Sheet1`: F2(C), F3(C), F4(C), F5(C), F6(C), F7(C), F8(C)
(C = NumericClose, M = NumericMismatch, unmarked = NumericUlp.)

Examples (expected vs actual):
- `T.DIST.RT(1E6,10)` F18: expected 1.230468749943611e-56, actual 0.0 (Mismatch: catastrophic `1 - cdf`).
- `T.DIST(1E-6,11,TRUE)` D20: expected 0.500000389989757, actual 0.5000003900287391 (7.8e-11 relative,
  statrs' incomplete beta near h = 1).
- `T.DIST(1,100,TRUE)` D17: expected 0.8401379221079384, actual 0.8401379221078716 (7.9e-14).
- `T.DIST(1,1,TRUE)` D15: expected exactly 0.75, actual 0.7499999999999998; `T.DIST.RT(1,1)` F15:
  expected 0.25000000000000006, actual 0.2500000000000002.
- `T.TEST(...)` F3: expected 0.3017830576436469, actual 0.3017830576436453 (5e-15).

Excel quirk (recorded in `docs/excel-semantics/statistics.md`, "T.DIST.RT and T.DIST.2T are not
1 - T.DIST"): T.DIST(1,1) = 0.75 but T.DIST.RT(1,1) = 0.25000000000000006, so the tails are computed
separately; `T.DIST(1,2,TRUE)` = 0.7886751345948129 = 0.5 + 1/(2*sqrt(3)).

Hypothesis (the oracle is the judge): T.DIST/T.DIST.RT/T.DIST.2T truncate df to an integer, so
Excel may use the closed-form finite series for integer df (Hill 1970 ACM 395 / AS 3: atan series for
odd df, sqrt series for even df), with the upper tail evaluated directly from the same series; T.TEST
type 3 has a non-integer (Welch) df, so it needs an incomplete-beta path too (share T-0970's kernel).
Try the closed form first on the cached cells; keep whichever reproduces them; never `1 - cdf` for a
tail. Put the kernel in `functions/statistical/special.rs` (one definition; T-0967 builds T.INV on it).

More cells (T-1289's gen-m9 suite, excel-graph, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/4df1b22d3ed63f58 Calc!A9 `=CHIINV(T.DIST(0.1,7,Data!D1),2)`: Excel 1.2382105318705625, engine 1.2382105318705663 (engine T.DIST(0.1,7,TRUE) = 0.5384259701976026); shared with T-0972.

Oracle sweep (T-1043): probes-numeric cases `m9-t-dist-cdf`, `m9-t-dist-pdf`, `m9-t-dist-rt`, `m9-t-dist-2t`, `m9-t-nonint-df`, `m9-t-edges`; findings in docs/excel-semantics/statistics.md "Student t sweep".

Rejected hypothesis (2026-10-20, opus run): Hill's integer-df series (AS 3: atan series for odd df,
sqrt series for even df), with the lower tail as 0.5 ± A/2, against probes-numeric m9-t-dist-cdf: 74/150
cells bit-exact (the engine has 78) and 117 within 4 ulp. Its premise about df = 1E6 was wrong; see
attempt 2.

## Attempt 2 (2026-10-20, opus run): Excel's CDF is the exact CDF; only the tails carry noise
Reference: the exact CDF from mpmath `betainc` at 60 digits, with y = n/(n+t^2) and z = t^2/(n+t^2)
taken exactly (I_y(n/2,1/2)/2 when y <= z, else 1/2 - I_z(1/2,n/2)/2).
- **Correction.** At df = 1E6 Excel is within 1 ulp of the exact CDF (`T.DIST(1,1E6,TRUE)`,
  `T.DIST(-2,1E6,TRUE)`, `T.DIST(0.1,1E6,TRUE)`: 1, 1 and 1 ulp). The "152,191 ulp" figure above comes
  from the fit lab's `t_cdf`, which rounds y to a double before I_y in the lower tail: the exact I at
  the rounded y reproduces 152,191 ulp (and 43,561 at t = -10) exactly. Fit-lab fix: T-2088.
- **T.DIST (cumulative)**: the exact CDF, rounded, is bit-exact on 127 of 160 probes-numeric cells
  (engine 78 on m9-t-dist-cdf; 118 of its 150 cells, the other 10 in m9-t-nonint-df and m9-t-edges), 155 within 4 ulp, worst 45 ulp (`T.DIST(-10,1000,TRUE)`). On `T_DIST.xlsx` it is
  bit-exact on every T.DIST cell but D6 (1 ulp) and D19 (33 ulp, the deep tail `T.DIST(-1E6,10,TRUE)`).
- **T.DIST.RT / T.DIST.2T**: T.DIST.2T is exactly 2 * T.DIST.RT on every cell. Excel's upper tail is
  not 1 - T.DIST (it fails on 11 of the 21 `T_DIST.xlsx` T.DIST/T.DIST.RT row pairs: rows 2, 4,
  6, 7, 8, 13, 15, 16, 17, 18, 28) and not 0.5 - c with c the centre
  I_z(1/2,n/2)/2 rounded (RT(5,30) is then about 12,050 ulp off): it is computed directly, near the exact
  value but with algorithm noise: exact RT is bit-exact on 79 of 167 cells and within 4 ulp on 134.
  On `T_DIST.xlsx` the exact tails are 0 to 4 ulp off except F7, F8, C7, C8 (`T.DIST.RT(1.96,60)`:
  13 ulp) and F18 (33 ulp).
- **Deep tail (y <= z)**: Excel's noise grows like |a ln y| eps, the error of exp(a ln y). Cephes'
  power series (`pseries`, b x <= 1) with x^a = exp(a * ln y), y = n/(n+t^2) as a double, and a
  correctly rounded Gamma ratio Gamma(a+b)/(Gamma(a) Gamma(b)) (ln Gamma when a + b >= 171.6),
  product `x^a * s * ratio`, is within 4 ulp on 131 of 135 deep-tail cells (38 bit-exact, worst 7);
  the correctly rounded I at the double y has worst 269 (`T.DIST.RT(1E6,30)`; 275 from the exact value). Rejected on the same cells: ln y as
  -log1p(t^2/n) (worst 498) or ln n - ln(n+t^2) (worst 464), one exp() over ln-prefactor + a ln y
  (worst >= 201), libm tgamma ratio (worst 5 to 6, fewer within 4 ulp), x^a by pow (worst 269),
  NR betai (worst 449), LibreOffice GetBetaDist (worst 96).
- **Large df**: full Cephes `incbet` with exp-log powers (the deep tail above, the continued
  fraction elsewhere) is 1,500 to 1E11 ulp off from df = 1000 up: its prefactor (1 - y)^b uses
  1 - y with y rounded. Excel is accurate there, so a kernel must take z = t^2/(n+t^2) directly.

Next attempt, in this order: (1) a kernel that is the exact CDF to <= 1 ulp for |t| < sqrt(n)
(centre path, incl. df to 1E10: a continued fraction for I_z(1/2,a) with z and 1 - z both passed in
exactly); (2) the deep tail by the pseries form above; (3) score both with the fit lab (after T-2088)
before touching the engine. That alone does not meet the acceptance criteria: F7, F8, C7, C8 and
F18 stay above 4 ulp unless Excel's tail noise is reproduced, so the 4-ulp criterion may need to be
re-cut per decision 0056 once (1) and (2) land.

## Attempt 3 (2026-10-20, opus run; merged as progress, decision 0082)
`functions/statistical/student_t.rs` now holds the kernel: one tail L = P(T > |t|), T.DIST(t>0) =
1 - L, T.DIST.RT(x<0) = 1 - L, T.DIST.2T = 2 L (Excel's T.DIST.RT(x) = T.DIST(-x) on every swept
pair); L exact in double-double (Hill's integer-df series) for y > z and df <= 1000; Cephes'
power series with exp((n/2) ln y) and a correctly rounded Gamma ratio in the deep tail (y <= z);
Cephes' fraction (`beta_reg_upper`) for the centre above df 1000 (follow-up T-2126); the density
in double-double for integer df <= 1000. probes-numeric bit-exact: T.DIST 78 -> 118 / 150,
T.DIST.RT 34 -> 73 / 150, T.DIST.2T 24 -> 51 / 123, density 9 -> 38 / 150. Gated: 27 of the 55
lines left `ironcalc-calc.txt`; three cells regressed (decision 0082).
Still open, in `T_DIST.xlsx`: F7, F8, C7, C8 (`T.DIST.RT(1.959999998,60)` and row 8: 13 ulp from
the exact tail, Excel's tail noise) stay NumericClose; the other remaining listed lines are
NumericUlp (D3, D5, D6, D10, D19, D32, D40, D52, F3, F4, F6, F16, F17, F18, C3, C4, C6, C16, C17;
T_TEST F3, F4, F6, F7, F8). The density cells (D3, D10, D32, ...) are Excel noise of 1 to 4 ulp
around the correctly rounded density. Next: model the centre-tail noise (scan the 31 swept tail
cells > 4 ulp: (3,100) 23 ulp, (5,1000) 65, (30,1000) 125 in the exact-centre region) and the
density's exp(a ln y) form; then re-cut the 4-ulp criterion per decision 0056 if no model fits.

## Attempt 4 (2026-10-20, opus run; merged as progress, decision 0082)
- Above df 1000 (integer df), the centre no longer uses Cephes' fraction on the rounded y, and no longer statrs'
  cdf above 1E7. `centre_tail` in `student_t.rs` takes every input from z, t^2/n and n/t^2:
  - up to t = 1, `incbcf` for the centre I_z(1/2, n/2), at most 6 steps;
  - beyond t = 1, `incbd` on n/t^2 for L itself, under 160 steps.

  Integer-df series tails below 1E-12 take the same path. The density from df 1000 on is ratio / sqrt(n) *
  exp(-((n+1)/2) log1p(t^2/n)), replacing statrs above 1000. The density at a truncated df of 0 is `#DIV/0!`.
- Visibly-off T.DIST/T.DIST.RT/T.DIST.2T/TDIST/T.TEST/TTEST cells went from 128 to 69 (59 fixed, 0 newly visible, 19 more
  bit-exact). No baseline line changed. The ungated probes-numeric cases carry the gain: m9-t-dist-cdf/-rt/-2t/-pdf at
  df 1E6, m9-t-edges, m9-t-nonint-df and t1518-t-df-cut at df 1E10 and up, plus the five `#DIV/0!` cells.
- What is left is Excel's own noise (statistics.md "Student t sweep", "Large df stays exact"): in deep-ish tails and
  densities Excel is tens to hundreds of ulp from the exact value. Our tails, and our densities up to |t| = 5, are within
  a few ulp of it. Our large-t densities carry their own |E| eps error (-96 ulp at `T.DIST(30,1000,FALSE)`, +148 at
  (30,1E6)). T.TEST/TTEST is
  20 cells and the integer-df centre noise (T.DIST.2T(3,30), (2,100), (0.1,2)) another 10. Next: model the density's
  exp(E) error at every df (38 to 200 ulp, df 30 to 1E6) and the T.TEST noise. A kernel that is only more accurate cannot close
  these.

## Attempt 5 (2026-10-20, opus run; merged as progress, decision 0082)
- T.TEST/TTEST: Excel takes `I_y(df/2, 1/2)` at y = df/(df+t^2) rounded to a double, not the tail at t
  (statistics.md "T.TEST rounds y = df/(df+t^2) to a double before the tail"). t1048-ttest B31 is bit-exact
  that way and 42 ulp off at the exact t.
- `student_t.rs`:
  - `student_t_tail_at(y, n)` feeds the same kernel from a double y. The kernel's inputs are now
    `TailArguments` / `SeriesArguments`, so the t path computes exactly what it did before.
  - The fractional (Welch) df takes the accurate centre path. Its Gamma ratio is a recurrence up to the
    asymptotic series instead of libm's tgamma.
- Paired t^2 is md^2 / (vd / n).
- Visibly-off T.TEST/TTEST cells went from 19 to 3, with none new. The 3 left are Welch, at 5 to 6 ulp:
  t1048-ttest A3/B3 and t1049-ttest A48.
- Gated: 4 known failures left the baselines (`T_TEST.xlsx` F3, F6; probes/stat-argument-modes H540, H546), with
  0 regressions.
- T.DIST, T.DIST.RT and T.DIST.2T are unchanged.
- Not cracked: the density's noise. Its error is not a rounded exponent E: no double form of
  E = ((n+1)/2) ln(1+t^2/n) predicts Excel at df <= 100, with residuals of tens of ulp.

## Acceptance criteria
- [ ] No T.DIST/T.DIST.RT/T.DIST.2T/T.TEST line listed above remains with a NumericClose or
      NumericMismatch verdict (`rustsheet-eval show` on both cases), at least 30 of the 55 lines
      leave `eval/baseline/ironcalc-calc.txt`, and no new lines appear in any baseline.
- [ ] Every remaining line is <= 4 ulp and listed in the PR with expected/actual; they stay for the
      oracle-sweep follow-up (T-1043).
- [ ] `T.DIST.RT(1E6,10)` returns 1.230468749943611e-56 (unit test), no `1 - cdf` in the tail paths.
- [ ] Findings (which formulation matched) added to the `docs/excel-semantics/statistics.md` entry.
- [ ] `make check` green; reviewers approve.

## Out of scope
T.INV/T.INV.2T/CONFIDENCE.T (T-0967); the incomplete-beta kernel for BETA.DIST/F.DIST (T-0970);
legacy TDIST/TTEST evidence (T-1048).

## Fit result (T-1744)
Through I_x (mapped via the smaller of n/(n+t^2) and t^2/(n+t^2), which large n needs, but I_y/2 directly for a lower tail under 0.1: 0.5 - I_z/2 cancels to 0 at T.DIST(-10,1000,TRUE) = 8.3E-23), Cephes incbet is bit-exact on 105 of 150 T.DIST cells and the correctly rounded value on 113, against 78 today; no candidate is Excel's algorithm yet.

## Visible-parity stage
Decision 0083: this task is done when every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 must be exactly 0). Criteria above that ask for bit-exact or ulp-level matches and are still unmet then move to a follow-up task (no campaign key), named in the Outcome. Regressions that stay within 4 ulp follow decision 0083's allowance.

Visibly-off cells on the 2026-10-20 eval run (`main` 35a5e71), blamed on this function rather than on glue around it: 470 in total.
- T.DIST: 161 cells (13-14 digits right 87, 10-12 45, 6-9 29, fewer or 0 vs nonzero 0); most in `probes-numeric/m9-t-dist-pdf.case.json` 96; `probes-numeric/m9-t-dist-cdf.case.json` 42. e.g. `T.DIST(1,2,TRUE)` Excel 0.7886751345948129, ours 0.7886751345948135
- T.DIST.RT: 114 cells (13-14 digits right 47, 10-12 28, 6-9 16, fewer or 0 vs nonzero 23); most in `probes-numeric/m9-t-dist-rt.case.json` 93; `ironcalc-calc/STATISTICAL/T_DIST.xlsx` 10. e.g. `T.DIST.RT(1,2)` Excel 0.21132486540518708, ours 0.21132486540518647
- T.DIST.2T: 108 cells (13-14 digits right 43, 10-12 27, 6-9 15, fewer or 0 vs nonzero 23); most in `probes-numeric/m9-t-dist-2t.case.json` 88; `ironcalc-calc/STATISTICAL/T_DIST.xlsx` 10. e.g. `T.DIST.2T(1,2)` Excel 0.42264973081037416, ours 0.42264973081037294
- TDIST: 12 cells (13-14 digits right 9, 10-12 2, 6-9 0, fewer or 0 vs nonzero 1); most in `probes-numeric/t1048-tdist.case.json` 11; `probes/stat-argument-modes.case.json` 1. e.g. `TDIST(2,10,1)` Excel 0.0366940173853702, ours 0.03669401738537015
- T.TEST: 46 cells (13-14 digits right 37, 10-12 9, 6-9 0, fewer or 0 vs nonzero 0); most in `probes-numeric/t1048-ttest.case.json` 28; `probes-numeric/t1049-ttest.case.json` 9. e.g. `T.TEST(C1:C6,B1:B8,1,2)` Excel 0.07941923058217022, ours 0.07941923058217071
- TTEST: 29 cells (13-14 digits right 20, 10-12 9, 6-9 0, fewer or 0 vs nonzero 0); most in `probes-numeric/t1048-ttest.case.json` 28; `probes/stat-argument-modes.case.json` 1. e.g. `TTEST(C1:C6,B1:B8,1,2)` Excel 0.07941923058217022, ours 0.07941923058217071

Against a 50-digit reference, Excel is close to correctly rounded for this family and the engine is the inaccurate side, so an accurate kernel (not a fit) should reach this stage.
