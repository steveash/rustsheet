---
id: T-2880
title: "Exit-owner report: a file stem shared by several cases (every calipers workbook, gen-m9 hashes copied into probes/t2429 and t2436) must not own all of them"
milestone: M7
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: [T-2731]
---

## Goal
`scripts/exit_owners.py` (decision 0096) counts a cell as owned only when an open task names *its* case. Today a
case is identified by its file stem alone, and 70 stems are shared by 601 cases of one report, so a task that names
one of those cases also owns the same A1 cell (or, on a line with no A1 token, every failing cell) of all the others.
The steward's and gap-triager's `make exit-owners` and `--unattributed` runs then hide unowned failures, which breaks
the Phase 1 rule that every remaining failure has a filed task (docs/milestones.md, decision 0118 item 6).

## Context
- `ExitCell.stem` is `case.rsplit("/", 1)[-1].removesuffix(".case.json")`; `owned` checks `names(text, stem,
  STEM_WORD)` and then the A1 name or the per-line whole-case rule.
- Shared stems on a `make eval` report of origin/main 9af4db8c with calipers fetched (planner run, 2026-10-27):
  - **calipers**: all 171 cases (eval/suites/calipers/suite.toml includes only that file name) have the same file stem: the workbook file name `golden` with the `.xlsx`
    extension. Any open task that writes that file name on a line with no A1 token owns *every* failing calipers
    cell, and one that writes it beside a cell name owns that cell in every calipers workbook. Tasks name calipers
    cases by their full id (`calipers/pv_goldens/pv23_008/...`) or their directory (`pv26_020`), see T-2867,
    T-2704, T-2708, T-2461.
  - **gen-m9 hashes copied into probes**: 69 gen-m9 stems also exist as `probes/t2429/<hash>` and/or
    `probes/t2436/<hash>` cases (the probe copies rerun a gen-m9 formula in a new layout, so their failing cells
    differ). A task that names `gen-m9/<hash>` with a cell owns the probe copy's cell of the same name.
  - Measured false ownership in `--unattributed` today: 6 visibly-off cells, owned only by a task that names
    neither their directory nor their suite: 4 calipers cells in pv26_011 and pv26_2_011 (owned by T-2704, T-2867),
    and 2 probes/t2436 cells (owned by T-2766 and T-2460, which name the gen-m9 hash). Reproduce with a script that,
    for each `unattributed_off` cell whose stem is shared, prints the open tasks that own it but do not contain the
    case's parent directory segment. After T-2731 lets `<stem>.case.json` name a case, the gen-m9 to probes leak
    grows: T-2731's 1,866 `gen-m9/<hash>.case.json (n): cells` lines would also own the probe copies.
- This task depends on T-2731 because both change `names`/`owned` in scripts/exit_owners.py and amend decision
  0096's ownership rule; doing them in order avoids a conflicting second rule change.
- Decision 0096 "Ownership rule" says the stem is "the last segment of the case id"; scripts/ is a protected path,
  so the change needs a decision record (`make new-decision`) amending it. Tests: scripts/test_exit_owners.py.

## Acceptance criteria
- [x] When a stem is shared by more than one case of the report, a task names one of those cases only with a longer
      suffix of the case id that is unique in the report (for example `pv26_011/<file>` or `t2436/<hash>`, or the
      full case id), or by the bare directory segment for calipers (`pv26_011`, the form open tasks already use).
      The decision record states the exact rule, including whether a bare `<hash>` still names its gen-m9 case,
      and tests cover each form.
- [x] A stem that only one case has keeps today's rule; every existing test in scripts/test_exit_owners.py passes
      unchanged.
- [x] A test builds a report with two calipers cases of the same file name and a task that names one by its
      directory and a cell: the other case's cell of that name is unowned. A second test does the same for
      `gen-m9/<hash>` against `probes/t2436/<hash>`. A third: a line that writes only the shared file name and no
      cell owns no case.
- [x] The Outcome gives `make exit-owners MILESTONE=M5`, `MILESTONE=M6`, `MILESTONE=M2` (once T-2822 lands) and
      `scripts/exit_owners.py --unattributed` counts before and after, on a report that includes calipers
      (`target/release/rustsheet-eval fetch --suite calipers` first). Each newly listed cell gets an owner in the
      same PR (a line in the open task that should fix it) or is listed in the Outcome for the steward; past 10 newly
      listed cells, the owner lines go to a follow-up task instead.
- [x] Decision record amending decision 0096 added; `make check` green; reviewers approve.

## Out of scope
- `<stem>.case.json` naming and A1 spans (T-2731).
- Renaming calipers cases or the probe copies, or changing how the eval harness forms case ids.
- Rewriting existing task files beyond adding owners for newly listed cells.

## Outcome
- `scripts/exit_owners.py` names a case through `case_names` (decision 0138, amending 0096/0136): a stem only one
  case of the report has still names it; a shared stem names none of them, and such a case is named by any suffix
  of its id of two segments or more that is unique in the report (`pv26_011/golden.xlsx`, `t2436/<hash>`,
  `gen-m9/<hash>`, the full id) or by its directory below the suite when that directory holds only it
  (`pv26_011`). A bare gen-m9 `<hash>` shared with a probe copy no longer names the gen-m9 case. The names replace
  the stem in all three forms (A1 anywhere, colon span on the case's line, whole-case line).
- Tests: `SharedStems` in scripts/test_exit_owners.py (names, calipers directory + cell, gen-m9 hash vs its
  t2436 copy, a shared stem alone owning nothing); the 18 existing tests pass unchanged.
- Counts on `make eval` of origin/main 5cadaf05 with calipers fetched (3,328 cases; 70 shared stems, 603 cases),
  before → after: `make exit-owners MILESTONE=M2` 0 → 0, `M5` 0 → 0, `M6` 0 → 0, `--unattributed` 0 → 0.
  Of the 271 unattributed visibly-off cells in shared-stem cases, 57 false owner links (task, cell) are gone, and
  no cell is left without an owner: the 6 cells the Context measured now have proper owners (T-2663 names
  pv26_011 and pv26_2_011; T-2940 names the `probes/t2436/<hash>` copies), filed since this task was written.
  No newly listed cells, so no owner lines were added.
- Eval delta: none (no engine change).
