---
id: T-3046
title: "FREQUENCY: probe an error cell in a bins range, a referenced numeric-text bin, and re-measure corpus-bench/53062 (T-2636 follow-up)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
FREQUENCY's remaining unprobed argument orders are probed and recorded, and the corpus-bench/53062 count is re-measured after T-2636.

## Context
T-2636 made an error element of FREQUENCY's bins *array* skipped (probes/array-error-element-order A1-A2). Left open: an error
*cell* in a bins *range* (ours returns that error, frequency.rs `freq_values_of`), a data-array error element against an error
cell in a bins range, a referenced numeric-text bin, and the case's visibly-off count (334 before T-2636), which T-2636 could
not measure because corpus-bench is not fetched in the worker environment (`rustsheet-eval fetch --suite corpus-bench`).

## Acceptance criteria
- [ ] Probe the three orders above; record them in statistics.md and fix FREQUENCY if they differ from ours.
- [ ] Give corpus-bench/53062's visibly-off count after T-2636 and file a follow-up for any cell left.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
