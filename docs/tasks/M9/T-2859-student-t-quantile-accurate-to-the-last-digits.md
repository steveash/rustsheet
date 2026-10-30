---
id: T-2859
title: "Student-t quantile accurate to the last digits for df 2 to 99998: refine statrs' inverse to the crossing of T.DIST (accuracy, 220 cells; the centre is T-2724's)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
T.INV, T.INV.2T, TINV and CONFIDENCE.T for df 2 to 99998 and p (two-tailed equivalent) in [2E-5, 1] are accurate to
a few ulps of the true quantile, as Excel's are, so the 220 cells below where only our kernel is off become visibly
right. The fix is a more accurate kernel, not a fit to Excel's bits (decision 0114, `accuracy`).

## Context
Gap triage 2026-10-27 (opus refill). The quantile is statrs' `inv_beta_reg` search (`t_inverse_cdf` in
functions/statistical/inverse_beta.rs, called from `t_quantile` / `t_upper_quantile` in t_dist.rs) followed by
`sqrt(df*(1-y)/y)`. Two separate losses:
1. General region (p/2 well inside (0, 0.5)): the search stops at a tolerance, about 1E-13 to 1E-10 relative.
   ironcalc CONFIDENCE.xlsx E2 `CONFIDENCE.T(0.05,1,50)`: Excel 0.2841968554957298, ours 0.28419685549566487;
   T_DIST.xlsx D17 `T.INV.2T(T.DIST.2T(1,100),100)`: Excel 1.0000000000000004, ours 0.9999999999993039;
   probes-numeric/m9-t-inv-2t A110 `T.INV.2T(0.25,1000)`: Excel 1.1510179278370747, ours 1.1510179269707717.
2. Centre, owned by T-2724, not this task (two-tailed p near 1, one-tailed p near 0.5, so |t| small): `y` is near 1 and `1 - y` cancels, then the
   search floor leaves an absolute error of about 1E-8. gen-m9/329b177b5d5165ad A389 `T.INV.2T(1-1E-10,10)`: Excel
   1.2849891237857122e-10, ours 3.332000937312528e-08; `TINV(1-1E-16,8)` Excel 1.4355131050823135e-16, ours 0;
   T_DIST.xlsx D62 `T.INV.2T(0.9999999,10)`: Excel 1.2849890167888884e-07, ours 1.2467205919833128e-07; E20
   `T.INV(0.500000389989757,11)`: Excel 9.999999999666944e-07, ours 1.0000999564468016e-06.

Excel is the accurate one here. High-precision reference (mpmath 1.4.1, 40-60 digits, P(|T| > t) =
I_{df/(df+t^2)}(df/2, 1/2) inverted by bisection; the scripts are trivial to redo): on 40 random cells of the two
groups Excel is within 3E-15 relative of the true quantile on 37 (median 3E-16), ours 6E-16 to 100% off. Examples:
`CONFIDENCE.T(0.05,1,50)` true 0.284196855495729923 (Excel 3.7E-16 off, ours 2.3E-13); `T.INV.2T(0.5,100)` true
0.676951043011471478 (Excel 2.8E-15, ours 2.3E-13); `T.INV.2T(1-1E-10,6)` true 1.3063946375759297E-10 (Excel
1.2E-15, ours 197x); `T.INV.2T(0.999999,1000)` true 1.253627505003303127E-6 (Excel 2.5E-17, ours 4.2E-2).
The exceptions, where Excel itself is about 1E-14 off and ours is nearer the truth (`T.INV.2T(0.9,3)` Excel 8.1E-15,
ours 6.3E-16; `TINV(0.5,9)` Excel 2.0E-15, ours 6.5E-16; `TINV(1-1E-16,6)`, where Excel reads 1-p differently), stay
with T-0967: an accurate kernel cannot reach them.

Approach (the implementer decides, these are the known constraints):
- Reuse, do not copy: `special::refine_increasing` / `nearest_crossing` (T-0974 solves F.INV as the crossing of
  F.DIST's tails this way, fisher.rs) and the engine's accurate tails `student_t::student_t_tails` (T-0969: within a
  few ulps of the exact value). For the general region, refine statrs' estimate to the crossing of the upper tail
  `P(T > t) = q` (q = p/2 two-tailed), never of `1 - q`.
- The centre (T-2724's; recorded here so the two kernels agree at the boundary) cannot come from any crossing of the cdf near 0.5 (its doubles are 1.1E-16 apart, so t would only be
  known to about 3E-16 absolute). Solve the central probability instead: P(|T| < t) = 1 - p (exact for p in
  [0.5, 1], Sterbenz) with P(|T| < t) = I_{t^2/(df+t^2)}(1/2, df/2) (incomplete_beta.rs `beta_reg`, or its leading
  terms for tiny t: t = (1-p) / (2 f(0)) (1 + O(t^2)), f(0) the density at 0). One-tailed T.INV near 0.5 is the
  same with 1 - p = |2p - 1|.
- T-2839 owns p in (1, 2) (the reflection `-t_upper_quantile((2-p)/2)` lands in the same centre). T-2724 and T-2839 share that
  centre path; this task only keeps its boundary consistent.
- Expect ulp-level moves on cells that pass or fail at the ulp level today: decision 0083's standing allowance
  (`rustsheet-eval update-baseline --allow-regressions`, each regressed cell numeric and within 4 ulp, listed in the
  Outcome with a follow-up) covers them. T-0972's gamma-quantile branch stopped exactly there (2026-10-25: the run's
  permission policy refused the flag); check before starting that this run may use it, or design for zero
  regressions (for example keep today's value where it is already within 1 ulp of the refined crossing).
- Keep the existing boundaries: df 1 is T-2858's closed form, df >= 99999 and p/2 below 1E-5 stay on today's series
  and tail paths (T-1542, T-1043), and the search stays bounded (directive 5; `refine_increasing` is at most 129
  evaluations).

Owned cells (visibly off on 2026-10-27, from the eval report; 91 in gated baselines: 54 gen-m9, 36 ironcalc-calc,
1 probes; 129 in the ungated probes-numeric suite). The 54 cells T-2724 also lists are left to it and removed here;
any remaining cell that turns out to be a centre cell (|t| small) moves to T-2724 in this task's first PR:
- gen-m9/0620505280d2ea6e.case.json (2): A28, A276
- gen-m9/09b47629f54b9817.case.json (2): A78, A268
- gen-m9/11eb3ab4f4f8104c.case.json (1): A32
- gen-m9/17c110a53924beb8.case.json (1): A51
- gen-m9/1b31092a0d79e5f1.case.json (1): A341
- gen-m9/201d67f1d59436ed.case.json (1): A295
- gen-m9/2138b909d8f810ea.case.json (1): A13
- gen-m9/2a18b2f5a4dc43ac.case.json (1): A389
- gen-m9/329b177b5d5165ad.case.json (1): A63
- gen-m9/334f3dc9ca937f10.case.json (1): A271
- gen-m9/38e4fc309fe199ea.case.json (1): A374
- gen-m9/3a25a03eb94fcfbd.case.json (1): A45
- gen-m9/46ac3ef5e61583ca.case.json (4): A63, A114, A187, A293
- gen-m9/4bad303d32cc3f46.case.json (1): A188
- gen-m9/607e85cfee83a689.case.json (1): A353
- gen-m9/60b0b6e69ad35a45.case.json (1): A57
- gen-m9/775d053de498b8c7.case.json (2): A114, A388
- gen-m9/787ba8d801105a60.case.json (1): A56
- gen-m9/7980ffb6095a340d.case.json (1): A46
- gen-m9/7fd0a98183c6babf.case.json (1): A90
- gen-m9/85f48d5cdfb24555.case.json (1): A372 (handed on from T-2695 to T-0967; this task's accuracy stage)
- gen-m9/8dd2312c96effb1b.case.json (1): A256
- gen-m9/91e167cd8d7eb652.case.json (2): A102, A393
- gen-m9/928d08e3e1eecb5f.case.json (1): A15
- gen-m9/92d4fa29e5510f1e.case.json (1): A295
- gen-m9/941c409532603d32.case.json (1): A170
- gen-m9/984cba67bbe6972b.case.json (2): A310, A338
- gen-m9/9cca17438a7e6f8a.case.json (2): A76, A320
- gen-m9/9e2225ead8a6a07b.case.json (2): A350, A361
- gen-m9/a2262ea466fcc142.case.json (1): A255
- gen-m9/a4090567758520df.case.json (3): A182, A306, A369
- gen-m9/a790a4d4a6a82756.case.json (1): A132
- gen-m9/b6593d7dd13f2dac.case.json (1): A135
- gen-m9/c2684066d8df263e.case.json (1): A51
- gen-m9/cb61169f4c033d4b.case.json (2): A231, A319
- gen-m9/d208bbafc3bc1997.case.json (1): A135
- gen-m9/d5fd9e05e9e6f466.case.json (1): A22
- gen-m9/d8f44d556674c2fd.case.json (1): A164
- gen-m9/ecc87f80077959e0.case.json (3): A5, A166, A348
- ironcalc-calc/STATISTICAL/CONFIDENCE.xlsx (15): CONFIDENCE.NORM CONFIDENCE.T!E2, E8, E11, E13, E16, E17, E30, E34, E35, E36, E48, E49, E53, E54, E56
- ironcalc-calc/STATISTICAL/T_DIST.xlsx (8): T.DIST T.INV T.DIST.RT!E7, E8, E10, E17, E21, E40, E75, E76
- ironcalc-calc/STATISTICAL/T_DIST.xlsx (13): T.DIST.2T T.INV.2T!D3, D7, D8, D17, D18, D47, D49, D51, D53, D54, D55, D58, D59
- probes-numeric/m9-confidence-t.case.json (25): A3, A4, A5, A9, A10, A13, A14, A15, A17, A19, A20, A24, A25, A27, A29, A30, A33, A34, A35, A39, A40, A43, A44, A45, A50
- probes-numeric/m9-t-edges.case.json (1): A44
- probes-numeric/m9-t-inv.case.json (38): A17, A19, A21, A23, A32, A34, A42, A45, A47, A50, A58, A60, A71, A73, A81, A82, A83, A84, A86, A87, A88, A89, A94, A95, A96, A97, A99, A100, A101, A102, A107, A108, A109, A110, A112, A113, A114, A115
- probes-numeric/m9-t-inv-2t.case.json (57): A16, A17, A20, A21, A23, A24, A29, A30, A33, A34, A35, A36, A37, A42, A45, A46, A47, A48, A49, A50, A55, A59, A61, A62, A63, A68, A72, A74, A75, A76, A81, A82, A83, A84, A85, A86, A87, A88, A89, A94, A95, A96, A97, A98, A99, A100, A101, A102, A107, A108, A109, A110, A111, A112, A113, A114, A115
- probes-numeric/t1048-tinv.case.json (8): A3, B3, A4, B4, A6, B6, A8, B8
- probes/stat-argument-modes.case.json (1): H518

Shared with T-3061 (which reproduces Excel's SQRT(df*(1/y-1)) form, stage `visible`): probes-numeric/m9-confidence-t
A5, m9-t-inv A108, m9-t-inv-2t A35 and probes/stat-argument-modes H518. Whichever task lands first takes them, and the
other re-triages them in its first PR.

Not owned: df = 1 (T-2858), p above 1 (T-2839), p/2 below 2E-5 and df >= 99999 (T-1542), 48 gen-m9 cells whose p or
df is a sub-expression (classify them after the change; any still visibly off goes to T-0967 or a follow-up).
These cells are also listed in T-0967, which keeps the bit-level target and waits on T-0969; this task is its
accuracy stage and needs no new CDF.

## Acceptance criteria
- [ ] At least 75 of the 91 gated visibly-off cells above leave eval/baseline (`make eval-baseline`), and every
      owned cell still visibly off is listed in the Outcome with its high-precision reference showing that Excel is
      further than 4E-15 relative from it (to T-0967). No new known-failure line in any baseline beyond decision
      0083's 4-ulp allowance.
- [ ] Of the 129 probes-numeric cells above, the Outcome reports how many pass and how many are within 15 digits.
- [ ] Unit tests for `CONFIDENCE.T(0.05,1,50)` within 15 digits of 0.2841968554957298 and
      `T.INV.2T(0.25,1000)` within 15 digits of 1.1510179278370747.
- [ ] statistics.md gets an entry "Student-t quantile: accurate crossing (T-2859)" with
      the reference method, the counts and the cells left to T-0967.
- [ ] The T.INV cases of `crates/rustsheet/benches/lift.rs` are no more than 50% slower (decision 0081); the
      search stays bounded.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Excel's own bit pattern where Excel is not correctly rounded (T-0967, T-0969).
- df = 1 (T-2858), p in (1, 2) (T-2839), tiny p and huge df (T-1542, T-1043).
- The centre, |t| small (T-2724).
- T.DIST, T.DIST.RT, T.DIST.2T (the cdf side).

## Visible-parity stage
Decision 0083/0114, stage `accuracy`: done when every owned cell is within half a unit of the 15th significant digit
of Excel's value, or is shown (high-precision reference in the Outcome) to be one where Excel is the inaccurate one,
which moves to T-0967. Bit-level residue goes to a `defer: phase-1b` follow-up.
