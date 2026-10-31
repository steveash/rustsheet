---
id: T-0085
title: "Phase 1 line: a visibly-off cell owned by a desktop-evidence (M14) task counts as filed, not as a miss (decision 0114)"
milestone: M0
lane: harness
status: done
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
- [x] A decision record (`make new-decision`) states the rule: a visibly-off cell owned only by an open M14 task is
      reported in a `desktop_owned` count per function and is left out of that function's Phase 1 rate cells and
      visibly-off count; cells owned by any non-M14 open task are unaffected.
- [x] The eval implements it with a unit test (a function whose only visibly-off cells are M14-owned meets the
      rate; one with a non-M14-owned cell does not), and report.md shows the new column.
- [x] One definition of ownership (directive 3): the generator of the desktop-owned list calls
      `scripts/exit_owners.py`'s `owned` (or the eval ports it and exit_owners.py is changed to use the port), and a
      test pins both to the same cases.
- [x] On a full `make eval`, the four FORECAST.ETS functions meet the Phase 1 rate if and only if all their
      visibly-off cells are T-2425's; no other function's verdict changes.
- [x] `make check` green (the generated list is up to date); reviewers approve.

## Out of scope
- Implementing FORECAST.ETS (T-2425).
- Any exemption for cells whose owner is not an M14 task.

## Outcome
Decision 0143. `scripts/desktop_owned.py` applies exit_owners.py's `owned` rule (one definition; `open_tasks` factored
out of `open_task_texts`) to every cell `F` line of eval/baseline and lists the cells that open M14 tasks own and no other
open task owns; `make eval` writes it to the git-ignored `eval/runs/desktop-owned.tsv` and runs `rustsheet-eval run
--desktop-owned`, which loads it into `Phase1Exceptions` (`load_desktop_owned`). `v15_checkpoint` leaves those
visibly-off cells out of the Phase 1 rate as `desktop_owned` (per function and in total, an "M14-owned, left out" column
and a line under the Phase 1 line in report.md); a cell phase1-exceptions.tsv also lists counts once, as `rate_excepted`.
Generating the list per run instead of committing it (the task's preferred option) avoids a protected-path file that every
task or baseline change would have to regenerate; the decision records why.

Eval on main 81569726 plus this branch (vendored suites; poi and calipers not fetched here): 1064 generated cells (971
T-2425, 41 T-1553, 38 T-2663, 8 T-2704, 6 T-1075); 1000 visibly-off cells left out, 453 of them counted
(FORECAST.ETS.SEASONALITY 175, FORECAST.ETS.STAT 158, FORECAST.ETS 63, FORECAST.ETS.CONFINT 56, F.INV 1). Phase 1 line
unchanged at 408/506 and no function's verdict changed: the four FORECAST.ETS functions still have 2-3 counted visibly-off
cells each (FORECAST.ETS 2/20, CONFINT 2/12, SEASONALITY 3/20, STAT 3/10), ten probe cells that M9 tasks T-0969 and T-0972
own through whole-case tally lines, so they correctly do not meet the rate. Follow-up T-3004 hands those cells to
T-2425. Eval baseline unchanged (no cell moved).
