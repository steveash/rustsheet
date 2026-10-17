---
id: T-1804
title: "Re-measure the lifted COUNTIFS bench with perf counters on a quiet box (T-1770 +5% wall, equal instructions)"
milestone: M9
lane: engine
status: todo
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
