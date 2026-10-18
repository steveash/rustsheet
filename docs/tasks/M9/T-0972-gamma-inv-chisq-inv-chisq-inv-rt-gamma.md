---
id: T-0972
title: "GAMMA.INV/CHISQ.INV/CHISQ.INV.RT: gamma quantile parity (39 known failures)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-0971, T-1045]
---

## Goal
GAMMA.INV, CHISQ.INV and CHISQ.INV.RT return Excel's values from one gamma-quantile routine built
on the T-0971 incomplete-gamma kernel.

## Context
Kernel map (T-1522, decision 0056): rank 10 in `docs/research/numeric-kernel-map.md`; fit with the fit lab (T-1523).
Oracle targets: probes-numeric cases `m9-gamma-dist-a*`, `m9-chisq-dist`, `m9-chisq-dist-rt`, `m9-gamma-inv`, `m9-chisq-inv`, `m9-chisq-inv-rt` (T-1045).
Triage T-0903. Kernel: statrs' default `ContinuousCDF::inverse_cdf` for `Gamma` (bisection from
[1,2] widened by halving/doubling, 8 bisection steps, then Newton on statrs' CDF), reached through
`functions/statistical/gamma.rs` `fn_gamma_inv` and `functions/statistical/chisq.rs` `fn_chisq_inv`,
`fn_chisq_inv_rt` (which inverts `1.0 - p`).
39 baseline lines in `eval/baseline/ironcalc-calc.txt` (11 NumericClose, 28 NumericUlp):
- `ironcalc-calc/STATISTICAL/CHISQ.xlsx` `Sheet1`: I2, J2
- `ironcalc-calc/STATISTICAL/GAMMA.xlsx` `GAMMA.DIST GAMMA.INV`: F2, F3, F4, F5, F6, F7, F8(C), F9, F10(C), F11, F19, F21, F22(C), F24, F34, F39, F41, F42, F43, F44, F45, F46, F47, F50, F51, F53, F61, F62, F66(C), F67(C), F68(C), F69(C), F70(C), F74(C), F75, F76(C), F77(C)
(C = NumericClose, unmarked = NumericUlp.)

Examples (expected vs actual):
- `GAMMA.INV(GAMMA.DIST(10,9,2,TRUE),9,2)` F2 (14 lines with this pair): expected 9.999999999999998, actual
  9.999999999999993. Excel does not round-trip to 10 (`docs/excel-semantics/statistics.md`).
- F74: expected 61.91422671617931, actual 61.91422671628383 (1.7e-12); F77: expected 1999.3333728539303,
  actual 1999.3333728539678.
- `CHISQ.INV.RT(CHISQ.DIST.RT(12.3,...))` J2: expected 12.3, actual 12.299999999999994.

Hypothesis (the oracle is the judge): Excel solves P(a, x/b) = p iteratively on its own GAMMA.DIST
kernel (T-0971); CHISQ.INV.RT solves on the upper tail. The round-trip residue (9.999999999999998)
points at a tolerance-based stop. **Needs an oracle sweep (T-0018):** fit against the T-1045
incomplete-gamma sweep (family 6), never blind.

More cells (T-1289's gen-m9 suite, excel-graph, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/4df1b22d3ed63f58 Calc!A9 `=CHIINV(T.DIST(0.1,7,Data!D1),2)` (D1 = 0.2, cumulative): Excel 1.2382105318705625, engine 1.2382105318705663. CHIINV with 2 degrees of freedom is -2*LN(p) in closed form; the miss may be T.DIST's (T-0969) as much as CHIINV's.

## Acceptance criteria
- [ ] All 11 NumericClose lines above leave that class, at least 20 of the 39 lines leave
      `eval/baseline/ironcalc-calc.txt`, no new lines in any baseline; remaining lines listed in the PR.
- [ ] No statrs `inverse_cdf` left for Gamma/ChiSquared (`rg inverse_cdf`); bounded iteration on
      untrusted input (huge shape/scale, p near 0 and 1).
- [ ] `make check` green; reviewers approve.

## Out of scope
The CDF kernel (T-0971); legacy GAMMAINV/CHIINV evidence (T-1047, T-1048).
