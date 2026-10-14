---
id: T-0973
title: "GAMMA and GAMMALN residue: Excel's gamma-function kernel (39 known failures)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
GAMMA returns Excel's values (correctly rounded factorials at positive integers up to 25 (exact up to 23), Excel's last digits elsewhere, finite up to 171.6)
from one accurate gamma-function routine, and the 18 GAMMALN/GAMMALN.PRECISE cells T-0926 left
behind are matched by the same kernel or explained.

## Context
Kernel map (T-1522, decision 0056): rank 8 in `docs/research/numeric-kernel-map.md`; fit with the fit lab (T-1523).
Oracle sweep (T-1042): probe cases `t1042-gamma-int-1-90`, `t1042-gamma-int-91-171`, `t1042-gamma-half-int`, `t1042-gamma-unit-interval`, `t1042-gamma-negatives-edges` in `eval/suites/probes-numeric`; findings in `docs/excel-semantics/statistics.md` (GAMMA and GAMMALN sweep).
Triage T-0903. Kernels: GAMMA uses `statrs::function::gamma::gamma` (Lanczos, a few ulp to 1e-13
off, not exact at integers); GAMMALN uses `special::ln_gamma` = `libm::lgamma` (fdlibm, T-0926,
correctly rounded at the points it fixed). Code: `functions/statistical/gamma.rs` `fn_gamma`,
`fn_gamma_ln`; `functions/statistical/special.rs` `ln_gamma`. statrs `gamma` is also used inside
statrs' Beta density (T-0970 may drop that use).
39 baseline lines in `eval/baseline/ironcalc-calc.txt` (16 NumericClose, 23 NumericUlp):
- `ironcalc-calc/STATISTICAL/BETA_GAMMA.xlsx` `Sheet1`: B4(C), B6(C), B7(C), B8, B10(C), C10, D10, B11(C), C11, D11, B12(C), C12, D12, B13(C)
- `ironcalc-calc/STATISTICAL/GAMMA.xlsx` `GAMMA`: B2(C), B3(C), B4, B6(C), B7(C), B8(C), B9, B10(C), B11(C), B16, B20, B22(C), B28(C)
- `ironcalc-calc/STATISTICAL/GAMMALN_GAMMALNPRECISE.xlsx` `GAMMALN GAMMALN.PRECISE`: B5, C5, B9, C9, B15, C15, B23, C23, B24, C24, B26, C26
(C = NumericClose, unmarked = NumericUlp; GAMMA 21 lines, GAMMALN 9, GAMMALN.PRECISE 9.)

Examples (expected vs actual):
- `GAMMA(5)` GAMMA.xlsx B8: expected 24.0, actual 24.000000000000114; `GAMMA(4)` B7 expected 6.0, actual
  5.999999999999988; `GAMMA(1)` B4 expected 1.0, actual 0.9999999999999996; BETA_GAMMA B7 `GAMMA(6)`
  expected 120.0.
- `GAMMA(0.5)` B9: expected 1.772453850905516 (correctly rounded sqrt(pi)), actual 1.7724538509055159.
- `GAMMA(-0.5)` B10: expected -3.5449077018110318 (one ulp from the correctly rounded value), actual
  -3.5449077018110255.
- `GAMMA(1E-10)` B16: expected 9999999999.422787, actual 9999999999.422785.
- BETA_GAMMA B10: expected 9.332621544394421e+155, actual 9.332621544395446e+155 (1.1e-13).
- `GAMMALN(3)` GAMMALN B5: expected 0.6931471805599454, actual (lgamma) 0.6931471805599453;
  B23 expected -5.772155829918508e-08, actual -5.772155829918507e-08.
Quirks recorded in `docs/excel-semantics/statistics.md` ("GAMMA is exact at small positive integers
but not always correctly rounded" and "GAMMA and GAMMALN sweep (T-1042)") and `numbers.md` ("GAMMALN / GAMMALN.PRECISE (T-0926)").

Hypothesis (the oracle is the judge): correctly rounded product for positive integers up to 25 (the T-1042 sweep shows Excel deviates from the correctly rounded factorial from n = 26); otherwise a
high-accuracy routine (fdlibm/musl `tgamma`, or a Lanczos variant with reflection for x < 0.5).
GAMMALN(3) = 0.6931471805599454 is one ulp above the correctly rounded ln 2 (0.6931471805599453),
so Excel's GAMMALN is not correctly rounded either and is not plain `ln(GAMMA(3))` in doubles; test
whether one shared kernel (e.g. ln of the GAMMA routine's internal Lanczos sum, or the routine's own
log form) reproduces both families. If the cached cells cannot separate the candidates, stop and fit
against the T-1042 gamma-function sweep instead of guessing.

More cells (T-1289's gen-m9 suite, excel-graph, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/d244411dd90f7fbd Calc!A17 `=IF(..,GROWTH(GAMMA(Data!$E7)),..)` (E7 = 100): Excel 9.332621544394404e+155; the engine's `GAMMA(100)` is 9.332621544395446e+155, about 1e-13 relative off 99! = 9.33262154439441e+155 (the cell also needs T-1402 for GROWTH of one value).

**T-1524 (fit lab `gamma-int`, `gammaln-int`, `gamma-half-int`, 2026-10-14):** GAMMA(n) for integers n <= 88 is
the downward product `(n-1)*(n-2)*...*2` in doubles: it matches all of them, 92/171 integer cells in all
(engine 1, correctly rounded 40, upward product 38, 64-bit product 40). From n = 89 Excel switches to another
routine (up to 11 ulps from the factorial). GAMMALN at integers: ln of that product 82/171, correctly rounded
80, engine 108, so GAMMALN is a separate routine. GAMMA(m+1/2): best double recurrence 34/62, correctly
rounded and 64-bit 45, engine 0. The x87 64-bit-intermediate model only ties correctly rounded, so it is not
Excel's gamma. Details: statistics.md "GAMMA and GAMMALN sweep (T-1042)".

## Acceptance criteria
- [ ] All 16 GAMMA NumericClose lines leave `eval/baseline/ironcalc-calc.txt` (exact integers among
      them), no new lines in any baseline (GAMMALN lines T-0926 fixed must stay fixed).
- [ ] GAMMALN/GAMMALN.PRECISE: either the 18 lines leave the baseline, or the PR lists them with the
      formulations tried and this task's Outcome points at the T-1042 sweep follow-up.
- [ ] `GAMMA(171.5)` and `GAMMA(171.6)` are finite (9.483367566824801e+307, 1.5858969096672508e+308) and
      `GAMMA(171.7)` is `#NUM!` (T-1042, `t1042-gamma-negatives-edges`); the t1042 probe cases improve.
- [ ] No statrs `gamma::gamma` left in `functions/statistical/gamma.rs` (one gamma-function definition
      in `special.rs`); unit tests GAMMA(1..=6) exact, GAMMA(0.5), poles are #NUM!.
- [ ] `make check` green; reviewers approve.

## Out of scope
Incomplete gamma (T-0971); GAMMA.INV (T-0972).
