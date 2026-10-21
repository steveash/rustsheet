---
id: T-2107
title: "Correctness first: performance reviewer off, 50% bench regression budget"
milestone: M2
lane: harness
status: done
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
While the project reaches Excel parity, PRs are not held by performance review: the performance
reviewer is off and a bench criterion allows up to 50% slower than main (owner, 2026-10-20).

## Context
T-1785 (decision 0077), T-1775 (#1164), T-1829 (#1153) and T-1898 (#1167) were held by the
performance reviewer or a "within 5%" bench criterion with correct results. Directive 5 (bounded
work on untrusted input) is a safety rule and stays with the security reviewer.

## Acceptance criteria
- [x] Decision 0081 records the rule.
- [x] The `review` skill no longer launches `reviewer-performance`; review-policy, agent-workflow,
      task-format and the reviewer's own file say it is off.
- [x] `make hygiene` green.

## Out of scope
- Re-enabling the reviewer or the M3 bench gate (a later decision).
- Re-prioritizing perf-lane tasks in the task tool.

## Outcome
Decision 0081. `.claude/skills/review/SKILL.md` drops `reviewer-performance`;
`docs/process/review-policy.md`, `agent-workflow.md`, `task-format.md` and
`.claude/agents/reviewer-performance.md` point at 0081.
