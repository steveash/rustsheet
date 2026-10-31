---
id: T-3427
title: "R1(c) milestone probe and ironcalc-calc rows count only semantic and outside-envelope cells, with Phase 1b counts, R3 and R7 (decision 0188)"
milestone: M0
lane: harness
status: in-progress
priority: P0
size: M
tier: opus
depends_on: [T-3359]
---

## Goal
Each milestone's probe row and ironcalc-calc row (decision 0146) measures Phase 1 as decision 0188 R1(c) says: it
counts visibly-off type, error or text cells, semantic-rule cells, and numeric cells outside Gate A's envelope; the other
visibly-off numeric probe and probes-numeric cells are reported as Phase 1b evidence; R3 cells pass; R7 applies.

## Context
- Split out of T-3359 (criteria 2 to 4 of that task), which landed Gate B (`crates/rustsheet-eval/src/gate_b.rs`).
- Code: `milestone_exit.rs` and `milestone_exit/` (decision 0146's rows, `probe_*` and `calc_*` counts),
  `probe_milestones.rs`, `phase1_exceptions.rs` (decision 0143's `owned` rule), `accuracy/envelope.rs` (`envelope`,
  T-3356), `eval/accuracy/math-over-excel.tsv` (R3 lines naming eval cells).
- Until T-3380 attaches math values to eval cells, a visibly-off numeric cell has no math and counts as outside the
  envelope (decision 0188 R1(c)), so nothing reads as met early.

## Acceptance criteria
- [ ] Each milestone's probe row and ironcalc-calc row count visibly-off type, error or text cells, semantic-rule cells
      and numeric cells outside the envelope (`envelope`); the other visibly-off numeric probe and probes-numeric cells
      are printed as "Phase 1b evidence" counts; the bit-level counts stay beside them.
- [ ] A cell named in `math-over-excel.tsv` where the engine returns the math within ε passes in R1(c) and in Gate B
      (`gate_b.rs`), and stays an F line of eval/baseline (decision 0188 R3).
- [ ] R7: a probe cell that only open M14 tasks own (decision 0143's `owned` rule) leaves the milestone probe row; the
      41 M9 FORECAST.ETS probe cells are the test case.
- [ ] Unit tests for a met and an unmet row of each new count; docs/architecture/eval-harness.md describes them.
- [ ] `make check` green; reviewers approve.

## Out of scope
- `exit_owners.py` and `phase1_exit.py` (T-3428); the dashboard (T-3429); math sidecars for eval cells (T-3380).
