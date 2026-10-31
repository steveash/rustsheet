---
id: T-3359
title: "Measure Gate B and the R1(c) probe and ironcalc-calc rows in the eval report and progress dashboard, with Phase 1b evidence apart (decision 0188)"
milestone: M0
lane: harness
status: done
priority: P0
size: M
tier: opus
depends_on: [T-3355, T-3356]
---

## Goal
The eval report and the progress dashboard measure Phase 1 as decision 0188 R1 defines it: Gate B per function on
user-representative evidence, milestone probe and ironcalc-calc rows counting only R1(c)'s cells, the other
visibly-off numeric probe cells reported as Phase 1b evidence, and R7 applied to the probe rows.

## Context
- Owner's 2026-10-31 assessment (decision 0188): 97% of the visibly-off numbers are fit-lab research sweeps in
  probes-numeric; on gen-*, ironcalc-calc, corpus-bench, smoke and fixtures alone 447 of 506 functions are at ≥ 99.9%
  within 15 digits, against 422 at HEAD with every suite counted.
- Code: `crates/rustsheet-eval/src/v15_checkpoint.rs` (decision 0118's measure, `meets_rate`, the Phase 1 line),
  `milestone_exit.rs` and `milestone_exit/` (decision 0146's rows), `phase1_exceptions.rs`, the desktop-owned list of
  decision 0143 (`scripts/desktop_owned.py`), `scripts/exit_owners.py`, and the dashboard generator
  (tools/src/rustsheet_tools/progress_report/, SPEC-0022, the `progress-report` skill).
- The math reference a probe cell needs for R1(c)'s envelope comes through the Gate A spec (T-3355). Until a cell has
  one, a visibly-off numeric cell counts as failing the envelope (decision 0188 R1(c)), so nothing reads as met early.
- SPEC-0029 (T-3355): use `envelope` (crates/rustsheet-eval/src/accuracy.rs, T-3356) for R1(c), the eval-cell lines
  of `eval/accuracy/math-over-excel.tsv` for R3's cells, `eval/runs/accuracy.json` for the Gate A line, and the math
  values T-3380 attaches to eval cells (until T-3380 lands, every visibly-off numeric cell counts as outside).

## Acceptance criteria
- [x] Gate B: `v15_checkpoint` gains a per-function rate over the attributed cells (decision 0118, its fallback and the
      decisions 0134 and 0143 exceptions) of the gen-* suites, ironcalc-calc, corpus-bench, calipers, poi, smoke and
      fixtures only; the Phase 1 line reads it; a run missing calipers, poi or corpus-bench shows Gate B as not
      measured (`n/a`, never met). The all-suite rate stays beside it as information.
Split (size L): this task delivers Gate B (criterion 1, its tests and its eval-harness.md description); criteria 2 to 4
are T-3427, criterion 5 is T-3428, criterion 6's dashboard is T-3429.

- [ ] (T-3427) R1(c): each milestone's probe row and ironcalc-calc row counts visibly-off type, error or text cells, semantic-rule
      cells, and numeric cells outside Gate A's envelope; the other visibly-off numeric probe and probes-numeric cells
      are reported in their own "Phase 1b evidence" counts, and the bit-level counts stay beside them.
- [ ] (T-3427) Cells in a `MATH-OVER-EXCEL` region where the engine returns the math within ε count as passing in Gate B and
      R1(c), and stay F lines of eval/baseline (decision 0188 R3).
- [ ] (T-3427) R7: a probe cell that only open M14 tasks own (decision 0143's `owned` rule) leaves the milestone probe row, as it
      leaves the Phase 1 rate; the 41 M9 FORECAST.ETS probe cells are the test case.
- [ ] (T-3428) `scripts/exit_owners.py` lists only the cells the new rows count.
- [ ] (T-3429) docs/progress.md's generator shows Gate B, Gate A (once T-3356 lands; `n/a` before) and the Phase 1b evidence
      counts; docs/architecture/eval-harness.md describes the measures.
- [x] Unit tests for a met and an unmet row of each new measure (Gate B; T-3427 owns its own).
- [x] `make check` green; reviewers approve.

## Out of scope
- The Gate A tables and check (T-3356 to T-3358).
- Any baseline or CI gate change.

## Outcome
- **Split** (the task was L): this PR delivers Gate B. R1(c), R3 and R7 rows are T-3427; exit_owners and phase1-exit are
  T-3428; the dashboard is T-3429 (each filed with the context of this task).
- **What changed.**
  - `crates/rustsheet-eval/src/gate_b.rs` rebuilds decision 0118's per-function counts from the gen-*, ironcalc-calc,
    corpus-bench, calipers, poi, smoke and fixtures cells only, through `report::attribute_cell`, which is now the one place
    a cell is attributed (RunReport::build uses it too).
  - Those suites' decision 0134 and 0143 cells are left out: `excepted_cells` takes a suite filter.
  - The rate step is `v15_checkpoint::rate_count`, shared with the all-suite rows. A function meets Gate B when it is
    covered (decision 0094, every suite) and ≥ 99.9% of its counted cells are within 15 digits. A function with no
    counted cell has no rate and does not meet it.
  - Without calipers, poi or corpus-bench, Gate B is `n/a` (`measured: false`, `missing`), never met.
  - `v15_checkpoint.gate_b` holds the result and `phase1_met` is `gate_b.met`, so `make phase1-exit`'s function-rate
    verdict follows Gate B. Until T-3428 it reads `not met` rather than `n/a` when Gate B is unmeasured, which fails safe.
  - The Phase 1 line prints Gate B with the every-suite `at_rate` beside it as information. The report lists up to 30
    functions below Gate B's rate.
  - v15_checkpoint's tests moved unchanged to `v15_checkpoint/tests.rs` (1000-line limit); one assertion now expects the
    new Phase 1 line.
  - docs/architecture/eval-harness.md describes Gate B.
- **Tests.** gate_b/tests.rs covers:
  - met on user-representative cells while probes-numeric is off;
  - unmet above one in a thousand;
  - not measured without poi or corpus-bench;
  - uncovered and unevidenced functions;
  - the counted suites;
  - listed cells leaving the rate only from counted suites.
- **Eval delta:** none; no scoring, baseline or gate change. Report and JSON fields only.
