---
id: T-2909
title: "Exit-owner report: a calipers case is named by its golden directory, not by the file name all of them share; a corpus-bench case by its directory number"
milestone: M2
lane: harness
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`scripts/exit_owners.py` (decision 0096) names xlsx-suite cases the way tasks write them. A task that names one
calipers golden owns only that golden's cells. A task that names a corpus-bench case by its directory
(`corpus-bench/53062`) owns it. `make unowned-off` and `make exit-owners` then stop counting calipers cells as owned
by tasks that never mention them, and stop counting owned corpus-bench cells as unowned.

## Context
- Naming note: every calipers case file has the same name, `golden` with the `.xlsx` extension. This task writes
  it as GOLDEN, and the corpus-bench file of case 53062 as INPUT-53062. Under the current rule, a task that spells
  either name out on a line with no A1-shaped token owns every cell of those cases: the bug itself.
- Reported by the gap-triager in the 2026-10-27 steward snapshot (docs/status.md, "Queues"): "every calipers
  case shares the stem GOLDEN, so a task naming GOLDEN 'owns' every calipers cell. No task covers
  that yet."
- Cause: `ExitCell.stem` is the last path segment of the case id. eval/suites/calipers/suite.toml includes
  `**/` + GOLDEN, so all 171 calipers cases (`calipers/pv_goldens/pv23_007/GOLDEN`,
  `calipers/roundtrip/formulas_nested/GOLDEN`, ...) have the same stem GOLDEN. `owned` checks that
  the stem is named somewhere in the task text, and then checks the A1 name or `names_whole_case`.
- Two leaks this causes today, both reproducible against the open task files:
  - T-2708 (M9) lists `calipers/pv_goldens/pv23_007/GOLDEN (11): Pro Forma Cash Flow Statement!AF13, ... G15,
    ...`. Any failing calipers cell named `G15`, `N13` or `BI15`, in any other golden, counts as owned by T-2708.
  - T-2704 (M14) line 21 names `calipers/roundtrip/formulas_nested/GOLDEN` on a line with no A1-shaped token.
    By `names_whole_case`, that makes T-2704 own every calipers cell. `--unattributed` reads the open tasks of
    every milestone, so every unattributed visibly-off calipers cell is hidden there. On 179909a8 that covers all 4
    of them: pv26_011 and pv26_2_011 `UC market model!B3` and `CD market model!B3`
    (`MID(CELL("filename",A1),...)`). `exit_owners.owned` credits them to T-2704 alone, a task about one-ulp
    products in formulas_nested. Their real owner is presumably T-2663 (desktop-only CELL filename), which the tool
    cannot see because it never spells out GOLDEN.
- The opposite problem, in corpus-bench: case ids are `corpus-bench/<n>/1_<n>_input` plus `.xlsx`, and tasks name them as
  `corpus-bench/<n>`. T-2636 (M9) says "corpus-bench/53062 (`formamspnc (7)`, AE6:AQ6 and the cells downstream; the
  case is a whole-case `F *` line ...)". It never writes INPUT-53062, so `make unowned-off` on 179909a8
  lists 220 cells of `corpus-bench/53062/`INPUT-53062 as unowned. That is the largest group of its 887 lines.
  T-2636's line also names `AE6:AQ6`, so after this fix it owns only those cells (and the span only once T-2731
  lands). The Outcome says how many of the 220 are still unowned. The gap-triager gives those an owner, for
  example by adding a whole-case line to T-2636.
  Check how the other open corpus-bench tasks (T-0478 and the M11 tasks) write their case names.
- Check poi case ids for repeated last segments as well, and name in the decision record which suites are affected.
- Rule text: docs/decisions/0096-exit-owner-report-for-milestone-exit-cells.md, "Ownership rule" ("the last
  segment of the case id"). scripts/ is a protected path, so the PR needs a decision record (`make new-decision`)
  that amends that rule. T-2731 (M7, harness) changes the same functions (`names`, `owned`, the stem boundary)
  and amends the same rule. The two share a lane, so they never run at once. Whichever lands second rebases onto
  the first and amends the rule as the first left it.
- Tests: scripts/test_exit_owners.py.

## Acceptance criteria
- [ ] A case whose last segment is not unique in its suite is named by enough of its path to tell it apart (for
      example `pv23_007/GOLDEN`, or the full case id). The decision record states the rule. A task line
      written the way the calipers tasks already write them (`calipers/pv_goldens/pv23_007/GOLDEN (11): ...`)
      still owns those cells, with no edits to task files.
- [ ] Test: with T-2708's pv23_007 line as the only task text, a failing `G15` of
      `calipers/pv_goldens/pv23_008/GOLDEN` is not owned and `G15` of pv23_007 is.
- [ ] Test: a line naming only `calipers/roundtrip/formulas_nested/GOLDEN` owns that case's cells and no
      cell of another golden.
- [ ] Test: a line naming `corpus-bench/53062` and no A1-shaped token owns every cell of
      `corpus-bench/53062/`INPUT-53062. A line naming `corpus-bench/5306` does not. A line naming
      `corpus-bench/53062` beside `AE6` owns AE6 of that case and not F6.
- [ ] Cases with unique stems (gen hashes, ironcalc-calc file names, probe names) keep today's behavior. The
      existing tests pass unchanged.
- [ ] The Outcome gives before and after counts of `make unowned-off`, and of `make exit-owners` for M2 (once
      T-2822 lands) and M11, on a `make eval` with calipers and corpus-bench fetched (`rustsheet-eval fetch`).
- [ ] Decision record amending decision 0096 added; `make check` green; reviewers approve.

## Out of scope
- Renaming calipers cases or changing the suite's case ids (eval/baseline keys depend on them).
- Filing owners for any calipers cells the fix uncovers. List them in the Outcome; the gap-triager files their
  tasks.
- The `.case.json` suffix and A1 spans (T-2731).
