---
id: T-3343
title: "Progress dashboard: a Phase 1 line in the goals block from make phase1-exit --json"
milestone: M2
lane: harness
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-3332]
---

## Goal
The progress dashboard's goals block (docs/progress.md) shows the Phase 1 exit verdict and the names of the
failing criteria, read from `scripts/phase1_exit.py --json`, and does not compute them itself.

## Context
Follow-up of T-3332 (decision 0187). `make phase1-exit` prints one verdict per Phase 1 exit criterion, and
`--json` prints `{"overall", "failing", "lines": [{criterion, verdict, detail}]}`. The dashboard
(tools/src/rustsheet_tools/progress_report, the `progress-report` skill) shows `phase1_met` today, which is only
the function-rate criterion. It should show the overall verdict and the failing criteria so readers see the
whole exit.

## Acceptance criteria
- [ ] The goals block has a Phase 1 line with the overall verdict and the failing criteria, from `--json` on
      the same report the dashboard reads; a test with a fixture JSON pins the line for `met` and `not met`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing any criterion or measure of decision 0187.
