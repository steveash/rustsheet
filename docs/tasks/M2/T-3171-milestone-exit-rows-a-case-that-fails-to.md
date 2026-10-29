---
id: T-3171
title: "Milestone exit rows: a case that fails to load, panics or times out fails its measure (the M0-M2 probe row reads met with 8 unloadable cases)"
milestone: M2
lane: harness
status: todo
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
No milestone exit row in the eval report reads `met` while a case it measures fails to run.
A case whose status is `EngineError`, `Panic` or `Timeout` counts against the measure that
reads its suite or probe row, and the row names it, so the current milestone (docs/milestones.md
"Phase order": the first one whose exit criteria are not met) cannot move past M2 while M2 probe
cases do not load.

## Context
- Found by the planner on 2026-10-29 (`make eval` on the working tree at 91be853a plus task
  files): the "Milestone exit" table reads
  `| M2 | 37/37 n/a | 16280, 1.00000, 0/0 [0] met | 0 [0] met | 0 [8] met | n/a n/a | n/a | met |`.
  The 8 bracketed failures are the eight `probes/t3097-nesting-paren-*` cases that T-3165 owns:
  each fails to load ("Formula is nested too deeply", `status=EngineError`, `checked=0`), so it adds
  0 scored cells and 0 visibly-off cells to the `M0-M2` probe row, and the row reads `met`.
  An unloadable case is the most visible difference there is: Excel shows a value in every cell.
- The same hole applies to every row that counts only scored cells: the gen suites (`gen_met`),
  the ironcalc-calc function-scope rows (`calc_met`) and the probe-milestone rows
  (`probes_met`, `crates/rustsheet-eval/src/milestone_exit.rs` around the `probes_met`
  assignments). Only two measures already handle it: the frozen milestone lists, which need
  `cases_run == cases` (decision 0146 item 5), and the poi M8 subset, which fails on "a crashed or
  errored case holding such a formula" (`crates/rustsheet-eval/src/poi_m8_subset.rs`, decision
  0146 item 2). Reuse that rule and its `CaseStatus` match; do not write a second one.
- `make exit-owners` already lists such cases (the 2026-10-29 18:47 UTC steward run saw the eight
  as unowned before T-3165 was filed), so the owner report and the exit row disagree today.
- Pending cases (no Excel value yet) run crash-only (T-0007). Decide in the decision record
  whether a crashed pending case counts too; the 4 pending M0-M2 probe cases (Excel for the web
  refuses them, docs/status.md "Oracle") must not hold an exit open forever, so a pending case that
  runs without crashing never counts.
- This changes how exits are measured (decisions 0085, 0135, 0146), so it needs a decision record.
  `crates/rustsheet-eval` is not a protected path.

## Acceptance criteria
- [ ] Decision record in `docs/decisions/` (via `make new-decision`): a scored case with status
      `EngineError`, `Panic` or `Timeout` fails the exit measure (gen suite, ironcalc-calc, probe
      row) that reads it; the rule for pending cases; amends decisions 0135 and 0146.
- [ ] The probe-milestone table (`## Probe milestones`) gains a column with the count of such
      cases per row, and the "Failing cases in <row>" lists keep naming them.
- [ ] Unit tests in `milestone_exit.rs`: a probe row with one `EngineError` case and 0 visibly-off
      cells reads `not met`; the same row with the case passing reads `met`; a gen suite with a
      `Timeout` case reads `not met`; a pending case that runs cleanly does not change a verdict.
- [ ] `make eval` on a tree where T-3165 is still open shows the M2 exit row `not met` and the
      row text names the 8 `t3097-nesting-paren-*` cases (or their count with the list below the
      table); every other row's verdict is unchanged on that run (state any that flip in the PR,
      with the case that flips it).
- [ ] The threshold paragraph above the exit table states the new rule.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing the eight cases (T-3165).
- The Phase 1 line and the v1.5 checkpoint's per-function rates: if they share the hole, file a
  follow-up task naming the field.
- Moving the current milestone or editing docs/status.md (the steward does that).
