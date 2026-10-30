---
id: T-3170
title: "Incomplete beta bpser: T-3007's remaining items (b0 >= 8 algdiv shortfall, Excel's w at a shape of 1, the 987 cells T-2505's port moved off)"
milestone: M9
lane: functions-stat
status: in-progress
priority: P1
size: M
tier: opus
campaign: accuracy
depends_on: [T-3007]
spec: docs/specs/SPEC-0025-incomplete-beta-bratio-port.md
---

## Goal
The bratio port's last open bpser items from T-3007 are ported where Excel's form is found, or recorded as negative
results with their scores.

## Context
T-3007 closed its other items (TOMS betaln and the half-integer ln B for both shapes > 1 above 8, the fl(a LN x)
overflow probe, SPEC-0025's layout and tests, the shape-of-1 w1 complement, the L140 -LN(a0) re-score). These remain
(T-3007 Context and Progress, statistics.md "Incomplete beta bpser"):
- the bpser criterion's shortfall on probes-numeric/t2464-r2-bpser-small-shapes: w15 4,601 of 4,602 at 1 < b0 < 8 and
  2,859 of 2,868 at b0 >= 8, against 2,861 for the model; `bratio::bpser::bpser_neg_ln_beta` takes
  `beta_density::algdiv` in double arithmetic for b0 >= 8 where the model runs TOMS algdiv x87 stored. Part of the
  w_bpser gap (engine 5,458 against the model's 6,767 bit-exact of 11,845; T-2977 Outcome,
  docs/research/t2977/score_branches.out) is the L form (the model scores 6,072 with GL53);
- Excel's w at a caller's shape of 1 outside the closed forms (a0 = 1, y0 > 0.9): w1 is now (0.5 - w) + 0.5 (T-3007,
  statistics.md "Incomplete beta at a shape of 1 outside the closed forms"), but 52 cells stay 1 ulp off on w itself;
- the 987 probes-numeric cells T-2505's port moved off the Cephes kernel (T-2847 Context), scored against the
  research model with the bindings venv (decision 0114): which are the port's and which the model's too.
Tools: docs/research/t2977/score_branches.py (needs the T-2464 harness back from git history, per its docstring) and
docs/research/t3007/cell_diff.py for full-eval A/B diffs.

## Progress
- 2026-10-29 (opus run): first item, part 1, is a negative result. I ran `beta_density::algdiv` x87-stored (every op
  through `extended_*`, `LN(b) - 1` a double op, as `r2-bpser-small/lib.py` `algdiv_x87` runs it) for
  `bpser_neg_ln_beta`'s min < 1, max >= 8 branch, and compared full `rustsheet-eval run` reports (base main
  8529a2849 against the change; `docs/research/t3007/cell_diff.py`). The two are bit-identical on every cell:
  probes-numeric 370,426 of 616,601 bit-exact both ways, and no cell changed. So the shapes case's 2-cell gap
  between the model and the engine at b0 >= 8 (2,861 against 2,859) is not algdiv's arithmetic. Next step: diff
  the model's L and the engine's L at those cells, which `score_branches.py` lists. The change was reverted (commit
  0305f063d, revert 8e63c0bd5) and is kept here only as this record. Items 2 and 3 are untouched.
- 2026-10-30 (opus run): resumed the idle branch (routine step 2).

## Acceptance criteria
- [ ] Each item above is ported (no gated cell regresses), or recorded as a negative result in statistics.md with
      its scores.
- [ ] `make check` green; reviewers approve.

## Out of scope
- bfrac/basym (T-2508), the bgrat family (T-2509, T-2510), the inverses (T-2511), T-2978's ulp cells; T-2924's
  fit of Excel's ln B for both shapes > 1 above 8.
