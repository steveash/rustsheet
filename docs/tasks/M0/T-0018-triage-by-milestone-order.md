---
id: T-0018
title: "Gap-triager and functions lane work in milestone order"
milestone: M0
lane: harness
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
The functions lane spends its runs on the current milestone's functions, not on M9 last-digit tails.

## Context
2026-10-04: 12 of the functions lane's open tasks were M9 numeric tails (two attempts already
blocked: T-0901, T-0925) while M2's everyday functions had no oracle evidence yet. The
gap-triager ranks by failing cells × frequency only.

## Acceptance criteria
- [x] gap-triager ranks groups by (milestone of the function, earliest first), then failing cells × frequency, and sets the task milestone from the catalog.
- [x] `scripts/tasks.py next` orders ready tasks by milestone before priority (or the functions routine prompt passes the current milestone); M9 tasks still get picked when nothing earlier is ready.
- [x] M9 numeric-fitting tasks that need oracle sweeps say so and depend on a sweep task instead of being attempted blind.
- [x] Decision record if protected paths change; `make check` green; reviewers approve.

## Outcome
- **Gap-triager** (`.claude/agents/gap-triager.md`): ranks groups by catalog milestone first, then
  failing cells × prior, and sets each task's milestone from the catalog. A function missing from
  the catalog counts as M9.
- **`scripts/tasks.py next`:** already ordered by milestone before priority (decision 0023), so
  nothing changed there. New tests in `scripts/test_tasks.py` pin that order and show an M9 task
  is still picked when nothing earlier is ready.
- **Sweep task:** filed T-0942 (research) for oracle sweeps. The six M9 fitting tasks that need
  more Excel points (T-0901, T-0908, T-0915, T-0916, T-0917, T-0925) now say "Needs an oracle
  sweep" and depend on it.
- **Decision record:** 0026 covers the protected-path change.
- **Eval:** unchanged (docs, agent prompt and tests only).
