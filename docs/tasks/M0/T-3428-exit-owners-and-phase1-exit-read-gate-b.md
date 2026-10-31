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
- T-3427 added the R1(c) fields these scripts should read. Each probe row (`probe_milestones[]`) has `r1c_off_cells`,
  which is never more than `visibly_off_cells`, and each exit row (`milestone_exit[]`) has `probe_r1c_off`,
  `calc_r1c_off` and the `probes_met` / `calc_met` that follow them. The ironcalc-calc cells R1(c) counts are the
  `visibly_off` cells of the function-scope row that `phase1b_evidence`'s classes do not exclude. The simplest way to
  give the script those cells is an `r1c_off_cells` list on the scope row, added in Rust.
- `scripts/` is a protected path: the PR needs a decision record (`make new-decision`).

## Acceptance criteria
- [ ] `exit_owners.py` lists only cells the new rows count (no Phase 1b-only probe cell, no R3 cell, no R7 cell).
- [ ] `phase1_exit.py` shows Gate B as met / not met / n/a with the missing suites; scripts/test_*.py cover each.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The measures themselves (T-3359, T-3427).
