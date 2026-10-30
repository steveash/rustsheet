---
id: T-3192
title: "Incomplete beta bpser: T-3170's open items (b0 >= 8 L diff, the 987 cells T-2505's port moved off, 6 shape-of-1 cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
campaign: accuracy
depends_on: [T-3170]
spec: docs/specs/SPEC-0025-incomplete-beta-bratio-port.md
---

## Goal
The bratio port's bpser items that T-3170 left open are ported where Excel's form is found, or recorded as negative
results with their scores.

## Context
T-3170 ported Excel's w at a caller's shape of 1 (Excel's expm1 and log1p; statistics.md "Incomplete beta at a shape of
1 outside the closed forms") and found that running algdiv x87-stored for b0 >= 8 changes no cell (T-3170 Progress).
Open:
- the bpser criterion's shortfall at b0 >= 8 on probes-numeric/t2464-r2-bpser-small-shapes (engine 2,859 of 2,868
  within 15 digits against the model's 2,861): it is not algdiv's arithmetic, so diff the model's L and the engine's L
  at the cells `docs/research/t2977/score_branches.py` lists (needs the T-2464 harness back from git history, per its
  docstring);
- the 987 probes-numeric cells T-2505's port moved off the Cephes kernel (T-2847 Context), scored against the
  research model with the bindings venv (decision 0114): which are the port's and which the model's too;
- the 6 cells T-3170's shape-of-1 w moved off: probes/ftest-decomposition B12 (1 ulp, gated; decision 0170),
  t2220-f-dist-sweep-cdf A191, A195, t2220-f-dist-sweep-d2-1 A16 (1 ulp), t2220-f-dist-sweep-rt A195 (4 ulp) and
  t1745-beta-inv-roundtrip G145 (16 ulp, visibly off; 1 ulp of w near 0.95): a near-tie in Excel's log1p (its 27 known 1-ulp misses are of that kind) or another
  stored step.
Tools: `docs/research/t3007/cell_diff.py` for full-eval A/B diffs.

## Acceptance criteria
- [ ] Each item above is ported (no gated cell regresses), or recorded as a negative result in statistics.md with
      its scores.
- [ ] `make check` green; reviewers approve.

## Out of scope
- bfrac/basym (T-2508), the bgrat family (T-2509, T-2510), the inverses (T-2511), T-2924's fit of Excel's ln B.
