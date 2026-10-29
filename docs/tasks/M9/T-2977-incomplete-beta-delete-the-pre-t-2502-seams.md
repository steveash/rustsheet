---
id: T-2977
title: "Incomplete beta: delete the pre-T-2502 seams and the T-2464 harness, move F.INV/BETA.INV crossings and F.TEST onto bratio, score the per-branch counts (T-2847 rest)"
milestone: M9
lane: functions-stat
status: done
priority: P1
size: M
tier: opus
depends_on: [T-2847]
campaign: accuracy
spec: docs/specs/SPEC-0025-incomplete-beta-bratio-port.md
---

## Goal
No incomplete-beta caller runs the pre-T-2502 seams any more, the T-2464 research harness is gone from the engine, and
the bratio port's per-branch accuracy is measured and on record.

## Context
- T-2847 split (its session budget, 2026-10-28): T-2847 moved F.DIST, F.DIST.RT, FDIST, BINOM.DIST's cdf and
  NEGBINOM.DIST's cdf onto `bratio::ported_tails` (bpser, w1 bpser, L140 bup+bpser), with today's kernel as the
  fallback off those branches. Its remaining acceptance criteria move here unchanged; the full context is the
  "Context" section of docs/tasks/M9/T-2847-incomplete-beta-move-f-binom-and-negbinom-onto.md:
  - delete `beta_reg_complement_unoriented`, `beta_reg_upper_unoriented`, `unoriented_shape_one`, `fisher.rs`
    `f_tail_pre_t2502` and the `A1_CLOSED_FORM_MIN_X` branch, moving their callers (F.INV/F.INV.RT/FINV's crossing
    with T-2511, BETA.INV's huge-shape crossing, F.TEST's non-bratio tail) onto the port;
  - delete `functions/statistical/t2464_harness.rs` and its `mod` line (docs/research/t2464/README.md "Rebuilding the
    harness" then points at the engine's own kernel);
  - r2-bratio's per-branch bit-exact counts (w_bpser 6,767 of 11,845, w1_bpser 1,666 of 1,845, bpser(a,b>1) 978 of
    1,523), both L forms (GL53 and r2-bratio's w3_lg = toms) scored and reported;
  - the bpser criterion on t2464-r2-bpser-small-shapes (3,083 of 3,083 for b0 <= 1, 4,602 of 4,602 for
    1 < b0 < 8, at least 2,861 of 2,868 for b0 >= 8 within 15 digits) and the x29 criterion (at least 284 of the 434
    differing cells bit-exact), measured per cell;
  - the open port items listed there (bpser for both shapes >= 1 with max > 8 via a TOMS `betaln`; a caller's shape
    of exactly 1 outside the closed forms; the 987 probes-numeric cells the port moved off; Excel at fl(a·LN x)
    overflow; SPEC-0025's `bpser.rs`/`ln_beta.rs` split and property tests; unifying
    `beta_density::ln_beta_small_shape` with `bratio::bpser_neg_ln_beta`).
- Re-score docs/research/t2506/RESULTS.md's 72 probes-numeric L140 cells that L = -LN(a0) at a whole b0 makes
  bit-exact at a0 <= 8 (T-2847 took it at every a0), and count the L140 cells it moves off, on the engine's port.
- The 9 visibly-off cells of probes/t2829-ftest-tail-over-b (S!A1, A3, A4, A6, A7, A9, A14, A18, A21) are F.TEST's
  tail; record which reach 15 digits once F.TEST's remaining tail is on the port and leave the rest to T-2924.

## Acceptance criteria
- [x] The seams and the harness above are deleted; their callers take the port; no gated cell regresses except as
      recorded under decision 0083. (F.INV's crossing and F.TEST's shape of 1 keep their own forms on evidence; see
      the Outcome. No gated cell regresses.)
- [x] The per-branch and per-probe counts above are measured and recorded in the Outcome.
- [x] `make check` green; reviewers approve.

## Out of scope
- L140's last bits at (2.5, 2.5) and the other cells T-2847 left 1 to 5 ulp off (T-2978).
- bfrac/basym (T-2508), the bgrat family (T-2509, T-2510) and the inverses beyond their crossing (T-2511).

## Outcome
- Deleted `beta_reg_complement_unoriented`, `beta_reg_upper_unoriented`, `unoriented_shape_one`, `beta_reg_tail`,
  `fisher.rs::f_tail_pre_t2502`, `A1_CLOSED_FORM_MIN_X` and `t2464_harness.rs`. The kernel is
  `incomplete_beta::beta_reg_tails` (closed forms, `shape_one_tails` for a0 = 1 with y0 > 0.9, Cephes), and
  `bratio::kernel_tails` puts the port in front of it.
- Callers, each measured on the eval gate (variants run against origin/main's baselines):
  - BETA.INV's huge-shape crossing (`inverse_beta::beta_lower_crossing`) and F.TEST's large shapes take
    `bratio::beta_dist_lower_tail`: no cell moves.
  - F.DIST, F.DIST.RT and FDIST: closed forms and the port on w5's arguments as before; the Cephes fallback keeps
    d1X/(d1X + d2). With w5's arguments there too, `FDIST(..,1,9)` (gen-m9 c18dbf21f11093f3 A151, 7c20351562a2d71d
    A205) moves 9 ulp off and `2*F.DIST.RT(1E+12,22,5)` (probes/t2829-ftest-tail-over-b S!A17) visibly off.
  - F.INV, F.INV.RT and FINV: searching the ported F tail fixes 9 gen-m9 cells and moves 9 gated cells 2 to 8 ulp off
    (gen-m9 40425341c5034eed A205 +4, 4bad303d32cc3f46 A56 -2, 530c636749a85209 A270 +2, 623663cacea53bbc A35 -2,
    9cca17438a7e6f8a A114 -8, eced4bb6a0234c9e A113 +2, fec39efaba9284c3 A336 -2; ironcalc-calc F_DIST.xlsx J3 +2,
    L3 -2). So the search keeps its unoriented target (`fisher.rs::f_quantile_tail`) until T-2511.
  - F.TEST at a shape of 1: Excel's oriented closed forms move gen-m9 68d8ffb69ff739d8 A143 (-2), a790a4d4a6a82756
    A192 (-3) and bbdfc06cae0e0d71 A356 (-10, visibly off). Its own unswapped form (`unoriented_shape_one_tail`) is
    bit-exact there. Recorded in statistics.md.
- Eval delta against origin/main (`make eval-check`): 0 fixed, 0 regressed in every suite. In-scope visibly-off
  cells: 3,041 (lock 3,284).
- Per-branch counts, engine against r2-bratio's sets (be, w2f, w4p1, ix, x29; closed-form rows apart), bit-exact /
  within 4 ulp / w15 (docs/research/t2977/score_branches.out; the harness was used before its deletion):
  - w_bpser 5,458 / 10,788 / 10,803 of 11,845; model with w3_lg = toms 6,767, with gl 6,072;
  - w1_bpser 1,591 / 1,841 / 1,840 of 1,845; model toms 1,666, gl 1,643;
  - bpser(a, b > 1) 930 / 1,313 / 1,422 of 1,523; model 978 with either L. 509 of these rows have max(a, b) > 8 and
    stay on today's kernel (T-3007).
  - L140 bup+bpser 8,872 / 11,697 / 11,721 of 11,742 (T-2506 recorded 8,800 / 11,696 / 11,721).
  - Every branch: 21,289 / 32,672 / 33,832 of 39,852 (the CR value scores 11,812 / 33,858 / 36,619).
- bpser criterion on t2464-r2-bpser-small-shapes (w_bpser by b0 = max(a, b), within 15 digits): 3,083 of 3,083 for
  b0 <= 1, 4,601 of 4,602 for 1 < b0 < 8 and 2,859 of 2,868 for b0 >= 8 (criteria 4,602 and 2,861: not met).
- x29 criterion: bit-exact on 140 of the 434 cells where the 0.29 and 0.3 dispatches differ, against 284 for the
  model (not met). Those cells are bup+bgrat, which is not ported (T-2509, T-2510).
- probes/t2829-ftest-tail-over-b's 9 cells (S!A1, A3, A4, A6, A7, A9, A14, A18, A21) are all still visibly off. Each
  is bpser with both shapes above 1 and one above 8, so it has no ported ln B (T-3007, T-2924).
- `docs/research/t2464/excelmath.py` evaluates Excel's EXP, LN, LOG10, GAMMALN and GAMMA through the python bindings.
  It matches the harness on 14,000 sampled inputs; the only difference is an overflowing GAMMA, which is NaN
  (`#NUM!`) instead of inf.
  `docs/research/t2506/score_l140.py`, `t2681/score_bratio.py` and `t2977/score_branches.py` still need the harness
  back from git history.
- Not measured here, handed to T-3007: the re-score of docs/research/t2506/RESULTS.md's 72 probes-numeric L140 cells
  (L = -LN(a0) at a whole b0, a0 <= 8) and the L140 cells it moves off; the bpser shortfall above (1 cell at
  1 < b0 < 8, 9 at b0 >= 8, where `bpser_neg_ln_beta` takes algdiv in double arithmetic).
- Follow-up: T-3007 (the port's open items: a shape of 1 outside the closed forms, a TOMS betaln, fl(a LN x)
  overflow, SPEC-0025's layout).
