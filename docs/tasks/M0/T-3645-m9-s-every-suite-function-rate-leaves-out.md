---
id: T-3645
title: "M9's every-suite function rate leaves out decision 0188 R1(c)'s Phase 1b cells (owner ruling)"
milestone: M0
lane: harness
status: todo
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
- [ ] The every-suite rate leaves out Phase 1b cells and Gate B still counts them (unit test
      `phase1b_cells_leave_the_every_suite_rate_but_gate_b_counts_them`).
- [ ] `exit_owners.py` drops Phase 1b rate cells and keeps M14-owned ones (`test_r1c_left_out_cells_leave_the_rows_and_the_rate`).
- [ ] On a `make eval` run, `make exit-owners MILESTONE=M9` lists no Phase 1b cell, and the report names the cells
      left out per function.
- [ ] `make check` green; reviewers approve (architecture and security for the protected scripts/ path).

## Out of scope
- Gate B, and the visibly-off cells owned only by deferred tasks in `make phase1-exit`'s `owners` line (a separate
  question).
