---
id: T-2259
title: "Report milestone exit cells that no open task of the milestone owns (decision 0085 item 3)"
milestone: M5
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
After `make eval`, one command lists every failing cell of a milestone's exit criteria that no open
(todo, in-progress or blocked) task of that milestone names, so the steward and the planner see
unowned exit gaps without a hand search of `docs/tasks/`.

## Context
- Decision 0085 item 3: a failing cell of an M5 exit criterion is owned by a task with milestone M5,
  so the pick order reaches it while M5 is current. Nothing checks this today; the planner greps by
  hand.
- The hand check on 2026-10-21 (steward run, eval/runs/latest/report.md) found 13 failing exit cells
  whose only owner was a done task: T-2134 (done) handed some of its cells to T-2240 in its Outcome
  but not `TRIGONOMETRIC.xlsx` COTH_CSCH_SECH!C51 and C55, nor 11 gen-m5 cells of its gap-triage list
  (e.g. gen-m5/1d22a002d4f380b3 Calc!A107 `CSCH(PI()/4)`, gen-m5/2abdc1524e5aa685 Calc!A351
  `SINH(PI()/4)`). The planner added them to T-2240 by hand.
- All inputs are already in `eval/runs/latest/report.json` (written by `make eval`):
  - `milestone_exit` (the criteria per milestone);
  - `cases[]` with `suite`, `id` and failing `cells[]` (`sheet`, `cell`) for the gen suite
    (`gen-m5` for M5, decision 0085 item 5);
  - `function_scope.rows[]` (`milestone`, `failing`, `excepted`) for ironcalc-calc (item 1; the
    excepted cells are owned through `eval/milestones/exit-exceptions.tsv` and are left out);
  - `probe_milestones[]` (`milestone`, `failing_cases`) for probes (decision 0051); the failing
    cells of each listed case come from `cases[]`.
- Ownership rule (what a task file must contain to own a cell): its front matter has the milestone
  and an open status, and its text contains the case's file stem (gen-m5: the 16-hex id; ironcalc:
  the workbook file name such as `TRIGONOMETRIC.xlsx`; probes: the case name such as
  `index-reference-args`) and either the cell's A1 name as a whole word (`A107`, `C51`; a sheet
  prefix is optional) or no A1 name of that case at all (a task that names only the case owns all
  of it, as T-2167 owns `t2201-mround-quotient-rounding`). Task files are read with
  `scripts/tasks.py`'s parser (`front_matter`); do not write a second one.
- `scripts/` and `Makefile` are protected paths (`scripts/check_governance.py`), so the PR adds a
  decision record (`make new-decision`), hence tier opus.

## Acceptance criteria
- [x] `scripts/exit_owners.py --milestone M5` (and a `make exit-owners MILESTONE=M5` target) prints
      each unowned failing exit cell as `criterion<TAB>case!Sheet!A1<TAB>formula`, then a count line,
      and exits 0; `--strict` exits 1 when the count is non-zero. It reads only report.json,
      `docs/tasks/` and `eval/milestones/exit-exceptions.tsv`; it fails with a clear message when
      report.json is missing or has no `milestone_exit` row for the milestone.
- [x] `scripts/test_exit_owners.py` covers: a cell named by an open M5 task is owned; a cell named
      only by a done task, or only by an open M6 task, is unowned; a case named with no cell list
      owns all its cells; an excepted ironcalc-calc cell is not listed; `A1` does not match `A10`.
      The test runs under `make hygiene` like the other `scripts/test_*.py`.
- [x] On the 2026-10-21 report with the T-2240 edit of the same day, `make exit-owners MILESTONE=M5`
      lists 0 cells; with that edit reverted in a scratch copy it lists the 13 cells named above
      (shown in the PR description).
- [x] A decision record covers the protected-path change and states the ownership rule above.
- [x] `make check` green; reviewers approve.

## Out of scope
- Making the check a CI gate or a `make check` step (a follow-up decision, if wanted).
- Changing the steward skill (`.claude/`) to run it; file a follow-up task.
- The coverage criterion (it is per function and already reported as met or not).

## Outcome
`scripts/exit_owners.py` and `make exit-owners MILESTONE=M5 [STRICT=1]` list failing exit cells with
no open owner of the milestone (decision 0096). Two refinements of the ownership rule above, both
recorded in the decision: a spec case's stem drops `.case.json` (report ids are
`probes/<name>.case.json`), and "names the case and no cell" is judged per line (a line naming the
stem with no A1-shaped token), because report.json keeps only failing cells: under the literal rule,
T-2240 (which named other `TRIGONOMETRIC.xlsx` cells that pass now) hid C51 and C55.
On this branch, cut from `main` at 9562cd69: 0 unowned M5 exit cells. On that report with
the 13 cells re-injected as failing and T-2240 at its pre-edit text (status todo, T-2259 done): 13
listed, exactly the cells in Context; with T-2240's 2026-10-21 edit: 0.
It reads report.json and `docs/tasks/` only: the ironcalc-calc exceptions are the report's
`excepted` cells, which `make eval` already takes from `eval/milestones/exit-exceptions.tsv`, so the
script does not re-implement their milestone rule.
Eval delta: none (scripts only). Follow-up: T-2293 (steward and planner run it; `.claude/` change).
