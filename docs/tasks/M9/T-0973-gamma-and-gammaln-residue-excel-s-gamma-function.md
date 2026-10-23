---
id: T-0973
title: "GAMMA and GAMMALN residue: Excel's gamma-function kernel (39 known failures)"
milestone: M9
lane: functions-stat
status: done
priority: P1
size: S
tier: opus
campaign: visible
depends_on: []
---

## Goal
GAMMA at positive integers n <= 88 returns Excel's values, which are the downward product
(n-1)*(n-2)*...*2 in doubles (T-1524), GAMMA is finite up to 171.6, and the n >= 89 integer routine is
either fitted or recorded as this task's open item. GAMMALN and GAMMA at non-integers are T-1529
(time-boxed).

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
routine (up to 32 ulps from the downward product and 29 from the factorial, at `GAMMA(167)`,
t1042-gamma-int-91-171 Calc!A77). GAMMALN at integers: ln of that product 82/171, correctly rounded
80, engine 108, so GAMMALN is a separate routine. GAMMA(m+1/2): best double recurrence 34/62, correctly
rounded and 64-bit 45, engine 0. The x87 64-bit-intermediate model only ties correctly rounded, so it is not
Excel's gamma. Details: statistics.md "GAMMA and GAMMALN sweep (T-1042)".

gen-m9 sweep (T-2255 triage), last digits only, not visibly off: GAMMALN.PRECISE last digits. Bit-level targets, outside the visible-parity stage (T-2255 triage, excel-graph, 2026-10-22): 6 known-failure lines in eval/baseline/gen-m9.txt, 6 formulas.
Examples:
- gen-m9/40425341c5034eed Calc!A115 `=GAMMALN.PRECISE((WRAPCOLS(1.005,2,7)^0.3))`: Excel -0.0008624698309035655, ours -0.0008624698309035657
Cells:
- gen-m9/40425341c5034eed.case.json (1): Calc!A115
- gen-m9/8f2ac25945af87da.case.json (1): Calc!A97
- gen-m9/b6593d7dd13f2dac.case.json (1): Calc!A124
- gen-m9/b8ee55934c5b824c.case.json (1): Calc!A153
- gen-m9/c468f5046aadfbbc.case.json (1): Calc!A294
- gen-m9/f2bd5f380ec002bf.case.json (1): Calc!A241

## Acceptance criteria
- [x] GAMMA(n) for integers 1 <= n <= 88 is the downward product (n-1)*(n-2)*...*2, each product rounded
      to a double: all 88 such cells of `probes-numeric/t1042-gamma-int-1-90` pass (92 of the 171 integer
      cells in the two t1042 integer cases at least; the fit lab's `gamma-int` scores this rule 92/171).
- [x] GAMMA(n) for integers n >= 89 (83 cells of `t1042-gamma-int-1-90` and `-91-171`): open item.
      Either a fitted routine passes them, or the PR leaves them in the baseline and lists the
      candidates tried in statistics.md "GAMMA and GAMMALN sweep (T-1042)".
- [x] `GAMMA(171.5)` and `GAMMA(171.6)` are finite (9.483367566824801e+307, 1.5858969096672508e+308) and
      `GAMMA(171.7)` is `#NUM!` (T-1042, `t1042-gamma-negatives-edges`).
- [x] No new lines in any `eval/baseline/` file: the GAMMALN lines T-0926 fixed stay fixed, and
      non-integer GAMMA cells do not regress (their fit is T-1529).
- [x] `make check` green; reviewers approve.

## Out of scope
GAMMALN and GAMMA at non-integers (T-1529, time-boxed); incomplete gamma (T-0971); GAMMA.INV (T-0972).

## Visible-parity stage
Decision 0083: this task is done when every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 must be exactly 0). Criteria above that ask for bit-exact or ulp-level matches and are still unmet then move to a follow-up task (no campaign key), named in the Outcome. Regressions that stay within 4 ulp follow decision 0083's allowance.

Visibly-off cells on the 2026-10-20 eval run (`main` 35a5e71), blamed on this function rather than on glue around it: 241 in total.
- GAMMA: 241 cells (13-14 digits right 200, 10-12 41, 6-9 0, fewer or 0 vs nonzero 0); most in `probes-numeric/t1042-gamma-int-1-90.case.json` 84; `probes-numeric/t1042-gamma-int-91-171.case.json` 75. e.g. `GAMMA(3)` Excel 2.0, ours 2.0000000000000067

Against a 50-digit reference, Excel is close to correctly rounded for this family and the engine is the inaccurate side, so an accurate kernel (not a fit) should reach this stage.

Integer arguments only (non-integers and the T-1941 model are T-2129). Excel is exact at small integers, so the downward product reaches 15 digits there; for 89..171 the stage-1 target is 15 digits, not bits.

## Update 2026-10-20: T-1941 identified Excel's algorithms
GAMMALN and GAMMA are mapped to Excel's own x87 routines (T-1941; `docs/research/t1941/HANDOFF2.md`). T-2128 and
T-2129 ship them; the four open kernels' remaining coefficient bits are T-2122. This supersedes the hypotheses above
(named library routines, Lanczos, ln of the product).

## Outcome
Closed with no engine change: T-2129 (GAMMA) and T-2128 (GAMMALN, GAMMALN.PRECISE) shipped Excel's own x87 routines
(T-1941), which meet every criterion here. Checked on `main` 47a2e53 with `rustsheet-eval show`:
`probes-numeric/t1042-gamma-int-1-90`, `t1042-gamma-int-91-171` and `t1042-gamma-half-int` have no failing cells (the
n <= 88 downward product and the n >= 89 item both pass); in `t1042-gamma-negatives-edges` GAMMA(171.5) and GAMMA(171.6)
are finite and GAMMA(171.7) is #NUM!; the two misses left there, GAMMA(-0.1) and GAMMA(-1.1), are an ulp through K1
(T-2122). No GAMMA, GAMMALN or GAMMALN.PRECISE cell of ironcalc-calc GAMMA.xlsx, BETA_GAMMA.xlsx or
GAMMALN_GAMMALNPRECISE.xlsx is in `eval/baseline/` any more (what is left there is GAMMA.DIST / GAMMA.INV / BETA.DIST,
T-0971, T-0972, T-0970). gen-m9 d244411dd90f7fbd A17 (GAMMA(100)) passes.

The six gen-m9 GAMMALN.PRECISE cells, each a few ulp, are not visibly off and now belong elsewhere. Evaluating the
argument on its own shows where each one falls:
- Five fall in K3 on [0.7, 1.5): 40425341c5034eed A115 (x = 1.0015), 8f2ac25945af87da A97 (0.817), b6593d7dd13f2dac
  A124 (0.999994), b8ee55934c5b824c A153 (0.871), f2bd5f380ec002bf A241 (0.845). K3's coefficients are the open
  kernel with a 0.32 exact rate. These cells move to T-2122.
- c468f5046aadfbbc A294 `GAMMALN.PRECISE(WEIBULL.DIST(..))`: the argument is 6.66e-99, so fl(x+1) = 1 and the
  routine returns -LN(x). Our value equals our `-LN(x)` bit for bit, and LN matches Excel's FYL2X model, so the 3-ulp
  miss comes from the WEIBULL.DIST density's last bits. This cell moves to T-0978.
Eval delta: none (no code change).
