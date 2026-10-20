---
id: T-2129
title: "Ship the T-1941 GAMMA model: products, half-integers, x>=10, reflection and the K1 kernel"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-2123]
---

## Goal
GAMMA reproduces Excel's routine as identified by T-1941: integers, half-integers, the x ≥ 10 path, the product reductions and reflection bit-exact, and G on [1,2) (K1) with the best known coefficients. Supersedes the non-integer GAMMA scope of T-1529 and the open n ≥ 89 item of T-0973.

## Context
Split from T-2123 (LN landed there: `x87_log10.rs` `excel_ln`, numbers.md "LN is FLDLN2 then FYL2X"). Source of
truth: `docs/research/t1941/HANDOFF2.md` (§2, §3, §9, §12) and `docs/excel-semantics/statistics.md` "GAMMA, GAMMALN and LN
kernels (T-1941)" on branch `agent/research/T-1941-crack-gammaln-gamma-ln`; copy the coefficient files and checkers you
cite from there. Arithmetic: x87 double rounding, every stored op fl53(fl64(.)) (`arithmetic.rs` `extended_sum`,
`extended_product`, `extended_quotient`); an unstored stretch needs 64-bit-significand ops. Every internal LN is
`excel_ln`, EXP is `excel_exp`. Verify with fixtures as T-2123 did (`crate::test::kernel_rows`: a committed sample with the
Python checker's value per point, plus an ignored full-data rate test over `RUSTSHEET_T1941_ROWS`).

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

Expected parity (HANDOFF2 §3): x ≥ 10, integers and half-integers ~100%; (0,10) and negatives through G ~78% (88% within 1 ulp; [1.8,2) weaker).
4. Each open-kernel model gets a doc comment saying it is Excel's structure with the best known coefficients,
   not bit-exact, with its measured exact rate and a pointer to T-2122. Keep the coefficient arrays in one
   module so T-2122 can swap them.

## Acceptance criteria
- [ ] GAMMA x ≥ 10 matches the committed T-1941 runs at 99.84%; integer, half-integer and reflection paths are bit-exact on `t1042-gamma-*` and `kmyst-gam-h10x`.
- [ ] K1 reproduces the research model's exact rate (±0.5%) on the committed runs and equals the Python checker on a committed sample.
- [ ] Range edges as T-1042 (GAMMA(171.6) finite, GAMMA(171.7) #NUM!); no panics or unbounded loops on any f64.
- [ ] `eval/baseline/` known failures only shrink; the GAMMA lines from T-0973 / T-1529 are reduced or explained.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Finding the remaining coefficient bits (T-2122).
- GAMMA.DIST and the incomplete gamma / beta kernels (T-0971, T-0970).
