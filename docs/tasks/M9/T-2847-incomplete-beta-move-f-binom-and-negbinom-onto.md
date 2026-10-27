---
id: T-2847
title: "Incomplete beta: move F, BINOM and NEGBINOM onto the bratio port's bpser branches, delete the pre-T-2502 seams and the T-2464 harness (T-2505 rest)"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
campaign: accuracy
depends_on: [T-2505]
spec: docs/specs/SPEC-0025-incomplete-beta-bratio-port.md
---

## Goal
Every incomplete-beta caller, not only BETA.DIST, takes the bratio port's bpser branches, and the seams T-2502 left
for the Cephes kernel are gone.

## Context
- T-2505 split (its session budget): T-2505 landed Excel's bpser (`functions/statistical/bratio/mod.rs` `bpser`) and
  put BETA.DIST's cdf on the dispatch (`beta_dist_lower_tail`). Every other caller still runs `incomplete_beta.rs`.
- The remaining T-2505 acceptance criteria, moved here unchanged:
  - the callers per SPEC-0025's caller table: F.DIST, F.DIST.RT, FDIST and F.TEST's tail with the w5 forms
    x = 1/(1 + d2/(d1·X)), y = 1/(1 + d1·X/d2) everywhere; BINOM.DIST (k + 1, n - k, p, fl(1 - p)) → w1, with
    T-2645's k = 0 shortcut in front; NEGBINOM.DIST (r, f + 1, p, fl(1 - p)) → w;
  - r2-bratio's per-branch bit-exact counts (w_bpser 6,767 of 11,845, w1_bpser 1,666 of 1,845, bpser(a,b>1) 978 of
    1,523), both L forms (GL53 and r2-bratio's w3_lg = toms) scored and reported;
  - the bpser criterion on t2464-r2-bpser-small-shapes (3,083 of 3,083 for b0 <= 1, 4,602 of 4,602 for 1 < b0 < 8,
    at least 2,861 of 2,868 for b0 >= 8 within 15 digits) and the x29 criterion (at least 284 of the 434 differing
    cells bit-exact), measured per cell;
  - the two gated cells r2-impact expects to lose (gen-m9 F.DIST.RT 0a614cfe A19, FDIST 05d44f6d A175) recorded under
    decision 0083, and the effect on T-2633's chained BETA.INV cells stated;
  - delete `beta_reg_complement_unoriented`, `beta_reg_upper_unoriented`, `unoriented_shape_one`, `fisher.rs`
    `f_tail_pre_t2502` and the `A1_CLOSED_FORM_MIN_X` branch, moving their callers (F.INV/F.INV.RT/FINV's crossing with
    T-2511, BETA.INV's huge-shape crossing, F.TEST) onto the port;
  - delete `functions/statistical/t2464_harness.rs` and its `mod` line (docs/research/t2464/README.md "Rebuilding the
    harness" then points at the engine's own kernel).
- Open in the port T-2505 landed:
  - bpser for both shapes >= 1 with max(a, b) > 8 (`bpser_neg_ln_beta` returns `None`): the model takes an accurate
    ln B there (`bratio_excel.py` `lnb_w3`, `w3_big = "acc"`); a TOMS `betaln` (gamln + algdiv, or bcorr for both >= 8)
    is the natural port.
  - A shape of exactly 1 outside the closed forms (a0 = 1, y0 > 0.9) stays on `unoriented_shape_one`'s x^a, which is
    bit-exact on probes/ftest-decomposition B12 where bpser(1, 3.5, x0) is 1 ulp high. Find Excel's form before moving
    it.
  - On probes-numeric the port moved cells the wrong way (T-2505 Outcome). Score them against the model with the
    bindings venv before fitting anything (decision 0114: accuracy stage, no last-bit fitting); a cell the model gets
    and the port does not is a port bug:
    - 987 cells bit-exact on the Cephes kernel are now off, 984 by 1 to 4 ulp, for example
      `BETA.DIST(0.1,0.5,0.5,TRUE)` (m9-beta-dist-a0p5 A7);
    - newly visibly off: t2464-w2f-absweep A7053 (w_bpser, b0 < 8, bit-exact before), t0969-beta-prefactor Pref!D77
      and D83, t0969-t-beta-link Link!E109, t2464-r2-bpser-small-shapes A2543, A9491 and A10836 (4 of the 7 by the
      harness's rule);
    - newly past 4 ulp: m9-beta-dist-a0p5 A9; m9-legacy-betadist A19, A20; t0969-t-beta-link Link!I105;
      t1745-beta-inv-roundtrip G4, I4, G180, I180; t2464-r2-bpser-small-shapes A432, A1092, A2541, A5219 (8 ulp),
      A5816, A8117, A9633, A10836; t2464-w2f-absweep A7053;
    - `bpser_neg_ln_beta` takes `beta_density::algdiv` in double arithmetic for b0 >= 8, where the model runs TOMS
      algdiv x87 stored (r2-bpser-small `lib.py` `algdiv_x87`); this may explain part of the b0 >= 8 residue.
  - SPEC-0025's layout and tests: split `bratio/mod.rs` into `bpser.rs`/`ln_beta.rs` and give it the spec's
    `bratio`/`Tails`/`bratio_or` entry before T-2506 adds bup and bgrat (or amend the spec), and add the hex-bits
    branch fixture and the complement, swap and monotonicity property tests. `beta_density::ln_beta_small_shape` and
    `bratio::bpser_neg_ln_beta` overlap; unify them or say in the code why they differ.

## Acceptance criteria
- [ ] F.DIST, F.DIST.RT, FDIST, F.TEST, BINOM.DIST and NEGBINOM.DIST take `bratio`'s bpser branches with SPEC-0025's
      caller arguments; no gated cell regresses except as recorded under decision 0083.
- [ ] The per-branch and per-probe counts above are measured and recorded in the Outcome.
- [ ] The pre-T-2502 seams and the T-2464 harness listed above are deleted.
- [ ] statistics.md's bratio and bpser entries name every caller in their "Implemented in" lines.
- [ ] `make check` green; reviewers approve.

## Out of scope
- L140 (T-2506), bfrac/basym (T-2508), the bgrat family (T-2509, T-2510) and the inverses (T-2511).
