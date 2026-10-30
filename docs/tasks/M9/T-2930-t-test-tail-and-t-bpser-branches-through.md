---
id: T-2930
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
- As landed, T-2509 routes every T point from df 3 through the kernel (`student_t::kernel_tail`), bpser branches
  included; df 1 and 2 keep the old path, and T.TEST is unchanged. Routing df 1 through the ported bpser lost
  bit-exact eval cells at small integer df, each 1-2 ulp, e.g.
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

## Added when T-2509 landed (2026-10-30, decision 0180)
Visibly off since T-2509 (Student t tails on the ported kernel), accepted by decision 0180; this task owns them:
- gen-m9/787ba8d801105a60: Calc!A91 `T.DIST.2T(0.5,1E+8)` (Excel 0.6170750785521777, ours 0.6170750785521784)
- probes-numeric/t2464-r2-tdist-p1: Calc!A10465, A3913, A4416, A9295 (`T.DIST.RT` at df 16, 1964, 1665, 16)

The ten gated cells T-2509 moves by 1 ulp: gen-m9 0620505280d2ea6e A158, A273; 334f3dc9ca937f10 A171;
3fb9cecf2c23169f A345; 44a8841352dc48e9 A310; 5389dd4d5b5739d2 A216; 5be29fa316931f05 A238; cb61169f4c033d4b A160;
a2262ea466fcc142 A58; eedd629f3d69fdda A261.
- Review follow-ups from T-2509's PR (#2055): one `Dd` type alias in `x87_exp.rs` for bgrat.rs, student_t.rs and
  discrete.rs; a unit test pinning `bgrat::algdiv_half(a).0` to `beta_density::algdiv(0.5, a)`; one exact-value
  assertion on the bup + bgrat branch (a bit-exact t2464-r2-tdist-p1 part C cell); `dd_exp` returning `None` (or
  clamping) for |a| > 650 so its range is enforced in the function.

