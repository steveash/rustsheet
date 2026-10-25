---
id: T-0065
title: "Adopt decision 0114: Phase 1 exits on fifteen-digit parity; semantics, accuracy, visible campaign stages; defer phase-1b"
milestone: M0
lane: harness
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
The repository works to decision 0114: Phase 1 exits on fifteen-digit parity, `next` picks
`semantics` tasks, then `accuracy`, then `visible`, parked last-bit tasks are never picked, and the
eval report measures the new exit.

## Context
Owner decision of 2026-10-25 (decision 0114), from the Phase 1 assessment of `main` 2c52346.

## Acceptance criteria
- [x] docs/milestones.md states the Phase 1 exit of decision 0114 and adds Phase 1b.
- [x] `scripts/tasks.py` accepts `campaign: semantics | accuracy | visible` and orders them in that
      stage order; `defer: phase-1b` tasks are never ready; `scripts/test_tasks.py` covers both.
- [x] docs/process/task-format.md and the gap-triager describe the stages and the defer key.
- [x] The eval report shows per-function within-15-digit rates and the Phase 1 line
      (`v15_checkpoint`), and the milestone exit rows count visibly-off cells.
- [x] The semantics and accuracy tasks of the assessment are filed or retagged; last-digit tasks
      carry `defer: phase-1b`; FORECAST.ETS value work moves to M14.
- [x] `make check` green; reviewers approve.

## Out of scope
- Any engine or function change: the filed tasks do that.
- Lowering a gate or a baseline.

## Outcome
Branch `claude/trusting-hypatia-5toqcu` (owner session https://claude.ai/code/session_015FFUAimJC9YQNhjLfmSH6Y).
- Decision 0114; docs/milestones.md Phase 1 exit and Phase 1b; task-format, gap-triager, steward skill
  and eval-harness docs updated.
- `scripts/tasks.py`: campaign stages `semantics` → `accuracy` → `visible`, dependencies at the
  earliest stage that needs them; `defer: phase-1b` never ready (3 new tests).
- Eval report: `v15_checkpoint` per-function within-15-digit share, `meets_rate` and the Phase 1
  line; milestone exit rows count visibly-off cells (bit failures in brackets);
  `Visible::is_off` is the one visibly-off test. Parity page: Phase 1 band table and share column.
- Tasks: 60 retagged `semantics`, 15 `accuracy`; 34 last-digit tasks `defer: phase-1b`; T-2425 moved
  to M14; new T-0461, T-0462, T-2660, T-2661, T-2662 (semantics, from calipers) and T-2663 (M14,
  blocked). Calipers MOD cells added to T-2654.
- Eval: 0 regressions in every gated suite (`make check`, 2026-10-25).
- Follow-up: T-0066 (parity page reads the share from `v15_checkpoint`; review MINOR).
- Review: security, architecture, quality APPROVE; MINORs fixed (shared visibly-off test, shared
  0.999 threshold, `gen_rate15` through `SuiteSummary::rate`) or filed (T-0066).
