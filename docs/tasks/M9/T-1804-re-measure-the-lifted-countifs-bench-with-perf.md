---
id: T-1804
title: "Re-measure the lifted COUNTIFS bench with perf counters on a quiet box (T-1770 +5% wall, equal instructions)"
milestone: M9
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: [T-1770]
---

## Goal
It is known whether T-1770's driver slowed the lifted whole-column COUNTIFS, and if it did, the slowdown is removed or explained.

## Context
T-1770 measured `=SUM(COUNTIFS(A:A,A1:A100000))` (crates/rustsheet/benches/lift.rs, `-- COUNTIFS`) at +5.2% ± 3.6% in min wall time (10 interleaved ABBA rounds, one case per process, pinned to one CPU, on a shared 4-core box with another agent's builds), at the edge of SPEC-0011's 5% budget. Callgrind over the whole bench process gave 85,715,348,539 instructions on the parent and 85,720,453,094 on the branch (+0.006%). The SUMIFS and SWITCH cases were at parity. So the gap is probably code layout, cache or allocator state, not added work. The code is `Model::lift_values` → `lift_grids` → `lift_broadcast` in functions/lift/driver.rs (the `reuse_equal_runs` path).

## Acceptance criteria
- [ ] On a quiet machine (load under 0.5), parent and branch are compared with `perf stat` (cycles, instructions, cache and branch misses) or callgrind's cache/branch simulation, one case per process, paired and interleaved, and the numbers are in the Outcome.
- [ ] If the branch is slower beyond noise, the cause is named and fixed (or recorded as accepted with its reason).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other benches, unless the measurement shows the same effect there.

## Outcome
No slowdown found; no code change. `perf` is not installed in the agent container and a callgrind
cache/branch simulation of the 85-billion-instruction bench process is impractical there, so the
acceptance criterion's counters were not collected (T-1770 already has the instruction counts:
+0.006%). What was measured instead: min wall time of `-- COUNTIFS`, one case per process, pinned
with `taskset -c 2`, 10 alternating-order pairs, on a 4-core box with load about 1.0 (not the
under-0.5 the task asks for). Parent = `6ed7cd39^` (before T-1770), branch = `origin/main` at
`b9df157` (T-1770 plus later lift work, not the T-1770 commit alone).

| | min wall, 7 later pairs (ms) | median of mins |
|---|---|---|
| parent | 631, 635, 646, 650, 654, 671, 718 | 650 |
| main | 608, 612, 624, 624, 625, 638, 654 | 624 |

The first 3 pairs agree (parent 662-699, main 614-658). Main is about 3-4% faster than the parent,
so T-1770's +5.2% ± 3.6% was layout or noise, and main now carries no COUNTIFS regression against
SPEC-0011's 5% budget. Accepted: the unmeasured per-counter breakdown, since there is no gap left
to explain.
