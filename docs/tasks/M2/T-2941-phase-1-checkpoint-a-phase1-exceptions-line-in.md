---
id: T-2941
title: "Phase 1 checkpoint: a phase1-exceptions line in a suite the run skipped is not stale (the 4 poi lines read 'remove their lines' when poi is not fetched)"
milestone: M2
lane: harness
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The eval report calls a line of eval/milestones/phase1-exceptions.tsv stale only when its suite was scored in the run and
the cell is no longer a visibly-off Excel cell. A line whose suite the run skipped (not fetched or not selected) is shown
as not checked, so no agent is told to delete a line that still applies.

## Context
- The 2026-10-27 report (commit ece922e6, "Skipped suites: calipers (not fetched), corpus-bench (not fetched), poi (not
  fetched)") prints, under "Fifteen-digit parity checkpoint": "Warning: 4 listed cells are not scored or not visibly off
  in this run; remove their lines". It then lists poi/FormulaEvalTestData_Copy.xlsx!EverythingTests!G936, H936, J136 and
  K936. All four are lines for T-1075 under decision 0134. They are reported only because poi was not fetched: the
  run never scored them, so the report cannot know whether they are stale.
- Code: `excepted_cells` in crates/rustsheet-eval/src/v15_checkpoint.rs marks every exception key that is not found among
  `results` as stale (`stale_exceptions`), whatever suite the key is in. The markdown warning is written near line 380 of
  the same file. The run's skipped suites are already known: `skipped_suites` in crates/rustsheet-eval/src/run_meta.rs
  (`skipped`). An exception's suite is the first path segment of its case id. docs/architecture/eval-harness.md (the
  phase1-exceptions paragraph, around line 269) says report.md "warns about a listed cell the run does not score or that
  is not visibly off". Update that sentence to the new rule.
- eval/milestones/exit-exceptions.tsv's stale check (`function_scope.rs`) only reads ironcalc-calc, which is vendored and
  always run. It needs no change. Say so in the PR.

## Acceptance criteria
- [ ] A unit test in v15_checkpoint.rs: an exception line whose suite has no case in `results` is not in
      `stale_exceptions`. It is counted (and listed in report.json, for example `unchecked_exceptions`) as not checked
      in this run. The existing test `stale_exceptions == ["probes/a!S!C1", "probes/a!S!Z9"]` still passes.
- [ ] The markdown says "N listed cells were not checked: their suite was skipped" for such lines, and shows the
      "remove their lines" warning only for lines in suites that were scored.
- [ ] `make eval` without poi fetched shows no "remove their lines" warning for the four poi lines.
      docs/architecture/eval-harness.md describes the new rule.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Editing eval/milestones/phase1-exceptions.tsv (a protected path, decision 0044).
- The Phase 1 rate itself: an excepted cell in a skipped suite is not scored, so the rate does not change.
