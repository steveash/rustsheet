---
id: T-1275
title: "Commit a seeded gen-m7 suite with oracle values and file its gaps"
milestone: M7
lane: harness
status: todo
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
- [ ] `eval/suites/gen-m7` committed with oracle sidecars and its baseline; new gaps filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
