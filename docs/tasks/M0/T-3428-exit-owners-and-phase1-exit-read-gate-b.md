---
id: T-3428
title: "exit_owners and phase1-exit read Gate B and the R1(c) rows (decision 0188)"
milestone: M0
lane: harness
status: done
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
- [x] `exit_owners.py` lists only cells the new rows count (no Phase 1b-only probe cell, no R3 cell, no R7 cell).
- [x] `phase1_exit.py` shows Gate B as met / not met / n/a with the missing suites; scripts/test_*.py cover each.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The measures themselves (T-3359, T-3427).

## Outcome
Done (2026-10-31).
- **Report:** a new `r1c_left_out` field gives `case!Sheet!A1` and its R1(c) class for every visibly-off cell
  decision 0188 leaves out (`R1cCells::left_out`).
- **`exit_owners.py`:**
  - A milestone's probe cells come from `r1c_off_cells`, falling back to `visibly_off_cells` for an older report.
  - Its ironcalc-calc cells drop every left-out class.
  - Its rate cells drop only R3 passes (`counted(name, left_out, gate_b)`), because Gate B is the 15-digit share.
  - `--all-off` and `--unattributed` drop the left-out cells outside Gate B's suites (`v15_checkpoint.gate_b.suites`)
    and the R3 passes inside them.
- **`phase1_exit.py`:** the `function rate` line reads Gate B: `met`, `not met`, or `n/a` with the suites the run did
  not score. `n/a` fails the exit, and an older report reads `phase1_met`.
- **Decision 0211** records the change to the protected `scripts/`.
- **Tests:** `test_r1c_left_out_cells_leave_the_rows_and_r3_passes_leave_the_rate`,
  `test_cells_decision_0188_leaves_out_need_no_owner` and `test_the_rate_reads_gate_b_met_not_met_or_not_measured`.
  All 62 script tests pass.
- **Eval delta:** none; only the report and the scripts change.
