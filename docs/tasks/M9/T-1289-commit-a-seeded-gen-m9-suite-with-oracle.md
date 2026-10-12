---
id: T-1289
title: "Commit a seeded gen-m9 suite with oracle values and file its gaps"
milestone: M9
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: [T-1080]
---

## Goal
A committed, seeded `gen-m9` suite with excel-graph values scores the M9 statistical functions, and its gaps are filed as tasks.

## Context
Like T-1239 for gen-m6. T-1080's uncommitted sample (`rustsheet-gen --milestone M9 --count 20 --seed 17`: 232 calls to
107 of the 142 statistical functions) was all accepted by excel-graph (0 failures), and the engine passed 746/800 cells with
no abort. Decide whether to wait for the financial, engineering and database signatures (T-1084, T-1085, T-1086) so the
suite covers all of M9, or commit the statistical half now. The FORECAST.ETS family is `#N/A` on excel-graph even for a
plain 8-point series (probes/stat-argument-modes, docs/excel-semantics/statistics.md "Statistical function argument
modes"): check whether generated ETS cells score anything before relying on them.

## Acceptance criteria
- [x] eval/suites/gen-m9 is committed with oracle sidecars (non-gating, like gen-m5), and each distinct engine gap is a task.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
eval/suites/gen-m9 (25 cases, `rustsheet-gen --milestone M9 --count 25 --seed 17`, `gate = false` with no baseline
file, like gen-m5/m6/m7) covers all of M9, since the financial, engineering and database signatures (T-1084..T-1086) had
landed. It has excel-graph sidecars; 2 cells stay pending because excel-graph returns a rich-value error it cannot decode
(696af2c8cee2f57e A40, 89693ab0cafc29df A2). The engine passes 895 of 998 cells (89.68%) with no crash.

The gap-triager sorted all 103 failing cells:
- 66 went to 30 existing tasks, whose Context cites the gen-m9 cells (T-1361, T-1273, T-1360, T-1293, T-1348,
  T-1241, T-1201, T-1326, T-1271, T-1254, T-1322, T-1236, T-1290, T-1349, T-1372, T-1255, T-0916, T-0221 and
  one cell each to T-1291, T-1385, T-0915, T-1214, T-1357, T-1292, T-0907, T-1384, T-1219, T-0972, T-0969, T-0973).
- 37 went to 13 new tasks: T-1401 to T-1413.
- 0 cells have no task.

FORECAST.ETS: only 3 FORECAST.ETS.CONFINT cells, all of them Excel argument errors (`#VALUE!` or `#N/A`), so they
check the family's argument validation, not its forecast. They are cited in T-1293 (statistics.md).
