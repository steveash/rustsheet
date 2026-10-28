---
id: T-2213
title: "Probe the T.DIST.RT rounded-y cut to the ulp and above df 1000"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
defer: phase-1b
---

## Goal
The switch in Excel's Student-t tail between the exact tail at t and the tail at the rounded
y = fl(n/fl(n+fl(t*t))) is pinned. The cut is now `t > 1 && y <= 0.99` (`ROUNDED_Y_MAX` in
`functions/statistical/student_t.rs`), and the rule is backed by evidence above df 1000.

## Context
T-0969 attempt 7 found the switch on probes-numeric `t0969-t-tail-grid` (statistics.md,
"T.DIST.RT follows the rounded y beyond t = 1"). The 0.05 grid brackets the cut at t = 1 for
df <= 100, and at t^2 between n/100 and n/99 for df 200 to 1000. It cannot tell y < 0.99 from
y <= 100/101, nor t > 1 from t >= 1. No oracle cell has df > 1000 in the switched band.

## Acceptance criteria
- [ ] A probes-numeric case (excel-graph) of T.DIST.RT on dense t runs around t = 1 (df 10, 50),
      around y = 0.99 and y = 100/101 (df 200, 1000), and at df 1E4 and 1E5 with t from
      sqrt(n/99) to sqrt(n).
- [ ] `ROUNDED_Y_MAX` and the t = 1 test match where the cut falls on that probe, or the probe
      shows they already do.
- [ ] Findings in the statistics.md entry, with its status updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Deferred (decision 0114)
`defer: phase-1b`: the cells this task owns are already within 15 significant digits of Excel, so it waits until Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells visibly off (then give it a `campaign:` stage instead).
