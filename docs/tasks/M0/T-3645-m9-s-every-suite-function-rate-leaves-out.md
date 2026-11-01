---
id: T-3645
title: "M9's every-suite function rate leaves out decision 0188 R1(c)'s Phase 1b cells (owner ruling)"
milestone: M0
lane: harness
status: done
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
M9's exit row reads the every-suite function rate without decision 0188 R1(c)'s Phase 1b cells (decision 0227),
so cells already within 1e-9 of the math on both sides no longer block Phase 1 through the rate, while Gate B keeps
counting them.

## Context
- Decision 0227 (owner ruling 2026-11-01). The 211 unowned M9 exit cells of 2026-11-01 were F.DIST Phase 1b rate
  cells whose only owners are `defer: phase-1b` tasks.
- Code: crates/rustsheet-eval/src/v15_checkpoint.rs (`LeftOutKind`, `phase1b_cells`, `rate_count`) and
  scripts/exit_owners.py (`counted_in_rate`). Gate B (crates/rustsheet-eval/src/gate_b.rs) is unchanged.

## Acceptance criteria
- [x] The every-suite rate leaves out Phase 1b cells and Gate B still counts them (unit test
      `phase1b_cells_leave_the_every_suite_rate_but_gate_b_counts_them`).
- [x] `exit_owners.py` drops Phase 1b rate cells and keeps M14-owned ones
      (`test_r1c_left_out_cells_leave_the_rows_and_the_rate`).
- [x] On a `make eval` run, `make exit-owners MILESTONE=M9` lists no Phase 1b cell, and the report names the cells
      left out per function.
- [x] `make check` green; reviewers approve (architecture and security for the protected scripts/ path).

## Out of scope
- Gate B, and the visibly-off cells owned only by deferred tasks in `make phase1-exit`'s `owners` line (a separate
  question).

## Outcome
Decision 0227 (owner ruling). `excepted_cells` (crates/rustsheet-eval/src/v15_checkpoint.rs) takes the run's R1(c)
classes as an option, and leaves Phase 1b cells out as a third `LeftOutKind` after listed and M14-owned cells.
- `summarize` passes them; Gate B passes `None`, so Gate B is unchanged.
- `scripts/exit_owners.py`'s `counted_in_rate` leaves out `r3_pass` and `phase1b` rate cells.
- Tests:
  - `phase1b_cells_leave_the_every_suite_rate_but_gate_b_counts_them`: Gate B still shows 1000 cells, 2 visibly
    off and not met, while the every-suite rate shows 998 cells, 0 visibly off and met.
  - `a_phase1b_cell_is_left_out_once_and_only_when_an_off_excel_cell`;
  - the exit_owners rate test.
- Decisions 0118, 0146 and 0211 carry "Amended by" notes.

`make eval` on this branch (main fa5b844d6 at the time, without T-3587's probes sidecars):
- The report line "Left out of the every-suite rate (M9's exit row) ... (decision 0227)" lists 32,150 cells:
  F.DIST 31,473, T.DIST 571, IRR 29, T.INV.2T 36, F.INV 19, RATE 4, TINV 5 and T.INV 13.
- T.DIST meets the every-suite rate, and M9 has 70 of 263 functions below it.
- Gate B is unchanged and still not met.
- `make exit-owners MILESTONE=M9` lists no Phase 1b rate cell.

What remains is:
- t3354 and DSUM cells, cleared by T-3587 (#2698) and T-3636/T-3637 (#2695);
- 100 cells of two probe sweeps that merged since, t3269-fresh-sweep and t3443-basym-sweep, which need owners from
  the steward's exit-owners pass.

