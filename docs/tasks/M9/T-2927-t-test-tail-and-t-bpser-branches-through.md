---
id: T-2927
title: "T.TEST tail and T bpser branches through the ported kernel: evidence for T.TEST's complement and a bpser that keeps small-df bits (T-2509 follow-up)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-2509]
---

## Goal
T.TEST's p-value and the T tails that bratio sends to bpser (the centre, bpser on z, and the deep tail, bpser on y)
come from the same kernel call as the bgrat tails, K(n/2, 1/2, y, z), without losing a bit-exact cell.

## Context
- T-2509 routes only the bgrat and bup + bgrat T points through the kernel (`student_t::kernel_tail`). Routing the
  bpser points too fixed some cells but lost 29 bit-exact eval cells at small integer df, each 1-2 ulp, e.g.
  gen-m9 `TDIST(2,3,1)` = 0.06966298427942158, `T.DIST.2T(0.5,1)` = 0.7048327646991335, `T.DIST.2T(1,1)` =
  0.5000000000000001 and ironcalc-calc STATISTICAL/T_DIST.xlsx D15/F15/C15/C37 (df 1, t = 1). The old double-double
  integer-df series is bit-exact there; the ported bpser (w3, `bratio::bpser`) is not.
- r2-tdist (docs/research/t2464/r2-tdist/RESULTS.md section 2) scores w3's bpser 351/850 and 410/557 on the two T bpser
  branches; find the form that also keeps the small-df cells (a different ln B at a = 1/2, or Excel's series at
  integer df), or show which cells each form wins and keep the better one per region.
- T.TEST (`student_t::student_t_tail_at`) evaluates the tail at a y that is already a double; which complement Excel
  passes (fl(1 - y) or a z of its own) is not probed. Probe it (excel-probe) before routing it.

## Acceptance criteria
- [ ] The T bpser points (cells.tsv lower side, branches w_bpser and w1_bpser) go through the kernel with no gated
      known-failure line added, or the Outcome shows per region why today's path stays.
- [ ] T.TEST's complement is evidenced by a probe and its tail takes the kernel where that evidence says so.
- [ ] `make check` green; reviewers approve.

## Out of scope
- bgrat (T-2509), the T density (T-2513), T.INV (T-2511).
