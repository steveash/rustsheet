---
id: T-0033
title: "Stop per-PR status.md log lines; union-merge append-only docs"
milestone: M0
lane: harness
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
Concurrent PRs stop conflicting on `docs/status.md`.

## Context
Every PR appended a log line to the end of docs/status.md (work-loop step 6, agent-workflow step 9): 31 of 33 PRs since 2026-10-04 touched it and it was in nearly every merge conflict that day. Owner asked to remove the contention. See decision 0021.

## Acceptance criteria
- [x] work-loop and agent-workflow no longer ask PRs to append to status.md; CLAUDE.md directive 6 points progress at the task Outcome.
- [x] `.gitattributes`: `docs/status.md merge=union` (tested: two branches appending at the same spot merge cleanly); not for excel-semantics (reason in decision 0021).
- [x] status.md header says so; its log ends with a pointer.
- [x] `make check` green; reviewers approve.

## Out of scope
- Routine prompts (the steward prompt already lists merged work in step 1; no re-apply needed).

## Outcome
Per-PR status.md log lines removed from the process; union merge for status.md (decision 0021). Eval delta none.
