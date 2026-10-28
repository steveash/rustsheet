---
id: T-0085
title: "Phase 1 line: a visibly-off cell owned by a desktop-evidence (M14) task counts as filed, not as a miss (decision 0114)"
milestone: M0
lane: harness
status: in-progress
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
The eval's Phase 1 line (report.json `v15_checkpoint.phase1_line`, decision 0118) treats a visibly-off cell whose
only open owner is a desktop-evidence task (milestone M14, decision 0039 rule 3) as a filed failure that does not
count against its function's Phase 1 rate, as decision 0114 already says ("FORECAST.ETS values need desktop Excel
(T-2425); they are M14 work, and their cells are filed tasks under decision 0039 rule 3, so they do not block the
Phase 1 exit"). The report shows those cells in their own column, so nothing is hidden.

## Context
- Today `crates/rustsheet-eval/src/v15_checkpoint.rs` counts every attributed visibly-off cell, so FORECAST.ETS,
  FORECAST.ETS.SEASONALITY, FORECAST.ETS.STAT and FORECAST.ETS.CONFINT (all desktop-only, owned by T-2425 since the
  T-0090 triage) can never reach 99.9%, and the Phase 1 line can never read 506/506 even when every other function
  is done. At the 2026-10-27 eval they need 178, 161, 65 and 58 cells.
- The only exemption mechanism today is the catalog's `v15: exempt` (decision 0094, evidence floor only); it does
  not cover cells.
- The owner rule exists: scripts/exit_owners.py `owned` (a task names the case stem and the cell's A1 name, or the
  stem on a line with no A1 token). The eval is Rust and must not shell out to Python: either port the rule (one
  definition, decision record if it moves), or have `make eval`'s caller pass a list of desktop-owned cells the way
  eval/milestones/exit-exceptions.tsv passes exit exceptions (decision 0087). Prefer the file: a generated
  `eval/milestones/desktop-owned.tsv` written by a script from the open M14 tasks, checked by `make check` to be up
  to date, read by the eval.
- Directive 2 applies: this must not weaken any other gate. The CI no-regression gate and the baselines are
  untouched; only the Phase 1 rate's denominator handling of these cells changes, and only for cells an open M14
  task owns.

## Acceptance criteria
- [ ] A decision record (`make new-decision`) states the rule: a visibly-off cell owned only by an open M14 task is
      reported in a `desktop_owned` count per function and is left out of that function's Phase 1 rate cells and
      visibly-off count; cells owned by any non-M14 open task are unaffected.
- [ ] The eval implements it with a unit test (a function whose only visibly-off cells are M14-owned meets the
      rate; one with a non-M14-owned cell does not), and report.md shows the new column.
- [ ] One definition of ownership (directive 3): the generator of the desktop-owned list calls
      `scripts/exit_owners.py`'s `owned` (or the eval ports it and exit_owners.py is changed to use the port), and a
      test pins both to the same cases.
- [ ] On a full `make eval`, the four FORECAST.ETS functions meet the Phase 1 rate if and only if all their
      visibly-off cells are T-2425's; no other function's verdict changes.
- [ ] `make check` green (the generated list is up to date); reviewers approve.

## Out of scope
- Implementing FORECAST.ETS (T-2425).
- Any exemption for cells whose owner is not an M14 task.
