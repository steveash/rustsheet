---
id: T-0925
title: "Inverse normal (Wichura AS241) for NORM.S.INV/NORM.INV/LOGNORM.INV/CONFIDENCE.NORM (~120 known failures)"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: S
tier: opus
depends_on: [T-1006, T-1746]
---

## Goal
NORM.S.INV, NORM.INV, LOGNORM.INV, CONFIDENCE.NORM (and the NORM-based part of CONFIDENCE.T inputs where applicable) return Excel's values, using one shared accurate inverse-normal routine.

## Context
Kernel map (T-1522, decision 0056): rank 11 in `docs/research/numeric-kernel-map.md`; fit with the fit lab (T-1523).
Oracle-free: Excel's cached values are the exact quantiles, ours (statrs `Normal::inverse_cdf`, which goes through a lower-accuracy erf_inv) are off by ~1e-11.
- `ironcalc-calc/STATISTICAL/NORM_DIST.xlsx` `NORM.S.DIST NORM.S.INV!D2` `=NORM.S.INV(C2)` expected 1.3333330000000008, actual 1.3333329999747054; `D4` expected 0.8000000000000002 actual 0.7999999999297425. NORM.S.INV 35 cells, NORM.INV 29 cells (incl. 2 NumericMismatch).
- `ironcalc-calc/STATISTICAL/LOGNORM.xlsx` `LOGNORM.DIST LOGNORM.INV!F2` expected 4.000000000000003 actual 4.000000000091908 (31 cells).
- `ironcalc-calc/STATISTICAL/CONFIDENCE.xlsx` CONFIDENCE.NORM 23 cells, all NumericUlp (`D2` expected 0.2771807648699355 actual 0.27718076486993554).
Code: `functions/statistical/{normal,log_normal}.rs`. Implement AS241 (PPND16) once, in the same module as the T-0924 kernel if that has landed (else a new `normal_kernel` module that T-0924 extends), and route all four functions through it. Where AS241 alone leaves ulp-level diffs, test whether one Newton/Halley refinement step against the T-0924 CDF reproduces the cached cells and keep what matches.

**Needs an oracle sweep (T-0018):** fitting this formulation needs more Excel points than the
cached workbooks give. Fit against the T-1006 sweep (`probes-numeric/m9-norm-s-inv`), never blind.

Triage T-0903 (2026-10-06 report): this task owns 106 baseline lines: NORM.S.INV 33, NORM.INV 19,
LOGNORM.INV 31 (all NORM_DIST.xlsx / LOGNORM.xlsx `*.INV` cells) and CONFIDENCE.NORM 23. The normal
CDF it inverts is not correctly rounded (`docs/excel-semantics/statistics.md`); its last digits are
T-0968, so an iteration against Excel's CDF needs T-0968 first.

## Acceptance criteria
- [ ] All NumericClose/NumericMismatch lines for NORM.S.INV, NORM.INV, LOGNORM.INV disappear from `eval/baseline/ironcalc-calc.txt`; the CONFIDENCE.NORM ulp lines disappear or the PR documents which remain and why. Exact before/after counts in the PR; no new lines.
- [ ] No other call sites of statrs `inverse_cdf` for Normal (rg).
- [ ] Unit tests incl. p near 0/1 and p = 0.5; `make check` green.

## Out of scope
erf/CDF accuracy (T-0924); T.INV/GAMMA.INV/BETA.INV iterations.

## Fit result (T-1746)
Implement PPND16 (Horner; log(1-p) on the upper side) for |p-0.5| > 0.425: it is bit-exact on 611 of 616 tail cells. Centrally, keep statrs: nothing fitted there, and statrs is the best candidate. The hybrid is bit-exact on 775 of 1,024 sweep cells, against 341 today (`uv run rustsheet-fitlab norm-s-inv`; docs/excel-semantics/statistics.md, "NORM.S.INV is AS 241 PPND16 in the tails").

## Earlier attempt (draft PR #38, 2026-10-04; salvaged by T-0031)
Plain AS241 (PPND16) fixed 57 of ~120 cells (ironcalc-calc known failures 1632 -> 1580) but regressed
5 cells that statrs matched by luck: `CONFIDENCE.xlsx CONFIDENCE.NORM CONFIDENCE.T!D8`,
`LOGNORM.xlsx LOGNORM.INV!F72`, `NORM_DIST.xlsx NORM.INV!F28`, `NORM.S.INV!D11` (expected
-2.0000000000000004) and `D13` (expected exactly -3.0; AS241 gives -2.999999999999999). One Newton or
Halley step against the T-0924 CDF was worse (fixed 38, 9 regressions). Excel's last digits (e.g.
`NORM.S.INV(C2)` -> 1.3333330000000008) look like iteration against Excel's own CDF, so parity likely
needs that CDF bit-for-bit (several NORM.S.DIST cells, e.g. `NORM.S.INV!C3`, are still NumericUlp)
plus oracle probes. The oracle now exists: probe NORM.S.INV at p = NORM.S.DIST(k) for integer k first.
