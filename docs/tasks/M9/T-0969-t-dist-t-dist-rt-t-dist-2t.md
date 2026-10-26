---
id: T-0969
title: "T.DIST/T.DIST.RT/T.DIST.2T/T.TEST: accurate Student-t CDF with direct upper tail (55 known failures)"
milestone: M9
lane: functions-stat
status: blocked
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

## Attempt 6 (2026-10-20, opus run; findings only, merged as progress)
- **The visible-parity premise below is wrong for what is left.** On `main` a15f80ae the family has 48
  visibly-off cells (T.DIST 28, T.DIST.2T 14, T.DIST.RT 5, TDIST 1; T.TEST/TTEST 0). Against mpmath at 60 digits, on the
  46 literal-argument ones the engine is within 1.7 ulp of the exact value and Excel 4.6 to 482 ulp off on 41
  (`T.DIST(1E6,30,FALSE)` +482, `T.DIST.RT(30,1E6)` +398, `T.DIST(1000,100,FALSE)` -165, `T.DIST.2T(3,100)` +23,
  `T.DIST(2,3,FALSE)` +4.6). The engine is the off side on 5: `T.DIST.RT(1E6,2)` (engine -4.5, Excel 0.5),
  `T.DIST.2T(2,9999999999)` (12.3 against 1.3), and df 1E6 at t = 10 and 30 (`T.DIST.RT(10,1E6)` -13.6 against -37.6,
  `T.DIST(30,1E6,FALSE)` +148 against -200). An accurate kernel cannot reach the stage: the 41 need Excel's noise reproduced.
- **Density at df = 2 is cracked:** Excel's `T.DIST(t,2,FALSE)` is `EXP(-1.5*LN(2+t^2))` with Excel's x87 EXP and LN
  (`x87_exp.rs`, `x87_log10.rs`) or libm's: bit-exact on all 13 df-2 density cells, including `T.DIST(1E6,2,FALSE)`, which is
  15 ulp from the exact value. `POWER(2+t^2,-1.5)` is 9/13, the ln(1+t^2/2) form 11/13.
- **The density noise is that of one exp of a double exponent, but not of any single double for df >= 3.** Excel's
  effective exponent (ln of Excel's value over the exact density, plus the exact exponent) is within about one ulp
  of the exact -((n+1)/2) ln(1+t^2/n) + ln K on every cell from df 2 to 1E6. A scan over the constant C (6,001
  doubles around ln K) for `exp(C - h ln(n+t^2))`, `exp(C) exp(-h ln(n+t^2))` and `exp(C - h ln(1+t^2/n))` (h = (n+1)/2)
  never reproduces more than 5 of a df's 8 to 16 cells, so for df >= 3 the exponent is not one double C plus one of these
  terms. Also rejected (bit-exact out of 121 density cells; the engine has 38): every ordering of the five-term exponent
  a ln n + GAMMALN(h) - GAMMALN(n/2) - ln sqrt(pi) - h ln(n+t^2) and its four regroupings with Excel's GAMMALN (T-1941),
  at most 22 of the 87 df-2-to-100 cells; GAMMA(h)/GAMMA(n/2) from Excel's GAMMA, EXP(GAMMALN differences), 1/(sqrt(n) B) and
  K POWER(n+t^2,-h) with K correctly rounded, at most 17.
- **The centre tail sometimes follows the rounded y, as T.TEST does.** For t^2 < n, I_y(n/2,1/2)/2 at y = n/(n+t^2)
  rounded to a double (`student_t_tail_at`) is bit-exact where the engine's exact tail is not at
  `T.DIST.2T(0.1,2)` (engine -8 ulp), (2,100) (-12 -> -1), (3,100) (-23 -> 1), (3,30) (-8 -> -2), (2,30) and (5,30), but is
  worse at small t (`T.DIST.RT(0.5,100)` 1 -> 29, (0.5,1000) -1 -> 97) and catastrophic from df 1E6 (y's rounding is n eps).
  No df/t cut separates the two on the swept cells (df 100 t = 5 and df 1000 t = 3 prefer the exact tail). For df 2,
  0.5 - 0.5 sqrt(1 - fl(2/(2+t^2))) is bit-exact at t = 0.1 and 1.
- Cell table: `docs/research/t0969/tcells.tsv` (README there). The acceptance criteria stand until the probes below; re-cut them per decisions 0056/0083 only if those find no model. Next: probe Excel at df 3 and 4 densities
  on a dense t grid (and the same t at df 2 as a control) to find the df >= 3 formula; and probe the centre tail on a dense
  (df, t) grid to locate where Excel switches between the rounded-y and the t form.

## Attempt 7 (2026-10-20, opus run; merged as progress)
Two new excel-graph probes (probes-numeric): `t0969-t-pdf-runs`, 14,914 densities on runs of consecutive doubles and a
coarse grid for df 2 to 10; and `t0969-t-tail-grid`, 3,232 T.DIST.RT cells on a (df, t) grid plus three runs.
- **Centre tail: cracked where it switches.** Excel's tail is `I_y(n/2,1/2)/2` at y = fl(n / fl(n + fl(t*t))) (the
  T.TEST kernel, `student_t_tail_at`) once t > 1 and y <= 0.99, and the exact tail at t otherwise. On the runs, a
  regression on the intermediates' rounding errors gives -1 on fl(n + t^2) and +1 on fl(n/.) (df 100). The grid
  brackets the switch at t = 1 for df <= 100 and at t^2 = n/99 to n/100 above. statistics.md has the entry
  "T.DIST.RT follows the rounded y beyond t = 1".
  - `unflushed_upper_tail` now takes that path.
  - The grid goes from 325 to 548 bit-exact cells of 3,232, and from 1,229 to 2,210 within 4 ulp.
  - Eval delta: no baseline line changed and no regressions; visibly-off in-scope cells 1774 -> 1772.
    `T_DIST.xlsx` F7, F8, C7, C8 (13 ulp) are now NumericUlp, 0 to 2 ulp, so no T.DIST/T.DIST.RT/T.DIST.2T/T.TEST
    line in `T_DIST.xlsx` or `T_TEST.xlsx` is NumericClose or NumericMismatch any more (only T.INV/T.INV.2T ones,
    T-0967). probes-numeric bit-exact: m9-t-dist-cdf 122 -> 124, m9-t-dist-rt 75 -> 77, m9-t-dist-2t 52 -> 53.
    Eleven more m9-t-dist cells move from NumericClose to NumericUlp, among them (2,100), (3,100), (3,30) and
    (5,30), and so do t1048-tdist A7/B7/A8/B8. One cell gets worse: `T.DIST.RT(5,100)` and `T.DIST.2T(5,100)` go from 2 to 15 ulp,
    still within 15 digits.
- **df 15 is an outlier:** Excel is 6 to 27 ulp off both models, with a sign flip at t = 1, as from a Gamma ratio
  about 1.5E-15 relative off at a = 7.5. Not modelled.
- **Density: structure found, kernel not cracked.**
  - df 2 is confirmed: `EXP(-1.5*LN(2+t^2))`, 1,024 of 1,024 cells.
  - For df >= 3, t*t is rounded but n + t^2 (or 1 + t^2/n) is not.
  - The residual noise is 1 to 5 x 2^-53 relative rms, growing with |exponent|.
  - None of these fit more than about 60% of a df's cells: D * EXP(RN(E)), EXP(RN(C + E)), D * exp(RN(E)) with exp
    unrounded, or D * exp(-h RN(ln u)), each for the u = 1 + t^2/n and n + t^2 forms and any D or C. In the
    D * exp(-h RN(ln u)) model the residuals spread over +-4 ulp.
  - Next: a model where the stored exponent carries a constant ln-Gamma term computed in extended precision, and a
    probe of `T.DIST(t,n,FALSE)` at t with t*t exact (t = k/2^m) to separate the exponent's rounding from that of t^2.

## Attempt 8 (2026-10-21, opus run; merged as progress)
- **df 2 cracked for y >= 0.1.** Excel's tail at df 2 is BETA.DIST's a = 1 closed form, (1 - exp(ln(1 - y)/2))/2, at
  y = fl(2/fl(2 + fl(t*t))), inside t = 1 too, down to y = 0.1 (t = 4.2426); below it Excel is accurate and no simple
  form fits (statistics.md "T.DIST.RT at df 2 is BETA.DIST's closed form down to y = 0.1"). New probe
  `t0969-t-df2-cut` (801 cells, t = 2 to 6): the closed form is 449 of 449 above the cut, 12 of 352 below.
  `student_t.rs` `closed_form_tail` serves both the t path and T.TEST's y path.
- Eval delta: on the cases that existed before, 41 cells newly bit-exact and 0 regressions. Of those, `T_DIST.xlsx` F16 and C16
  left `ironcalc-calc.txt`, 30 are `t0969-t-tail-grid` cells, and 2 are T.TEST cells in t1049-ttest. Also `T.DIST.RT(0.0013,2)` went from 56 ulp to 1.
  Family visibly-off cells on those cases: 491 -> 487. New-case cells: df2-cut 560 of 801 bit-exact, 0 visibly off.
  df-ratio has 9 visibly off, the df-15 tails among them.
- **Per-df prefactor (not modelled).** New probe `t0969-t-df-ratio` (df 1 to 200 and 250 to 1000 by 50, tails at t =
  0.5, 1.5 and 2, densities at 0.5 and 2): df 15's tails are all -20 x 2^-53 relative off, at every t, while its
  densities are exact. A handful of other df (7, 13, 16, 26, 147) agree within 0.5 x 2^-53 at t = 0.5 and 2 on errors of 4 to 7.
  exp of GAMMALN differences with Excel's own GAMMALN reproduces df 13, 15 and 16 but predicts errors of 25 to 215 at df 17, 18, 20, 28
  and 147, where Excel has under 4. Excel's GAMMA ratios and `GAMMA(7.5)` are correctly rounded. One scaled ratio
  fits only 24 of df 15's 120 cells (statistics.md "Student t prefactor per df").
- **Density.** Sheet Exact of the same probe has t*t and n + t*t exact (df 3, 4, 5, 7). Every K * f(u) model, and the
  correctly rounded density with a fitted K, still tops out near 45%. So the noise is inside Excel's kernel, not the
  inputs' rounding. R's `dt` (Loader's bd0/stirlerr) is rejected on t0969-t-pdf-runs: 363 of 1,024 at df 2
  (statistics.md "Student t density noise is not input rounding").
- Next: the 260 visibly-off tail-grid cells are df 15 (about 60) and Excel's incomplete-beta noise at df 50 to 1000
  for t = 2.4 to 5.5 (the rounded-y region, 3 to 24 ulp on values of 1E-2 to 1E-7). Both need Excel's
  incomplete-beta algorithm (T-1744, T-1759: its direct branch is the unknown). The density needs its power/exp kernel. Candidate probes:
  `BETA.DIST(y,n/2,0.5,TRUE)` at the same rounded y as `T.DIST.RT(t,n)`, to see whether T.DIST.RT calls BETA.DIST's
  kernel (df 15 included); and BETA.DIST at a = 7.5, b = 0.5 to see whether the df-15 prefactor is BETA.DIST's.

## Attempt 9 (2026-10-21, opus run; findings only, merged as progress)
- **T.DIST.RT is BETA.DIST, switched at the mean.** New probe `t0969-t-beta-link` (184 (t, df) pairs, df 1 to 1000):
  with the worksheet's y = fl(n/fl(n + fl(t*t))) and z = fl(fl(t*t)/fl(n + fl(t*t))), `T.DIST.RT(t,n)` is
  `BETA.DIST(y,n/2,0.5,TRUE)/2` to the bit on 136 of 155 cells beyond t = 1 (y <= 0.99; the rest 1 to 4 ulp, df 7 to 30
  at t = 1.5 to 3), and `0.5 - BETA.DIST(z,0.5,n/2,TRUE)/2` on 22 of 23 cells at t = 0.5. t = 1 is y = n/(n+1), the mean of
  Beta(n/2,1/2), so attempt 7's switch reads as the incomplete beta's side choice (an interpretation) (statistics.md "T.DIST.RT is BETA.DIST's
  incomplete beta, switched at the mean").
- Consequence: the family's remaining tail noise (the 260 visibly-off `t0969-t-tail-grid` cells: df 15 about 60,
  df 50 to 1000 for t = 2.4 to 5.5 about 175) is Excel's BETA.DIST noise. Against the exact I at the rounded y the
  grid's rounded-y region is 373 of 2,853 bit-exact and 2,261 within 4 ulp; the engine is 409 and 2,320. No more
  accurate Student-t kernel closes it: the stage needs Excel's incomplete-beta algorithm (T-1744, T-1759).
- **df 15 is B(1/2, 7.5).** New probe `t0969-beta-density-b`: `BETA.DIST(x,0.5,b,FALSE)` at small x is within 8.1 x
  2^-53 of exact for b = 0.25 to 40 by 0.25 except b = 7.5 (-18, -20) and 7.75 (-37, -39); a = 1.5 is clean at every
  b. So Excel's Beta function has a method switch near 8 that is off just below it for a = 1/2 (statistics.md
  "Student t prefactor per df"). Probe `t0969-beta-prefactor` (BETA.DIST around a = 7.5) is too noisy to isolate it.
  Correctly rounded Gammas, Gamma(a+b)/Gamma(a)/Gamma(b), match Excel's per-df prefactor error within 3 x 2^-53 at 174
  of 200 df (df-ratio probe, averaged over its three tails); exp of ln-Gamma differences at 15.
- The density is not BETA.DIST's density times the Jacobian (at most 20 of 184 bit-exact in five orderings).
- Eval delta: no engine change; no baseline line moved. The three new probes-numeric cases (ungated) add family
  cells: t0969-t-beta-link has 42 visibly-off T.DIST/T.DIST.RT cells (C 15, J 12, L 15).
- Next: (1) model Excel's B(a, 1/2) switch near 8 with a dense `BETA.DIST(x,0.5,b,FALSE)` probe for b in [7, 8.5] by
  1/64 and a = 0.25 to 2 (the df-15 fix: about 60 grid cells); (2) the rounded-y tails need Excel's incomplete beta,
  so route the tail through one beta kernel once T-2119 moves BETA.DIST's cdf onto `incomplete_beta`, and score it
  on t0969-t-beta-link column E; (3) in the y > 0.99, t > 1 region (df 500, 1000) t = 1.5 takes the centre form but t = 2 to 3 match neither.

## Attempt 10 (2026-10-21, opus run; merged as progress)
- **Excel's Beta function, cracked for a < 1 and b <= 8.** New probe `t0969-beta-switch`:
  B(a,b) = EXP(GAMMALN(a) + (GAMMALN(b) - GAMMALN(a+b))), with Excel's own GAMMALN. It predicts 140 of
  the 147 a < 1, b <= 8 density rows within 3 x 2^-53; past b = 8 (GAMMALN's Stirling switch) Excel's B is
  accurate. This is the df-15 prefactor: B(7.5, 1/2) (statistics.md "Student t prefactor per df").
- `student_t.rs` `log_gamma_ratio` scales the integer-df series tail by Excel's 1/B over the exact one for
  df 13 to 16. Below df 13 the same scaling loses cells (t1049-ttest 34 -> 26, m9-t-dist-rt 80 -> 74), so b < 6.5 is
  not yet understood. Neither is the ratio inside `gamma_ratio` (fractional df).
- Eval delta: no gated baseline line moved and none regressed. probes-numeric bit-exact: tail-grid
  578 -> 587, beta-link 623 -> 625, others unchanged. Visibly-off: tail-grid 260 -> 245, df-ratio 9 -> 8,
  beta-link 42 -> 40.
- Next: probe B(1/2, b) for b = 0.5 to 6.5 to find where the GAMMALN form starts; the remaining visibly-off
  tails (df 50 to 1000) are still Excel's incomplete-beta noise (attempt 9).

## Attempt 11 (2026-10-21, opus run; findings only, merged as progress)
- **Excel's B(1/2, b) is the GAMMALN form at every b up to 8, not only from 6.5.** On the existing
  `t0969-beta-density-b` (b = 0.25 to 8 by 0.25), `EXP(GAMMALN(1/2) + (GAMMALN(b) - GAMMALN(b + 1/2)))` predicts Excel's
  density error within 2 x 2^-53 at 29 of 32 b (x = 0.05). For a = 1.5 it does not hold. Its error at df 11 and 12 is
  -4 to -7 x 2^-53, at most a few ulp, so the cells attempt 10 lost below df 13 come from the tail kernel's own noise,
  not from B.
- **TOMS 708, tested and rejected as the tail kernel.** The B form's branches (a0 < 1, b0 < 8, then `algdiv`) are
  those of TOMS 708's `betaln`. R's `toms708.c` `bratio`, compiled standalone, is bit-exact on 27 of 176
  `t0969-t-beta-link` column-E cells, against 36 for the exact I_y, and is no better in any branch. It is closer than
  the exact value on the centre columns H and I: 122 and 131 of 176, against 102 and 115 (statistics.md "Student t
  prefactor per df").
- **Rejected:** the GAMMALN sum in BETA.DIST's argument order (a = n/2 first) for df 13 to 16. On probes-numeric,
  probes and ironcalc-calc it fixes 8 cells and breaks 13, all in the family's own probes.
- Eval delta: no engine change, no baseline line moved.
- Next: (1) score bratio with Excel's GAMMALN as `gamln` (needs `excel_gammaln` callable from the harness) on
  columns E, H and I; (2) bratio's `bgrat`/`bup` against the df 50 to 1000 rounded-y cells. That is the region of the
  tail grid's 175 noise cells, where TOMS 708's asymptotic expansion for large a would run.

## Attempt 12 (2026-10-22, opus run; findings only)
- Eval delta: no engine change, no baseline line moved. On `main` (this run's `make eval`) the family has 522
  visibly-off cells, all in ungated probes-numeric cases: `t0969-t-tail-grid` 245, `t0969-t-pdf-runs` 186,
  `t0969-t-beta-link` 40, `m9-t-dist-pdf` 26, and 25 in six other cases. None is in a gated suite.
- **TOMS 708 with Excel's GAMMALN, rejected.** Attempt 11's next step (1): R's `toms708.c` `bratio`, compiled
  standalone, with `gamln` looked up from `excel_gammaln` (no miss), is bit-exact on 29 of 184 `t0969-t-beta-link`
  column-E cells, the same as with its own `gamln`: `betaln` is never reached for (n/2, 1/2). H and I: 129 and 136.
- **Rejected on column E** (exact I_y: 39 bit-exact, 145 within 4 ulp):
  - Cephes `incbet` with Excel's B, prefactor exp-log, pow or one exp: 35 to 36.
  - Cephes `incbet` with the exact B: 31.
  - NR `betai`: 25 or 26.
- **Rejected on 700 rounded-y tail-grid cells** (exact: 92 bit-exact, 589 within 4 ulp, 53 visibly off): Cephes
  exp-log 52, Cephes pow 60, each about 175 visibly off.
- **Finding:** Excel's tail error grows about linearly with |(n/2) ln y|. Its standard deviation is 3 x 2^-53 at
  |E| < 2 and 24 at |E| >= 20, with no bias. (n/2) times the rounding error of ln y explains part of it
  (correlation 0.38), whether ln y is Excel's LN or libm's. No double form of E tried
  explains more (statistics.md "T.DIST.RT's noise beyond t = 1 grows with a ln y").

## Blocked because: time-boxed (decision 0056)
Attempts 11 and 12 did not raise the bit-exact count, which is decision 0056's bar. What is left is Excel's own
noise. Every one of the 522 visibly-off cells is in an ungated probes-numeric case, and the engine's kernel is
within a few ulp of the exact value there. Excel is not. Reaching the visible-parity stage needs Excel's algorithm
reproduced, not more accuracy. Candidates tried, with their counts:
- tails: attempts 2 to 12, ending in the list in statistics.md "T.DIST.RT's noise beyond t = 1 grows with a ln y";
- B: "Student t prefactor per df";
- densities: "Student t density noise is not input rounding" and attempts 6 to 8.
Reopen when a new hypothesis is recorded here, for example:
- a reconstruction of Excel's BETA.DIST cdf kernel that reproduces column E of `t0969-t-beta-link`;
- the second component of the a ln y noise;
- a density kernel for df >= 3.
A dependency landing (T-2119 included) does not reopen it.

Update 2026-10-24 (T-2513): the density hypothesis is found, so the density no longer blocks this task. For df >= 3,
T.DIST(t,n,FALSE) is Loader's `dt` in Excel's x87 pieces (log branch at x² > n, bd0 below it, E stored, the
denominator fl(fl(EXP(E)/fl(sqrt(w)))·fl(1/sqrt(2π)))); df 1 is 1/(π(1 + t·t)) and df 2 EXP(-1.5·LN(2 + t·t))
(statistics.md "Student t density is Loader's dt (T-2513)"). With the exact Loader constant it is within 15 digits on
977 of the verifier's 1,000 hold-out cells (CR 787) and on 18,631 of cells.tsv's 18,699 df >= 3 cells (CR 18,472); the
per-df constant is still ±1-2 units off (T-2586). T-2585 ships it. The tails are T-2464's kernel call (statistics.md
"Student t tails are one kernel call", T-2509). This task stays blocked until T-2585 and T-2509 land; then rescore
the cells above and the B prefactor ("Student t prefactor per df") against them.

T-2256 sweep (gen-m9 part 2, statistical M to Z; gap triage 2026-10-23): TDIST, T.DIST.2T, TTEST and T.TEST values off Excel's (the Student t tail and T.TEST's rounded-y path this task fits), from the last digit to the 13th (`T.DIST.2T(ODD(7),1E+10)` Excel 2.5596252476292973E-12, ours 2.5596252476292783E-12; `TDIST(DCOUNTA(..),6,2)` is T-1390's). x of 1E+300 is T-2474's.

Owned gen-m9 cells (T-2256 triage, excel-graph, 2026-10-23): 247 known-failure lines in eval/baseline/gen-m9.txt, 19 visibly off.
Examples:
- gen-m9/17c110a53924beb8 Calc!A159 `=T.DIST.2T(ODD(7),1E+10)`: Excel 2.5596252476292973e-12, ours 2.5596252476292783e-12
- gen-m9/928d08e3e1eecb5f Calc!A393 `=T.TEST(Data!B3:C11,Data!B4:E6,2,2)`: Excel 0.7772031601919551, ours 0.7772031601919531
- gen-m9/9bc247733a1ca9fa Calc!A139 `=TTEST(Data!B9:E9,Data!A4:D7,1,3)`: Excel 0.08451014632899068, ours 0.08451014632899077
- gen-m9/a5342a7e24fd803e Calc!A230 `=TDIST(EOMONTH(24332,-7),2,Data!A6)`: Excel 8.581583593277439e-10, ours 8.581583593277429e-10
- gen-m9/afc00c8b70e78baf Calc!A238 `=T.TEST(Data!B8:E12,Data!A8:B8,2,2)`: Excel 0.725939456341534, ours 0.7259394563415346
Cells:
- gen-m9/0072265d385d8ab2.case.json (3): Calc!A60, A63, A184
- gen-m9/0620505280d2ea6e.case.json (1): Calc!A34
- gen-m9/097af1ca33638670.case.json (1): Calc!A228
- gen-m9/09b47629f54b9817.case.json (1): Calc!A144
- gen-m9/0bbab6206b5c9a07.case.json (8): Calc!A23, A65, A82, A85, A99, A161, A209, A313
- gen-m9/11eb3ab4f4f8104c.case.json (4): Calc!A174, A272, A321, A360
- gen-m9/139684e68fb2fc3b.case.json (5): Calc!A63, A139, A177, A215, A224
- gen-m9/17c110a53924beb8.case.json (2): Calc!A71, A159
- gen-m9/1b31092a0d79e5f1.case.json (7): Calc!A34, A96, A111, A158, A340, A344, A359
- gen-m9/1db18890916ee643.case.json (1): Calc!A133
- gen-m9/201d67f1d59436ed.case.json (4): Calc!A34, A72, A82, A282
- gen-m9/2138b909d8f810ea.case.json (2): Calc!A161, A351
- gen-m9/31f09e6c48ca245c.case.json (2): Calc!A107, A173
- gen-m9/334f3dc9ca937f10.case.json (3): Calc!A308, A324, A357
- gen-m9/38e4fc309fe199ea.case.json (1): Calc!A181
- gen-m9/3fb9cecf2c23169f.case.json (2): Calc!A34, A344
- gen-m9/41248d1cb153c66e.case.json (3): Calc!A22, A235, A362
- gen-m9/43c6227e0ff368c7.case.json (3): Calc!A204, A306, A310
- gen-m9/44a8841352dc48e9.case.json (3): Calc!A44, A196, A289
- gen-m9/46ac3ef5e61583ca.case.json (6): Calc!A50, A96, A100, A224, A236, A298
- gen-m9/5389dd4d5b5739d2.case.json (4): Calc!A79, A92, A278, A302
- gen-m9/5395b8a427764a3f.case.json (4): Calc!A1, A35, A125, A373
- gen-m9/583ca2baea0bb30a.case.json (5): Calc!A75, A161, A199, A203, A285
- gen-m9/5919660b7bb4d195.case.json (3): Calc!A38, A70, A298
- gen-m9/5be29fa316931f05.case.json (5): Calc!A31, A114, A176, A279, A300
- gen-m9/5cebac1f02bf30f2.case.json (3): Calc!A7, A83, A337
- gen-m9/607e85cfee83a689.case.json (3): Calc!A53, A64, A115
- gen-m9/6e18550e9fc9c1b9.case.json (3): Calc!A104, A290, A319
- gen-m9/73f053591a54bc43.case.json (11): Calc!A3, A20, A88, A105, A122, A127, A173, A195, A207, A224, A343
- gen-m9/775d053de498b8c7.case.json (2): Calc!A91, A274
- gen-m9/787ba8d801105a60.case.json (6): Calc!A29, A52, A91, A153, A240, A277
- gen-m9/7980ffb6095a340d.case.json (1): Calc!A102
- gen-m9/7fd0a98183c6babf.case.json (4): Calc!A2, A138, A192, A230
- gen-m9/80586b859b73c87d.case.json (4): Calc!A164, A283, A317, A351
- gen-m9/873f30d2bf76f8db.case.json (2): Calc!A37, A151
- gen-m9/91e167cd8d7eb652.case.json (2): Calc!A8, A292
- gen-m9/928d08e3e1eecb5f.case.json (5): Calc!A107, A160, A169, A231, A393
- gen-m9/92d4fa29e5510f1e.case.json (1): Calc!A157
- gen-m9/941c409532603d32.case.json (3): Calc!A7, A13, A329
- gen-m9/9735c5ab72fa5ecf.case.json (2): Calc!A185, A298
- gen-m9/9bc247733a1ca9fa.case.json (4): Calc!A24, A139, A196, A254
- gen-m9/9e2225ead8a6a07b.case.json (4): Calc!A143, A267, A329, A387
- gen-m9/a2262ea466fcc142.case.json (4): Calc!A120, A197, A306, A368
- gen-m9/a4090567758520df.case.json (7): Calc!A63, A100, A115, A239, A286, A311, A373
- gen-m9/a5342a7e24fd803e.case.json (3): Calc!A53, A192, A230
- gen-m9/a63d1c7d6f96c56d.case.json (4): Calc!A150, A157, A274, A344
- gen-m9/acac71a65a5a1d1b.case.json (7): Calc!A6, A8, A23, A57, A176, A363, A365
- gen-m9/afc00c8b70e78baf.case.json (2): Calc!A85, A238
- gen-m9/b9a32ce9a1fd0d71.case.json (1): Calc!A304
- gen-m9/bbab6562e88b70e8.case.json (6): Calc!A83, A168, A181, A266, A304, A355
- gen-m9/bcb1105c61a2649d.case.json (8): Calc!A23, A25, A209, A226, A273, A335, A350, A400
- gen-m9/c2684066d8df263e.case.json (7): Calc!A58, A59, A152, A276, A307, A368, A400
- gen-m9/c4635963fc926284.case.json (3): Calc!A180, A199, A275
- gen-m9/c7664e494e418395.case.json (5): Calc!A12, A126, A219, A316, A392
- gen-m9/cb61169f4c033d4b.case.json (2): Calc!A4, A346
- gen-m9/ce312817dc9c7f0b.case.json (7): Calc!A90, A190, A204, A228, A318, A356, A394
- gen-m9/d208bbafc3bc1997.case.json (4): Calc!A108, A146, A260, A298
- gen-m9/d37313406b9f6e55.case.json (2): Calc!A170, A360
- gen-m9/d5fd9e05e9e6f466.case.json (4): Calc!A77, A166, A246, A250
- gen-m9/d8f44d556674c2fd.case.json (5): Calc!A20, A58, A139, A248, A286
- gen-m9/d910f000b60e2eac.case.json (5): Calc!A48, A106, A217, A234, A358 (A358 `TDIST(DCOUNTA(..),6,2)` = `TDIST(3,6,2)`, 1 ulp; moved from T-2524)
- gen-m9/eedd629f3d69fdda.case.json (11): Calc!A23, A28, A108, A176, A193, A210, A227, A244, A278, A295, A329
- gen-m9/f82473275380882e.case.json (8): Calc!A60, A142, A178, A199, A204, A308, A328, A390

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2256): these 247 known-failure lines (the T-2256 cells in Context) disappear from
      eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
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
