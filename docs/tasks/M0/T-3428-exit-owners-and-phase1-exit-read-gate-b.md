---
id: T-3428
title: "exit_owners and phase1-exit read Gate B and the R1(c) rows (decision 0188)"
milestone: M0
lane: harness
status: todo
priority: P0
size: S
tier: opus
depends_on: [T-3427]
---

## Goal
`scripts/exit_owners.py` lists only the cells the R1(c) rows and Gate B count, and `scripts/phase1_exit.py` reads Gate B
(`v15_checkpoint.gate_b`, n/a when not measured) and the R1(c) rows.

## Context
- Split out of T-3359 (criterion 5). Since T-3359, `v15_checkpoint.phase1_met` is `gate_b.met`, so phase1-exit's
  function-rate verdict already follows Gate B; it should also print `gate_b.measured` / `missing` and say `n/a`
  rather than `not met` when Gate B was not measured.
- `scripts/` is a protected path: the PR needs a decision record (`make new-decision`).

## Acceptance criteria
- [ ] `exit_owners.py` lists only cells the new rows count (no Phase 1b-only probe cell, no R3 cell, no R7 cell).
- [ ] `phase1_exit.py` shows Gate B as met / not met / n/a with the missing suites; scripts/test_*.py cover each.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The measures themselves (T-3359, T-3427).
