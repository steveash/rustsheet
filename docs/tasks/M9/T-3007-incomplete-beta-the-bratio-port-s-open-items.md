---
id: T-3007
title: "Incomplete beta: the bratio port's open items after T-2977 (shape of 1 outside the closed forms, TOMS betaln for both shapes >= 1 above 8, fl(a LN x) overflow, SPEC-0025 layout)"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
campaign: accuracy
depends_on: [T-2977]
spec: docs/specs/SPEC-0025-incomplete-beta-bratio-port.md
---

## Goal
The bratio port reaches the points it still hands to today's kernel for want of a piece, and its code has SPEC-0025's
layout and tests.

## Context
- T-2977 deleted the pre-T-2502 seams and the T-2464 harness and scored the engine per bratio branch
  (docs/research/t2977/score_branches.out; T-2977 Outcome). These items from T-2847's Context were left open:
  - a caller's shape of exactly 1 outside the closed forms (a0 = 1, y0 > 0.9) stays on
    `incomplete_beta.rs::shape_one_tails` (`TODO(T-3007)` in `bratio::ported_tails`): bpser(1, 3.5, x0) is 1 ulp
    high at probes/ftest-decomposition B12, where y0^b0 is Excel's value. Find Excel's form before moving it;
  - bpser for both shapes >= 1 with max > 8 (`bpser_neg_ln_beta` returns `None`): 509 r2-bratio bpser_large rows
    and the F tails' fallback there (for example probes/t2829-ftest-tail-over-b S!A17, `2*F.DIST.RT(1E+12,22,5)`)
    keep today's kernel. Gap triage 2026-10-28 (origin/main 1a0b616f): the same probe's other visibly-off cells,
    all this branch (shapes df2/2 in {2.5, 6} and df1/2 from 8.5 to 13), had no open owner and are this item's too:
    probes/t2829-ftest-tail-over-b S!A1 (`2*F.DIST.RT(1E+12,17,12)`, Excel 5.654118407649194E-70, ours
    5.6541184076491873E-70), A3, A4, A6, A7 (Excel 4.032210775559413E-70, ours 4.032210775559509E-70), A9 and A18
    (`2*F.DIST.RT(1E+12,23,5)`). statistics.md (F.TEST section) puts them off ours by T-2924's k plus whole steps of
    the fl(a·LN x) grid; a TOMS `betaln` (gamln + algdiv, or bcorr for both >= 8) is the natural port, against
    T-2924's fit of Excel's betaln;
  - probe (excel-probe) Excel's value where fl(a·LN x) overflows, `BETA.DIST(0.1,1E308,0.5,TRUE)`;
  - SPEC-0025's `bpser.rs`/`ln_beta.rs` split, the hex-bits branch fixture and the complement, swap and
    monotonicity property tests, or a spec amendment; unify `beta_density::ln_beta_small_shape` with
    `bratio::bpser_neg_ln_beta` or say in the code why they differ;
  - the 987 probes-numeric cells T-2505's port moved off the Cephes kernel (T-2847 Context), scored against the
    model with the bindings venv (decision 0114).
- From T-2977's Context: re-score docs/research/t2506/RESULTS.md's 72 probes-numeric L140 cells that L = -LN(a0) at
  a whole b0 makes bit-exact at a0 <= 8, and count the L140 cells it moves off; and the bpser criterion's shortfall on
  t2464-r2-bpser-small-shapes (w15 4,601 of 4,602 at 1 < b0 < 8, 2,859 of 2,868 at b0 >= 8 against 2,861 for the
  model; `bpser_neg_ln_beta` takes `beta_density::algdiv` in double arithmetic for b0 >= 8, the model x87 stored).
- The model's w_bpser counts beat the engine's (6,767 against 5,458 bit-exact of 11,845; T-2977 Outcome): the
  model scores 6,767 with w3_lg = toms and 6,072 with gl (the engine's GL53), so part of the gap is the L form.
- Gap triage 2026-10-28 (eval on origin/main abefb50): the visibly-off cells of that same case, which no open task
  named, are this item's too (both shapes >= 1 with one above 8, so no ported ln B; T-2977 Outcome):
  probes/t2829-ftest-tail-over-b S!A1, A3, A4, A6, A7, A9, A18 (e.g. A1 `2*F.DIST.RT(1E+12,17,12)`, Excel
  5.654118407649194E-70, ours 5.6541184076491873E-70; A7 `2*F.DIST.RT(1E+12,23,12)`, Excel 4.032210775559413E-70,
  ours 4.032210775559509E-70). statistics.md (F.TEST, T-2829) says they are off by T-2924's ln B k plus whole steps
  of the fl(a·LN x) grid, so a TOMS betaln alone may not reach them: score them and say which part is left.

### Cases owned (T-2981)
Routed by T-2981 (eval of 2026-10-28 at `main` 9586958b): cdf cells on branches already ported (bpser for min(a,b) > 1, L140 bup+bpser, w and w1 bpser) or at a shape of 1 outside the closed forms, still visibly off; most of the ported-branch ones have a shape above 8, so they are this task's TOMS betaln item (verify per cell). 54 visibly-off cells in 10 cases, per case:
- probes-numeric/m9-beta-dist-a10: BETA.DIST cdf, ported BpserLarge 1
- probes-numeric/m9-beta-dist-a100: BETA.DIST cdf, ported BpserLarge 1
- probes-numeric/m9-beta-dist-a2p5: BETA.DIST cdf, ported BpserLarge 1
- probes-numeric/m9-beta-dist-a5: BETA.DIST cdf, ported BpserLarge 1
- probes-numeric/m9-f-dist-rt: F.DIST.RT cdf, ported L140BupBpser 1
- probes-numeric/t2220-f-dist-sweep-cdf: F.DIST cdf, ported BpserLarge 5, F.DIST cdf, ported L140BupBpser 8, F.DIST cdf, shape of 1 7
- probes-numeric/t2220-f-dist-sweep-d1-1: F.DIST.RT cdf, ported W1Bpser 1
- probes-numeric/t2220-f-dist-sweep-d2-1: F.DIST cdf, ported W1Bpser 1
- probes-numeric/t2220-f-dist-sweep-rt: F.DIST.RT cdf, ported BpserLarge 7, F.DIST.RT cdf, ported L140BupBpser 10, F.DIST.RT cdf, shape of 1 5
- probes-numeric/t2464-w2f-absweep: BETA.DIST cdf, ported WBpser 5

## Progress
- 2026-10-29: the "bpser for both shapes >= 1 with max > 8" item is ported (`bratio/ln_beta.rs`): accurate ln B at
  half-integer shapes, TOMS `betaln` elsewhere; scores and the rejected all-TOMS variant in statistics.md
  ("Incomplete beta bpser", T-3007). Eval: bit-exact +67, visibly off -79 on the beta family; gen-m9 and probes known
  failures -1 and -5. t2829-ftest-tail-over-b A1, A3, A4, A6, A7, A9 stay visibly off (T-2924's ln B k plus fl(a LN x)
  steps); the other items above remain open.
- 2026-10-29: the fl(a LN x) overflow item is a negative result (statistics.md "Incomplete beta at a shape near the
  double range", probes/t3007-beta-dist-ln-x-overflow): Excel for the web refuses any workbook holding BETA.DIST with a
  shape of 1E+295 or more, so `BETA.DIST(0.1,1E308,0.5,TRUE)` has no excel-graph value and keeps today's kernel. The
  same probe found bpser `#NUM!` once 1/B(a, b) overflows and 5 bgrat/bfrac cells where ours are `#NUM!` (11 off cells,
  T-3104). No engine change; eval: +27 probe cells (16 pass, 11 baselined as known failures).

- 2026-10-29: SPEC-0025's layout and tests: `bratio/mod.rs` is split into `bpser.rs` (bpser and its ln B) and `bup.rs`
  (bup, L140); the branch fixture `test/fixtures/bratio_beta_dist.fixture` (857 BETA.DIST cdf cells, branch and swap from
  r2-bratio's model; `docs/research/t3007/make_bratio_fixture.py`) backs the dispatch test (857 of 857 agree), a per-branch
  bit-exact ratchet and the complement, swap and monotonicity properties (spec amended for the single fixture). Excel's
  cdf itself falls by an ulp at three adjacent-double pairs (statistics.md "BETA.DIST's cdf is not monotone in its last
  bit"). `beta_density::ln_beta_small_shape` stays separate from bpser's ln B, with the reason in its doc comment. No
  value changes.

## Acceptance criteria
- [ ] Each item above is ported (no gated cell regresses), or recorded as a negative result in statistics.md with
      its scores.
- [ ] `make check` green; reviewers approve.

## Out of scope
- bfrac/basym (T-2508), the bgrat family (T-2509, T-2510), the inverses (T-2511), T-2978's ulp cells.
