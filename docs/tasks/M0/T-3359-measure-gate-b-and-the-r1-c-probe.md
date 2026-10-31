---
id: T-3359
title: "Measure Gate B and the R1(c) probe and ironcalc-calc rows in the eval report and progress dashboard, with Phase 1b evidence apart (decision 0188)"
milestone: M0
lane: harness
status: todo
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
- [ ] Gate B: `v15_checkpoint` gains a per-function rate over the attributed cells (decision 0118, its fallback and the
      decisions 0134 and 0143 exceptions) of the gen-* suites, ironcalc-calc, corpus-bench, calipers, poi, smoke and
      fixtures only; the Phase 1 line reads it; a run missing calipers, poi or corpus-bench shows Gate B as not
      measured (`n/a`, never met). The all-suite rate stays beside it as information.
- [ ] R1(c): each milestone's probe row and ironcalc-calc row counts visibly-off type, error or text cells, semantic-rule
      cells, and numeric cells outside Gate A's envelope; the other visibly-off numeric probe and probes-numeric cells
      are reported in their own "Phase 1b evidence" counts, and the bit-level counts stay beside them.
- [ ] Cells in a `MATH-OVER-EXCEL` region where the engine returns the math within ε count as passing in Gate B and
      R1(c), and stay F lines of eval/baseline (decision 0188 R3).
- [ ] R7: a probe cell that only open M14 tasks own (decision 0143's `owned` rule) leaves the milestone probe row, as it
      leaves the Phase 1 rate; the 41 M9 FORECAST.ETS probe cells are the test case.
- [ ] `scripts/exit_owners.py` lists only the cells the new rows count.
- [ ] docs/progress.md's generator shows Gate B, Gate A (once T-3356 lands; `n/a` before) and the Phase 1b evidence
      counts; docs/architecture/eval-harness.md describes the measures.
- [ ] Unit tests for a met and an unmet row of each new measure.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The Gate A tables and check (T-3356 to T-3358).
- Any baseline or CI gate change.
