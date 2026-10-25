---
id: T-0071
title: "Progress report: run history in docs/progress/history.json, since-last-run deltas, trend charts and stale handling for incomplete runs"
milestone: M0
lane: harness
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-0081]
spec: docs/specs/SPEC-0022-daily-progress-dashboard.md
---

## Goal
Each `make progress-report` adds one row to docs/progress/history.json. The page shows what
changed since the last complete run, and trend charts. When the eval is incomplete or a fetch
failed, the page says so and keeps the last good numbers, marked stale.

## Context
- SPEC-0022, sections "History and deltas" and "Failure handling". The row schema is in the spec.
- Inputs: report.json `run` (T-0080), `v15_checkpoint` (T-0080), the T-0068 tiers, tasks.json (T-0069),
  and git.
- The steward missed 2026-10-23 and 2026-10-24 without a trace in the repository. A missing date
  in the history must show as a gap in the trend charts, not be interpolated.

## Acceptance criteria
- [ ] The module `progress_report/history.py` handles the history file:
  - it reads docs/progress/history.json, or creates it with today's row only;
  - it adds one row per run date, and a rerun on the same date replaces that row;
  - it never rewrites an older row;
  - it fails non-zero on invalid JSON or a row missing a schema key.
- [ ] The `since-last-run` block shows deltas against the last *complete* row:
  - Phase 1 functions at rate;
  - functions per tier;
  - visibly-off cells by kind;
  - Excel cells;
  - tasks done;
  - commits on `main` since that row's commit (`git rev-list --count --first-parent`).
  On the first run it says there is no earlier run.
- [ ] The `trends` block has line-chart SVGs, with a dark-scheme style and explicit fills as in
      T-2041: functions at ≥ 99.9% within 15 digits, visibly-off cells, and open tasks per
      campaign stage. A missing date leaves a gap.
- [ ] An incomplete run (report missing or unparseable, an older shape, or
      `run.skipped_suites` not empty):
  - rewrites only the `status` block, with a Stale banner naming the reason and the last complete
    run;
  - leaves the other blocks, the SVGs and functions.md byte-identical;
  - adds a `complete: false` row;
  - exits 0.
  Tests cover each reason.
- [ ] A report whose `run.commit` is not `HEAD` still counts as complete, and `status` names both
      commits (test).
- [ ] `make py-check` green; `make check` green; reviewers approve.

## Out of scope
- Back-filling the assessment's numbers: the method differs (SPEC-0022).
- Running the generator in CI or a routine (T-0076).
