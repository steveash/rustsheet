---
id: T-0926
title: "GAMMALN/GAMMALN.PRECISE: accurate ln-gamma (66 known failures)"
milestone: M9
lane: functions
status: todo
priority: P2
size: S
depends_on: []
---

## Goal
GAMMALN and GAMMALN.PRECISE (and the ln-gamma helper used elsewhere) return correctly rounded values, including exact 0 at x = 1 and x = 2.

## Context
Oracle-free: in `ironcalc-calc/STATISTICAL/GAMMALN_GAMMALNPRECISE.xlsx` Excel's values are the true ones:
`B2` `=GAMMALN(4)` expected 1.791759469228055 (ln 6), actual 1.7917594692280527; `B3` `=GAMMALN(1)` expected 0.0, actual -4.440892098500626e-16 (NumericMismatch, 5 cells); `B5` GAMMALN(3) expected 0.6931471805599454 actual 0.6931471805599485; `B6` GAMMALN(5) expected 3.1780538303479458 (ln 24).
GAMMALN 33 cells + GAMMALN.PRECISE 33 cells failing (15 ulp, 13 close, 5 mismatch each), 54 baseline lines in this file plus 28 in `BETA_GAMMA.xlsx` (shared inputs; its GAMMALN rows).
Code: `functions/statistical/gamma.rs` uses `statrs::function::gamma::ln_gamma` (Lanczos, ~1e-15 abs error, not exact at 1 and 2). Replace with a port of fdlibm/musl `lgamma_r` (exact zeros at 1 and 2, < 1 ulp), as a single shared helper; check `rg ln_gamma` for other users (GAMMA.DIST, BETA etc.) and switch them only if it does not add baseline failures.

## Acceptance criteria
- [ ] The GAMMALN and GAMMALN.PRECISE lines in `GAMMALN_GAMMALNPRECISE.xlsx` (up to 66) and the GAMMALN lines in `BETA_GAMMA.xlsx` disappear from `eval/baseline/ironcalc-calc.txt`; report any that remain with their expected/actual; no new lines.
- [ ] Unit test: ln_gamma(1) == 0, ln_gamma(2) == 0, ln_gamma(4) bit-equals ln(6).
- [ ] `make check` green.

## Out of scope
GAMMA function (Gamma(x) itself), incomplete gamma/beta kernels.
