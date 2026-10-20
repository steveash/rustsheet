---
id: T-2123
title: "Ship T-1941 gamma models: bit-exact pieces plus best kernels"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
GAMMA, GAMMALN, GAMMALN.PRECISE and LN reproduce Excel's own algorithms as identified by T-1941. The solved
pieces are bit-exact. The four open kernels use the best known Excel-structured models, which are far closer
to Excel's bits than today's fdlibm / statrs kernels. This supersedes the GAMMALN / non-integer GAMMA scope of
T-1529 and the open n ≥ 89 item of T-0973.

## Context
Source of truth: `docs/research/t1941/HANDOFF2.md` (§2 solved pieces, §3 open kernels, §9 engine guidance) and
`docs/excel-semantics/statistics.md` "GAMMA, GAMMALN and LN kernels (T-1941)". Research is on branch
`agent/research/T-1941-crack-gammaln-gamma-ln`. Copy the coefficient files and checkers from there (they reach
main with that branch; if they haven't yet, take them from the branch).

Arithmetic model, everywhere: x87 double rounding. Every stored op is fl53(fl64(.)): compute in an 80-bit
long double, or emulate with exact rational arithmetic / double-double, then store to double. Plain f64 (SSE2)
arithmetic is measurably wrong (e.g. 0.937 vs 0.952 on K2). Reuse the existing x87 helpers on main:
`crates/rustsheet-engine/src/x87_exp.rs` (EXP model V6, `excel_exp`) and `x87_log10.rs` (FYL2X emulation;
LN is the same with FLDLN2). Engine code to change: `functions/statistical/gamma.rs` (`fn_gamma`,
`fn_gamma_ln`), `functions/statistical/special.rs` (`ln_gamma`), and worksheet LN in
`functions/math_and_trigonometry`.

What to implement, in order (each piece cites its evidence file):
1. **LN** = RN53(RN64(RN64(ln2)·log2 x)), FLDLN2 + FYL2X (`docs/research/t1941/lnmodels.py`; 114,687/114,688).
   Worksheet LN and every internal LN in the gamma paths use it.
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
3. **GAMMA**:
   - integers n ≤ 88: downward product (n−1)(n−2)…2 (T-1524).
   - exact half-integers (x == floor(x) + 0.5): ascending product × √π double; negative half-integers
     ±fl(fl(π/Gh(q))/q); x ≤ −171.5 gives 0.
   - x ≥ 10: binary-powering x^(x/2) path with EXP V6 and LN (README "GAMMA for x >= 10"; 99.84%).
   - 1 ≤ x < 10: descending stored product × G(y); 0 < x < 1: fl(fl(1/x)·G(fl(x+1))); −10 < x < 0: the z/y
     loop then z·G(y); x ≤ −10: reflection with FSIN as RN53(RN64(sin(RN53(π·z)))) (README; locate the exact
     reflection cut with `h104`, between −10 and −30).
   - G on [1,2) = K1: raw-y Horner (7,7), G = fl(N/D), D0 = −1/16, coefficients `w4/k1r/plain1.txt` (0.776 exact,
     0.878 within 1 ulp).
   - Range edges as T-1042: GAMMA(171.6) finite, GAMMA(171.7) = #NUM!.
4. Each open-kernel model gets a doc comment saying it is Excel's structure with the best known coefficients,
   not bit-exact, with its measured exact rate and a pointer to T-2122. Keep the coefficient arrays in one
   module so T-2122 can swap them.

Expected parity (HANDOFF2 §3), as fractions of points matching Excel to the bit:
- LN, GAMMALN x ≥ 8, GAMMA x ≥ 10, integers and half-integers: ~100%.
- GAMMALN (2,3] and [3,8): ~95% (98% within 1 ulp).
- GAMMA on (0,10) and negatives (through G): ~78% (88% within 1 ulp; [1.8,2) weaker).
- GAMMALN near 1 and below 0.7: ~33%.
- GAMMALN (1.5,2]: ~40%.
- All misses are a few ulp (≤ 7), invisible at 15 significant digits.

## Acceptance criteria
- [ ] LN matches the T-1941 LN runs (114,687 of 114,688 committed points in `kmyst-gam-*` column C) and adds no
      baseline lines elsewhere.
- [ ] GAMMALN x ≥ 8 and GAMMA x ≥ 10 match the committed T-1941 runs at the rates above. Integer, half-integer
      and reflection paths are bit-exact on `t1042-gamma-*` and `kmyst-gam-h10x`.
- [ ] The open kernels reproduce the research models' exact rates (to ±0.5%) on the committed runs.
- [ ] `eval/baseline/` known failures only shrink (`make eval-baseline` refuses regressions); the GAMMA /
      GAMMALN lines from T-0973 / T-1529 are reduced or explained.
- [ ] No panics or unbounded loops on any f64 input (recurrence loops bounded; NaN/inf handled).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Finding the remaining coefficient bits (T-2122).
- GAMMA.DIST and the incomplete gamma / beta kernels (T-0971, T-0970), which should switch to these routines
  in their own tasks.
