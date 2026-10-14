---
id: T-1568
title: "probe-milestones.tsv loader rejects a cell line whose cell is not a checked cell of its case"
milestone: M2
lane: harness
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A cell line in eval/milestones/probe-milestones.tsv whose `Sheet!A1` is not a checked cell of its case (a key of the case's `.oracle.json`, or of its `expected`) fails the eval run, naming the line. Today such a line is silently ignored.

## Context
The T-1562 review (decision 0060, 1,086 new cell lines) found that `Mapping::check_known` (crates/rustsheet-eval/src/probe_milestones.rs) checks only that a line's case exists. A typo in a cell would leave that cell counted for its case's milestone, and nobody would see it. Extend the check to cells, using the scored cells the run already has, and add a unit test beside `an_unknown_case_names_its_line`.

## Acceptance criteria
- [x] A cell line naming an unchecked cell makes `rustsheet-eval run`/`check` fail with the line number; a unit test covers it.
- [x] The committed mapping still passes (`the_committed_mapping_parses` and `make eval-check`).
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing any line of the mapping.

## Handoff (resumed)
Tried: `Mapping::check_cells` (crates/rustsheet-eval/src/probe_milestones.rs) plus `MappingError::UnknownCell`
and `Rules::check_cells`; `score_milestones` (main.rs) now returns `Result` and calls it, so `run` and `check`
fail with the line number. Unit test `an_unchecked_cell_names_its_line` passes (`cargo test -p rustsheet_eval
probe_milestones`). A case with skipped volatile cells or no scored cells is not checked (the results do not say
which cells were skipped).

Blocked: the committed mapping fails the new check on exactly one line (I scanned all 1228 cell lines with a script
against each case's `check` list, else `expected` plus oracle keys): line 245,
`probes/volatile-invariant.case.json	Calc!A6	M5	RANDBETWEEN`. `Calc!A6` (`=RANDBETWEEN(5.5,5.7)`) is not in
the case's `check` list, so it is never scored and the line does nothing (the oracle has a value, 6.0, for it).
Removing or fixing that line edits `eval/milestones/` (a protected path, scripts/check_governance.py) and needs a
decision record, which the task's "out of scope: changing any line" did not foresee.

Next: `make new-decision TITLE="..."` for the line (decide: drop line 245, or add `Calc!A6` to the case's `check`
list if Excel's 6.0 is stable for RANDBETWEEN(5.5,5.7), which it is not evidenced to be), then run
`make eval-check` and `make check` and get reviewers. Branch `agent/harness/T-1568-milestone-cell-check` carries the code.

## Outcome
- Resumed on opus. The handoff checked cell lines against the run's scored cells, which flagged line 245
  (`probes/volatile-invariant` `Calc!A6`, left out of `check` until T-1318 fixes RANDBETWEEN's bounds) and
  could not check partial runs or cases with skipped volatile cells. The check now follows the Goal's
  definition instead: `load_milestones` (main.rs) gives `Mapping::check_cells` each named case's
  expectation cells (`runner::expectation_cells`: the keys of `expected` and of the oracle sidecar).
  `Calc!A6` has an oracle value, so the committed mapping passes unchanged and no protected path is touched.
- `rustsheet-eval run`/`check` fail before running, naming the line (verified by changing line 245 to
  `Calc!A7` locally: "line 245: probes/volatile-invariant.case.json has no checked cell Calc!A7").
  Unit test `an_unchecked_cell_names_its_line`.
- Eval: 0 delta (`make eval-check`).
