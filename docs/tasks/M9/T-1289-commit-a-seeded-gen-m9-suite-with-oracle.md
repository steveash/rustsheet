---
id: T-1289
title: "Commit a seeded gen-m9 suite with oracle values and file its gaps"
milestone: M9
lane: harness
status: in-progress
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
- [ ] eval/suites/gen-m9 is committed with oracle sidecars (non-gating, like gen-m5), and each distinct engine gap is a task.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
