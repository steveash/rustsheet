---
id: T-2909
title: "Exit-owner report: a task that names a corpus-bench case by its directory (corpus-bench/53062) owns it"
milestone: M2
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: [T-2880]
---

## Goal
`scripts/exit_owners.py` (decision 0096) recognises a corpus-bench case named the way tasks write it, by its directory
(`corpus-bench/53062`). `make unowned-off` and `make exit-owners` then stop listing owned corpus-bench cells as unowned.

## Context
- Naming note: this file never spells out a corpus-bench input file name, because under today's rule a task that
  writes one on a line with no A1-shaped token owns every cell of that case.
- Scope: corpus-bench only. The shared-stem leak in calipers (all 171 cases share one file name, so T-2704 and T-2708
  over-claim) is T-2880's. This task depends on T-2880 and amends decision 0096's ownership rule as T-2880 leaves it.
- Cause: `ExitCell.stem` is the last path segment of the case id. corpus-bench case ids are
  `corpus-bench/<n>/1_<n>_input.xlsx`, so the stem is the input file name, while tasks name the case as `corpus-bench/<n>`.
  T-2636 (M9) says "corpus-bench/53062 (`formamspnc (7)`, AE6:AQ6 and the cells downstream; the case is a whole-case
  `F *` line ...)" and never writes that file name. So `make unowned-off` on 179909a8 lists 220 cells of
  case 53062's input file as unowned, the largest group of its 887 lines.
- T-2636's line also names `AE6:AQ6`, so after this fix it owns only those cells (the span only once T-2731 lands).
  The Outcome says how many of the 220 are still unowned. The gap-triager gives those an owner, for example by adding
  a whole-case line to T-2636.
- Check how the other open corpus-bench tasks (T-0478 and the M11 tasks) write their case names.
- Rule text: docs/decisions/0096-exit-owner-report-for-milestone-exit-cells.md, "Ownership rule". scripts/ is a
  protected path, so the PR needs a decision record (`make new-decision`) that amends that rule.
- Tests: scripts/test_exit_owners.py.

## Acceptance criteria
- [ ] Test: a line naming `corpus-bench/53062` and no A1-shaped token owns every cell of
      case 53062. A line naming `corpus-bench/5306` does not. A line naming
      `corpus-bench/53062` beside `AE6` owns AE6 of that case and not F6.
- [ ] A task that writes the full case id keeps owning it; every existing test passes unchanged.
- [ ] The Outcome gives before and after counts of `make unowned-off`, and of `make exit-owners` for M11 (and M2 once
      T-2822 lands), on a `make eval` with corpus-bench fetched (`rustsheet-eval fetch --suite corpus-bench`).
- [ ] Decision record amending decision 0096 added; `make check` green; reviewers approve.

## Out of scope
- calipers and any other shared-stem suite (T-2880).
- The `.case.json` suffix and A1 spans (T-2731).
- Filing owners for the cells the fix uncovers (the gap-triager does).

## Outcome
- `scripts/exit_owners.py`: a case whose stem is its own and whose id is below the suite is also named by its
  directory path (`corpus-bench/53062`) when no other case is under that path; a shorter path (`corpus-bench/5306`)
  names nothing. A name may end a sentence (`corpus-bench/55049.`, as T-0484 writes it). Decision 0139 amends 0096's
  ownership rule; tests in `scripts/test_exit_owners.py` (`DirectoryPaths`, 7 tests); the existing tests are unchanged.
- Counts, on `make eval` of origin/main de48145d with corpus-bench fetched (912 cases; 1,508 `unattributed_off` cells,
  220 of them corpus-bench): `make unowned-off` 0 before and 0 after; `make exit-owners` M2, M5 and M6 0 before and
  0 after. The count did not move because the gap-triager had meanwhile added a full-id whole-case line to T-2636.
  With that line cut back to `corpus-bench/53062` (a scratch copy of docs/tasks), `make unowned-off` is 220 before
  this change and 0 after. M11 has no `milestone_exit` row in report.json, so `make exit-owners MILESTONE=M11` stops
  with "no milestone_exit row for M11" both before and after; no M11 count exists.
- Other open corpus-bench tasks: T-0478 and T-0950 write the full id too; T-0484 (`corpus-bench/55049.`) and T-1089
  (`corpus-bench/1726`) write only the directory and now name their cases.
