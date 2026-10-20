---
id: T-0060
title: "Visible-parity campaign: campaign front-matter key orders campaign tasks before milestone order"
milestone: M0
lane: harness
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
Workers pick the tasks that fix values a user can see before last-ulp fits and probes. These are
numbers off past the 15th significant digit, and wrong types, errors or text. The tasks for them
are filed and tagged, and the policy is recorded (decision 0083).

## Context
The owner asked on 2026-10-20 for this order: first 15 significant digits for all in-scope
functions, then a few ulp, then bit parity. On the eval run of 2026-10-20 (`main` at 35a5e71):
- 110 of 505 scored in-scope functions have numbers off past the 15th digit.
- 131 have a wrong type, error or text.
- 90 of the 95 functions whose own kernel is off numerically are M9. TEXT is M10, and names and
  tables are M8.

`scripts/tasks.py next` orders by milestone before priority (decisions 0023 and 0026), so all of
that waits behind M5. Decision 0083 has the full measurement and the task list.

## Acceptance criteria
- [x] `scripts/tasks.py` accepts an optional `campaign:` key, rejects unknown names, and `next`
      orders `campaign: visible` tasks after P0 tasks and handoffs, before milestone order.
      Tests in `scripts/test_tasks.py` pin both.
- [x] docs/process/task-format.md and `.claude/agents/gap-triager.md` describe the key; decision
      0083 records the policy and the standing ulp-regression allowance.
- [x] New tasks filed: T-0061 (the fifteen-digit bucket and ratchet), T-2128 (GAMMA at
      non-integers), T-2129 (ACOTH), T-2130 (PMT, IPMT and PPMT) and T-2131 (IRR).
- [x] The existing tasks listed in decision 0083 carry `campaign: visible` and a
      `## Visible-parity stage` section.
- [x] `make check` green; reviewers approve.

## Out of scope
- Measuring the fifteen-digit bucket in the harness (T-0061).
- Changing the routines, the critical-lane line or milestone exit criteria.

## Outcome
Docs, task files, `scripts/tasks.py` ordering and its tests only; no eval delta. Tagged 43
existing tasks plus the 5 new ones (decision 0083 lists them).
