---
id: T-0069
title: "tasks.py stats --json: task counts per milestone, campaign stage and status for the progress dashboard"
milestone: M0
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: [T-0067]
spec: docs/specs/SPEC-0022-daily-progress-dashboard.md
---

## Goal
`python3 scripts/tasks.py stats --json` prints task counts that the progress dashboard reads, so
the generator never parses task front matter itself:
- per milestone and status;
- per campaign stage and status, with ready counts;
- the `defer: phase-1b` tasks.

## Context
- SPEC-0022, "Task counts".
- scripts/tasks.py already has `load()`, `ready()`, `CAMPAIGNS`, `DEFERRALS` and `OPEN`. Reuse
  them.
- scripts/ is protected, so this needs a decision record (`make new-decision`).

## Acceptance criteria
- [x] `tasks.py stats --json` prints one JSON object with three keys:
  - `by_milestone`: milestone → status → count;
  - `by_campaign`: stage → status → count, plus `ready`, counted with `ready()` and no claim
    check;
  - `deferred`: `{open, done}` for `defer: phase-1b` tasks.
  Keys are sorted, so the output is deterministic.
- [x] It never fetches, and it exits non-zero on the same errors `validate` reports.
- [x] scripts/test_tasks.py covers a fixture set with campaign, defer, blocked and dropped tasks.
- [x] A decision record covers the protected change. `make check` green; reviewers approve.

## Out of scope
- Branch-claim-aware ready counts (`available()`, which fetches).
- Rendering (T-0072).

## Outcome
`scripts/tasks.py stats --json` prints `by_milestone`, `by_campaign` (with `ready`) and
`deferred`, keys sorted, every status and stage listed even at zero (decision 0119). It reuses
`load()`, `ready()`, `CAMPAIGNS` and `OPEN`; a stage counts the tasks tagged with it. It never
calls git (the tests fail on any subprocess call) and runs `validate`'s checks through a shared
`check()`: on an error it prints the errors to stderr, no counts, and exits 1. scripts/test_tasks.py
covers a fixture set with campaign, defer, blocked and dropped tasks, the claim-free `ready`
count, validation and stale-TODO failures, and the `--json` requirement. On main (2026-10-25):
semantics 51 todo / 46 ready / 29 done, accuracy 13 / 9 / 1, visible 32 / 22 / 216 (4 blocked),
deferred 36 open and 1 done. No eval change.
