---
id: T-2731
title: "Exit-owner report: a task's 'case.json (n): cells' lines and A1:B2 spans own their cells (1,866 cell lines in 74 open tasks are invisible to it)"
milestone: M7
lane: harness
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`scripts/exit_owners.py` (decision 0096) counts a cell as owned when an open task of the milestone names it in the
two forms gap-triage tasks use: the `Cells:` list line `- gen-m9/<hash>.case.json (2): Calc!A145, A328`, and a span
such as `A5:D5`. Today the report lists those cells as unowned. The planner then files duplicate owners, or a
milestone exit looks blocked when it is not.

## Context
Found by the gap triage on 2026-10-25, which used `exit_owners.owned` to find unowned failing cells.
- **The stem is followed by `.case.json`.** `names(text, stem, STEM_WORD)` requires that no `[\w.-]` character
  follows the stem, so `146b46e74f2dac3b.case.json` does not name the stem `146b46e74f2dac3b`. That is the format
  of every "Owned gen-m9 cells ... Cells:" block that the triage template writes. Reproduction:
  `owned(ExitCell('gen-m9','gen-m9/146b46e74f2dac3b.case.json','Calc!A328',''), [T-2427's text])` is False, though
  T-2427 lists `- gen-m9/146b46e74f2dac3b.case.json (2): Calc!A145, A328`. The cell `05d44f6dd6cfc567 A4` counts as
  owned only because an example line also names it without the suffix.
  Scale: 74 open tasks hold 1,866 such lines (70 tasks in M9, 2 in M8, 1 in M7, 1 in M14).
  `grep -cE '^- [a-z0-9-]+/[^ ]+\.case\.json \([0-9]+\): '` over the open tasks counts them.
  M5 and M6 report 0 unowned today, so no current exit is wrong yet. The M7, M8 and M9 exits will be.
- **Spans.** `A1_TOKEN` matches each end of `A5:D5` but not the cells between them. T-2519 owns
  "probes/t2454-offset-one-cell-array-args A5:D5, A9:D9", yet Sheet1!B5 (`COLUMNS(OFFSET(B9:C12,5,1,2,TAKE(H2:H4,1)+0))`,
  Excel 3, ours 1, visibly off) is reported unowned. Tasks also write hyphen spans: T-2460 "P!D49-D65", T-2640
  "C1-C21, D1-I12". A hyphen also appears in prose and in case names, so whether it counts as a span is a rule
  change for the decision record. Colon spans are an A1 reference by Excel's own syntax.
- Rule text: docs/decisions/0096-exit-owner-report-for-milestone-exit-cells.md ("Ownership rule"). scripts/ is a
  protected path, so the change needs a decision record (`make new-decision`) that amends 0096's ownership rule.
  Tests: scripts/test_exit_owners.py.

## Acceptance criteria
- [ ] A stem followed by `.case.json` names its case (the T-2427 reproduction above is owned). A stem that is only
      the prefix of a longer name still does not (`abc` does not name `abcd.case.json`).
- [ ] A colon span `A5:D5` (with or without `Sheet!`) owns every cell in its rectangle, so T-2519's B5 is owned. Hyphen
      spans are owned or explicitly not, as the new decision record states, with a test either way.
- [ ] The per-line "stem with no A1 token owns the whole case" rule is unchanged: a `Cells:` line with cells owns only
      those cells, and the existing tests pass unchanged.
- [ ] Before/after counts of `make exit-owners MILESTONE=M5` and `MILESTONE=M6` (and of the same check run at M7, M8
      and M9 with an ad hoc report of all suites, as the triage did) are in the Outcome.
- [ ] Decision record amending 0096 added; `make check` green; reviewers approve.

## Out of scope
- Rewriting existing task files into a new format.
- Companion cells that no task names. The probes/t2436 ROWS/COLUMNS cells in columns T and U are an example: 130
  failing companions of owned column-A cells, visibly off. They need a bookkeeping task, not a tool rule.
