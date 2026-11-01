---
id: T-0097
title: "M10 milestone exit row: gen-m10 displayed text and value cells, M10 probe value and text cells (decision 0146 item 4)"
milestone: M0
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: [T-0089, T-0093]
---

## Goal
The eval report has an M10 `milestone_exit` row measured as decision 0146 item 4 defines, and
`scripts/exit_owners.py --milestone M10` lists the failing displayed-text items and visibly-off value cells that
keep M10 from exiting, as `case!Sheet!A1`.

## Context
- Decision 0146 (T-0088, PR #2144) item 4, required measures:
  - (a) gen-m10's displayed-text items, at least 10,000, with 0 failing (a text item has no 15-digit reading, so
    the exit's "100%" is exact), and gen-m10's value cells with 0 visibly off. On 2026-10-28: 10,400 items,
    76 failing; 1,690 value cells, 0 failing. probes' text items are not part of the suite;
  - (b) the M10 probe row with 0 visibly-off value cells (50 on 2026-10-28) and 0 failing text items (T-0093's
    counts);
  - coverage and ironcalc-calc `n/a` (TEXT, FIXED and DOLLAR are catalog M5; a `probes/catalog/*` case of them
    counts toward M5, decision 0051).
- The suite totals are report.json `suites[].text_cells` / `text_passed`; the failing items are each case's
  `text` results with `pass == false` (T-1610). Reuse T-0089's required-suite list and T-0093's probe-row fields;
  score nothing new.
- `scripts/` is protected: amend decision 0146's Consequences (or add a record) in the same PR.

## Acceptance criteria
- [x] report.md and report.json have an M10 row with both measures; unit tests cover a met row, a failing gen-m10
      text item alone, gen-m10 short of 10,000 items, a visibly-off probe value cell alone and a failing probe text
      item alone.
- [x] `python3 scripts/exit_owners.py --milestone M10` runs and lists failing text items and visibly-off value cells
      of both sources; `scripts/test_exit_owners.py` covers each.
- [x] The Outcome note records the row's numbers and the unowned count `make exit-owners MILESTONE=M10` prints.
- [x] `make progress-report` renders the row; `make check` green; reviewers approve.

## Out of scope
- Owning or fixing the 76 gen-m10 text items and the M10 probe cells (functions-text follow-ups).
- Making probes' text items part of gen-m10's measure.

## Outcome
- `milestone_exit` has an M10 row (decision 0146 item 4, Consequences amended). gen-m10 is its suite with no value-cell
  floor and none visibly off; its text items are part of the same measure (`M10_MIN_TEXT` = 10,000, none failing:
  `gen_text_items`, `gen_text_failing`, `gen_text_failing_cells`). The probe measure is now a `ProbeMeasure` (row,
  row and text, or M1's list): M10's needs the probe row's visibly-off cells and failing text items both at 0
  (`probe_text_items`, `probe_text_failing`). Other rows leave the text fields null. report.md prints
  `text <failing> failing of <items>` in both columns, and so does the progress page.
- `scripts/exit_owners.py` lists failing text items as `gen-m10 text` and `probes text`. It also stopped listing
  ironcalc-calc cells for a row without that criterion (a null `calc_met`): before this, M10 and M11 listed the 119
  visibly-off cells of their function-scope rows, which decision 0146 does not require.
- `make eval` on this branch: M10 not met. gen-m10 has 10,400 text items with 61 failing, and 1,690 value cells with
  0 visibly off. The M10 probe row has 34 visibly-off cells and 1 failing text item of 418.
  `make exit-owners MILESTONE=M10` listed 37 unowned cells: 32 gen-m10 text items with T-1736's root cause, and 5
  probes/t3029-b2-calendar-weekday cells that T-3101 covered only by hyphen spans. T-1736 now names the 32 items, and
  T-3101's acceptance line uses colon spans, so 0 are unowned. M11's unowned count drops from 119 to 0.
- Eval delta: none (report-only change).
