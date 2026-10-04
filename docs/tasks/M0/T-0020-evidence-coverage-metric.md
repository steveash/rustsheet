---
id: T-0020
title: "Report Excel-evidence coverage per function"
milestone: M0
lane: harness
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
We can see which functions are proven by enough Excel evidence, not only which ones pass.

## Context
2026-10-04 eval: 435 functions have checked cells, 112 of them fewer than 10, and about 60
implemented functions have none. Pass rates on 3 cells say little.

## Acceptance criteria
- [ ] `make eval` report adds a section: per function, oracle/cached cells checked, source suites, and a flag below a threshold (e.g. 25 cells); plus implemented functions with zero evidence (from the `Function` enum).
- [ ] status.md snapshot shows the counts; the gap-triager uses the list to file probe/generator tasks for in-scope functions under the threshold.
- [ ] `make check` green; reviewers approve.
