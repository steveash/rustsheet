---
id: T-1275
title: "Commit a seeded gen-m7 suite with oracle values and file its gaps"
milestone: M7
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A committed, seeded `gen-m7` suite with excel-graph values scores the M7 dynamic-array and LAMBDA functions, and its gaps are filed as tasks.

## Context
T-1083 gave all 32 in-scope M7 entries signatures. Its seeded sample (`rustsheet-gen --milestone M7 --count 25 --seed 17`, not committed) was all accepted by excel-graph (6 cells left pending: excel-graph returned an error it cannot decode), and the engine passed 924/994 cells with no crash. Most misses are already filed (T-1271, T-1272, T-1273, T-1276, T-1241, T-1242). Commit the suite (suite.toml as gen-m6/gen-m5 do), record the baseline, and file the gaps not yet covered.

## Acceptance criteria
- [x] `eval/suites/gen-m7` committed with oracle sidecars and its baseline; new gaps filed.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
eval/suites/gen-m7 (25 cases, `rustsheet-gen --milestone M7 --count 25 --seed 17`, `gate = false` with no
baseline file, like gen-m5/gen-m6) has excel-graph sidecars. 6 cells stay pending because excel-graph returns an error it
cannot decode. The engine now passes 926 of 994 cells (93.16%; T-1083's sample run gave 924) with no crash.

The gap-triager sorted all 68 failing cells:
- 46 went to existing tasks, whose Context now cites the gen-m7 cells: T-1271, T-1272, T-1273, T-1241,
  T-1269, T-1313, T-1322, T-1326, T-1255, T-1214, T-1202, T-1201, T-1259, T-0915, T-0905, T-1249,
  T-1319, T-1261, T-1262.
- 21 went to 15 new tasks: T-1347 to T-1361. T-1360 holds the REGEX optional arguments (T-1345 was
  taken by T-1189) and T-1361 the delimiter ranges (T-1346 was taken on main).
- 1 has no task: bare PERCENTOF is `#NAME?`, the gen-m6 precedent.

New quirks are recorded in arrays.md, text.md, dates.md, lookup.md, numbers.md and number-formats.md.
