---
id: T-2128
title: "Ship the T-1941 GAMMALN model: Stirling x>=8, recursions and the K2, K3, (1.5,2] kernels"
milestone: M9
lane: functions-stat
status: done
priority: P1
size: M
tier: opus
depends_on: [T-2123]
---

## Goal
GAMMALN and GAMMALN.PRECISE reproduce Excel's routine as identified by T-1941: the solved pieces bit-exact, the open kernels (K2, K3, (1.5,2]) with the best known Excel-structured coefficients. Supersedes the GAMMALN scope of T-1529.

## Context
Split from T-2123 (LN landed there: `x87_log10.rs` `excel_ln`, numbers.md "LN is FLDLN2 then FYL2X"). Source of
truth: `docs/research/t1941/HANDOFF2.md` (§2, §3, §9, §12) and `docs/excel-semantics/statistics.md` "GAMMA, GAMMALN and LN
kernels (T-1941)" on branch `agent/research/T-1941-crack-gammaln-gamma-ln`; copy the coefficient files and checkers you
cite from there. Arithmetic: x87 double rounding, every stored op fl53(fl64(.)) (`arithmetic.rs` `extended_sum`,
`extended_product`, `extended_quotient`); an unstored stretch needs 64-bit-significand ops. Every internal LN is
`excel_ln`, EXP is `excel_exp`. Verify with fixtures as T-2123 did (`crate::test::kernel_rows`: a committed sample with the
Python checker's value per point, plus an ignored full-data rate test over `RUSTSHEET_T1941_ROWS`).

2. **GAMMALN / GAMMALN.PRECISE** (identical routines):
   - x ≥ 8: 10-term Bernoulli Stirling, every op stored (`gammaln_big_check.c`; README "GAMMALN for x >= 8").
     Bit-exact.
   - x < 0.7 (exact compare with 0x1.6666666666666p-1): fl(GAMMALN(fl(x+1)) − LN(x)), recursive.
   - 3 ≤ x < 8: reduce t into [2,3), p = ascending stored product, fl(Gbase(t) + LN(p)); Gbase = K2.
   - (2,3]: K2 = (x−2)·fl(N/D), raw-x Horner N(9)/D(8), coefficients and structure from
     `docs/research/t1941/k2_ref.py` (0.95 exact, 0.978 within 1 ulp). If T-2122 has settled the structure as
     all-stored, use stored ops throughout with the T-2122 coefficient set. Reviewer 3's stored "cand9" set
     (HANDOFF2 §12.1) scores 0.993 on the near-2 ladders against k2_ref's 0.946, but is worse further out
     (0.933 against 0.950 on the long runs). Whether to use it for x−2 below about 0.02 is a measured choice
     for this task; see HANDOFF2 §12.6.
   - (1.5,2]: fl((x−1)·fl((x−2)·R)), R = N/D from W6's current chain (`w6/data/`, with D3 and the low block as in
     HANDOFF2 §3.5), D0 not normalised.
   - [0.7, 1.5): K3 = (x−1)·fl(N/D), (9,9), coefficients `w5/p4/ss_fc.txt`.
   - Exact zeros at 1 and 2 must hold (check them; the models give them).

Expected parity (HANDOFF2 §3): x ≥ 8 ~100%; (2,3] and [3,8) ~95% (98% within 1 ulp); near 1 and below 0.7 ~33%; (1.5,2] ~40%.
4. Each open-kernel model gets a doc comment saying it is Excel's structure with the best known coefficients,
   not bit-exact, with its measured exact rate and a pointer to T-2122. Keep the coefficient arrays in one
   module so T-2122 can swap them.

## Acceptance criteria
- [x] GAMMALN x ≥ 8 matches the committed T-1941 runs (291,957 of 291,957); x < 0.7 and [3,8) reproduce the research composition.
- [x] K2, K3 and (1.5,2] reproduce the research models' exact rates (±0.5%) on the committed runs, and equal the Python checkers on a committed sample; cand9 near 2 decided by measurement and documented.
- [x] GAMMALN(1) = GAMMALN(2) = 0; no panics or unbounded loops on any f64.
- [x] `eval/baseline/` known failures only shrink; the GAMMALN lines from T-1529 are reduced or explained.
- [x] `make check` green; reviewers approve.

## Out of scope
- Finding the remaining coefficient bits (T-2122).
- GAMMA.DIST and the incomplete gamma / beta kernels (T-0971, T-0970).

## Outcome
`functions/statistical/gammaln.rs` `excel_gammaln` is Excel's routine and the worksheet GAMMALN / GAMMALN.PRECISE use it;
`gamma_kernels.rs` holds the open kernels' coefficient arrays (K2 `k2_ref.py`, cand9, K3 `ss_fc.txt`, (1.5,2]
`xl_chain_d3ls.txt`, each with its measured rate and a T-2122 pointer, transcribed by bit pattern), and `x87_register.rs` a
64-bit-significand value for K2's register head. Every other op is a stored x87 op (`extended_*`) and every LN `excel_ln`.
The region ends come from Excel's own cells: 1.5 is K3, 3 is K2 and 4 reduces to t = 2 (the task text had [0.7, 1.5) and
3 ≤ x < 8; GAMMALN_GAMMALNPRECISE.xlsx A5/A9 and `kmyst-gam-c304/r203/r205` settle it). K2 takes cand9 for x - 2 < 0.01: the
measured optimum (statistics.md "GAMMALN is Excel's own x87 routine"). Rates on 18,360,467 distinct T-1941 points: x < 0.7
0.869, K3 0.322, (1.5,2] 0.428, K2 (2,2.02) 0.983, K2 [2.02,3] 0.950, (3,8) 0.991, x ≥ 8 292,645 of 292,649 (the 4 misses are
LN near-ties where Excel's LN column also differs, so Stirling is exact given Excel's LN). These are the research models'
rates: the committed fixture (1,500 random points per region) equals the composed Python checkers on all 10,500 points.
Eval: ironcalc-calc GAMMALN_GAMMALNPRECISE.xlsx 91/91 (8 + 4 known failures gone), BETA_GAMMA.xlsx 6 fixed,
probes/stat-argument-modes 2 fixed, 0 regressions. The non-gated t1042 cases: every GAMMALN cell at integers 1..171 and
half-integers now matches (fdlibm missed 63 of the integers); 5 of 24 unit-interval cells miss by an ulp, all in K3. GAMMA
(T-2129), GAMMA.DIST and the incomplete gamma/beta kernels still use the old ln-gamma (`special.rs` `ln_gamma`, T-0971 and
T-0970 switch in their own tasks).
