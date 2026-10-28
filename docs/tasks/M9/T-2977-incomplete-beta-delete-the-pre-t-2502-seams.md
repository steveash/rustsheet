---
id: T-2977
title: "Incomplete beta: delete the pre-T-2502 seams and the T-2464 harness, move F.INV/BETA.INV crossings and F.TEST onto bratio, score the per-branch counts (T-2847 rest)"
milestone: M9
lane: functions-stat
status: todo
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
- The 9 visibly-off cells of probes/t2829-ftest-tail-over-b (S!A1, A3, A4, A6, A7, A9, A14, A18, A21) are F.TEST's
  tail; record which reach 15 digits once F.TEST's remaining tail is on the port and leave the rest to T-2924.

## Acceptance criteria
- [ ] The seams and the harness above are deleted; their callers take the port; no gated cell regresses except as
      recorded under decision 0083.
- [ ] The per-branch and per-probe counts above are measured and recorded in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- L140's last bits at (2.5, 2.5) and the other cells T-2847 left 1 to 5 ulp off (T-2978).
- bfrac/basym (T-2508), the bgrat family (T-2509, T-2510) and the inverses beyond their crossing (T-2511).
