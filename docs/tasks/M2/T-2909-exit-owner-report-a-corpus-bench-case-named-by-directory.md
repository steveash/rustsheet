---
id: T-2909
title: "Exit-owner report: a task that names a corpus-bench case by its directory (corpus-bench/53062) owns it"
milestone: M2
lane: harness
status: in-progress
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
