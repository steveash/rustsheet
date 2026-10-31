---
id: T-3427
title: "R1(c) milestone probe and ironcalc-calc rows count only semantic and outside-envelope cells, with Phase 1b counts, R3 and R7 (decision 0188)"
milestone: M0
lane: harness
status: done
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
- [x] Each milestone's probe row and ironcalc-calc row count visibly-off type, error or text cells, semantic-rule cells
      and numeric cells outside the envelope (`envelope`); the other visibly-off numeric probe and probes-numeric cells
      are printed as "Phase 1b evidence" counts; the bit-level counts stay beside them.
- [x] A cell named in `math-over-excel.tsv` where the engine returns the math within ε passes in R1(c) and in Gate B
      (`gate_b.rs`), and stays an F line of eval/baseline (decision 0188 R3).
- [x] R7: a probe cell that only open M14 tasks own (decision 0143's `owned` rule) leaves the milestone probe row; the
      41 M9 FORECAST.ETS probe cells are the test case.
- [x] Unit tests for a met and an unmet row of each new count; docs/architecture/eval-harness.md describes them.
- [ ] `make check` green; reviewers approve.

## Out of scope
- `exit_owners.py` and `phase1_exit.py` (T-3428); the dashboard (T-3429); math sidecars for eval cells (T-3380).

## Outcome
Done (2026-10-31).
- **One classifier** (`crates/rustsheet-eval/src/r1c.rs`): every visibly-off cell of a run gets one R1(c) class. The
  rules, in order:
  1. `m14_owned`: the cell is in decision 0143's desktop-owned list (R7).
  2. `counts`: the cell is not a number on both sides.
  3. `counts`: a number outside Gate A's envelope, using the shared `accuracy::envelope`. A number with no math value
     is outside too.
  4. `r3_pass`: an R3-marked cell inside the envelope.
  5. `counts`: a number within ε of the math where Excel is more than G = 1e-4 off it with no R3 mark. This is
     R1(c)'s "wrong number from a semantic rule", in R3's own terms: such a gap needs an owner ruling, and without
     one Excel's value is the target.
  6. `phase1b`: anything else.

  An unclassified cell counts.
- **Readers:**
  - Probe rows gain `r1c_off`, `r1c_off_cells`, `phase1b`, `r3_pass` and `m14_owned`.
  - The milestone exit's probe and ironcalc-calc measures (`probe_r1c_off`, `calc_r1c_off`) read the R1(c) count, and
    the table prints the visibly-off total beside it.
  - Gate B attributes an R3 pass as within 15 digits.
  - `phase1b_evidence` gives per-suite class counts, with a report line under the exit table.
  - The R3 list is `eval/accuracy/math-over-excel.tsv`, through `accuracy::lists::load_r3_marks`.
- **Not changed:** M1's frozen probe list still counts every visibly-off cell. Its own measure
  (`milestone_lists`) is outside the probe rows this task names.
- **Measured** with `make eval` on main plus T-3380's math sidecars:
  - **M9 probe row:** 673 visibly off, 622 counted. The 51 left out are the M9 FORECAST.ETS probe cells, all
    M14-owned. The task named 41; the case has 51 visibly-off cells today, and none of them still counts.
  - **ironcalc-calc:** 47 counted and 4 M14-owned. The M9 calc measure is 46 of an allowance of 36, not met, as
    before.
  - **probes:** 910 counted and 51 M14-owned. No probe cell has a math value yet, so none is Phase 1b evidence.
  - **probes-numeric:** 69,757 counted and 584 Phase 1b evidence. T-3358 and its slices add definitions, and with
    them math values for the rest.
- **Tests:** `r1c::tests` (8: each class, the boundaries, the per-suite counts), the probe row split, the exit
  measures met and unmet for both counts, and Gate B with and without an R3 mark.
- **Eval delta:** none. The report changes only; no baseline or gate is touched.
- **Follow-ups:** T-3428 points `exit_owners.py` and `phase1_exit.py` at `r1c_off_cells` and `calc_r1c_off`. Until
  then those scripts read `visibly_off_cells`, which is never fewer, so nothing reads as met early.

