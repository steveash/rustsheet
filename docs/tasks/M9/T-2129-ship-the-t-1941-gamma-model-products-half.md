---
id: T-2129
title: "Ship the T-1941 GAMMA model: products, half-integers, x>=10, reflection and the K1 kernel"
milestone: M9
lane: functions-stat
status: done
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
- [x] GAMMA x ≥ 10 matches the committed T-1941 runs at 99.83% (138,313 of 138,552 distinct x; the research's 99.84% is over 155,964 cells with duplicates); integer, half-integer and reflection paths are bit-exact on `t1042-gamma-*` and `kmyst-gam-h10x`.
- [x] K1 reproduces the research model's exact rate (±0.5%) on the committed runs and equals the Python checker on a committed sample.
- [x] Range edges as T-1042 (GAMMA(171.6) finite, GAMMA(171.7) #NUM!); no panics or unbounded loops on any f64.
- [x] `eval/baseline/` known failures only shrink; the GAMMA lines from T-0973 / T-1529 are reduced or explained.
- [x] `make check` green; reviewers approve.

## Out of scope
- Finding the remaining coefficient bits (T-2122).
- GAMMA.DIST and the incomplete gamma / beta kernels (T-0971, T-0970).

## Outcome
`functions/statistical/gamma_x87.rs` `excel_gamma` is Excel's routine and the worksheet GAMMA uses it (statrs' Lanczos is gone
from GAMMA): integers ≤ 88 by the downward product, exact half-integers by the ascending product times √π (negatives by
π/(Gh·q), +0 from -171.5 on, where Excel flushes the subnormal), x ≥ 10 by the binary-powering x^(x/2) path with EXP V6, LN and
the shared Stirling series (`gammaln.rs` `stirling_series`), the descending product / 1/x / division-chain reductions to G on
[1,2), and the reflection below -10 with FSIN (`trig.rs` `excel_fsin`: RN53(RN64) of the true sine in double-double). K1 is
W4's `plain1.txt` in `gamma_kernels.rs` with its measured rate and a T-2122 pointer. Every loop is bounded (at most 8, 10, 172
or 64 steps). Rates on 4,680,197 distinct T-1941 points: integers and half-integers 330/330, x ≥ 10 0.998, [1,10) 0.784, (0,1)
0.838, [-10,0) 0.876, x < -10 6,304 of 6,308 (three inherit 1-3 ulp from GAMMA(q); the fourth is the research's FSIN tie). The committed fixture (600 per region plus
every integer and half-integer) equals the composed Python checkers (`docs/research/t1941/engine/garef.py`, x87 rounding; the
same composition with single SSE rounding scores lower in every region, e.g. x ≥ 10 0.960 against 0.993 on the sample) on all
3,330 points. New probe probes/gamma-x87-edges (excel-graph): the model's overflow edge (171.6243769563027 is the last finite
argument) and its 0 past the reflection's overflow are Excel's, 15 of 15. Eval: ironcalc-calc GAMMA.xlsx every GAMMA cell (13 fixed; GAMMA.DIST / GAMMA.INV remain, T-0971 / T-0970),
BETA_GAMMA.xlsx 8 fixed, probes/stat-argument-modes 1 fixed, 0 regressions. The non-gated t1042 cases: every GAMMA cell at the
integers 1..171 and the half-integers matches (the n ≥ 89 item of T-0973 included), 4 of 24 unit-interval and 2 negatives-edges
cells (GAMMA(-0.1), GAMMA(-1.1)) miss by an ulp through G. `special.rs` `ln_gamma` stays for GAMMA.DIST and the incomplete
gamma / beta kernels (T-0971, T-0970).
