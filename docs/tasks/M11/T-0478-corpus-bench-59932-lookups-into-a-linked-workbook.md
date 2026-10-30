---
id: T-0478
title: "corpus-bench 59932: lookups into a linked workbook with no cached data, #ERROR! against the file's cached values"
milestone: M11
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The about 18,000 cells of corpus-bench/59932 that look up a linked workbook's sheet with no cached
data stop counting as visibly-off Excel evidence against VLOOKUP, MATCH, MAX and MIN. Either
rustsheet gives Excel's offline answer for them and the expected values are that answer, or the
harness stops scoring cells whose inputs are not in the file. Which of the two is decided in this
task and recorded.

## Context
- Eval report of 2026-10-25 at `a2d5914a` (eval/runs/latest/report.json): the case
  `corpus-bench/59932/1_59932_input.xlsx` has `expected_source` excel-graph, 32,948 checked and
  14,734 passed cells. Its failing cells are marked `from_excel`, so they count as Excel evidence:
  17,933 VLOOKUP cells (VLOOKUP's attributed share within 15 digits is 45.73% on
  docs/progress.md; 17,946 attributed visibly-off cells in all), 70 MAX, 70 MIN and 50 MATCH cells
  attributed to it. Example: `Sort!C12` `=VLOOKUP($B12,'[1]<2010Price'!$A:$BE,13,FALSE)` expects
  35462, rustsheet gives `#ERROR!`. eval/baseline/corpus-bench.txt lists the case as a known
  failure (`F ... *`).
- T-0457 (done) found Excel's offline answer: lookups into a linked sheet with no cached data are
  `#N/A`, before the index check (docs/excel-semantics/references.md "Lookups into a linked sheet
  with no cached data are `#N/A`"). The file's cached values came from the linked workbook while
  it was open, so no engine can reproduce them from the file. T-0430's Outcome names the same
  cells ("the rest are linked-workbook VLOOKUPs, T-0457").
- The engine still gives `#ERROR!` on this workbook, not the `#N/A` of the T-0457 fixture: find
  why (the whole-column area past the array bound, `ExternalArea::blocked_value`, or the sheet
  names such as `<2010Price`; crates/rustsheet-engine/src/external_links.rs).
- Precedent for not scoring: T-0462 (calipers `_xll.` add-in cells are out of scope and not
  scored) and T-0466 (cells that call another workbook's function).
- Found by T-0075's dry run of the dashboard's risks (risk 2).

## Acceptance criteria
- [x] The engine gives Excel's offline `#N/A` for these lookups, or a test shows why it already
      does elsewhere and what differs here.
- [x] A decision, recorded in docs/decisions/ if it changes scoring: cells whose value depends on
      a linked sheet with no cached data are either scored against Excel's offline answer (oracle
      recalculation) or reported as not scored, through the same harness path as T-0462. The
      cached values from the open linked workbook are never the expected value.
- [x] corpus-bench 59932 no longer contributes these cells to VLOOKUP, MATCH, MAX and MIN's
      visibly-off counts in report.json; the eval baseline only shrinks (`make eval-baseline`).
- [x] `make check` green; reviewers approve.

## Out of scope
- Linked sheets whose cache holds data (T-1790, T-0474).
- The three T-0457 fixture cells left to T-0472.

## Outcome
Closed with no code change, because the task's premise was wrong and T-0474 has already fixed the cells. The linked
sheet these lookups read is not uncached. In `xl/externalLinks/externalLink1.xml` of
corpus-bench/59932/1_59932_input.xlsx, `<2010Price` is `sheetId` 8 with 1,128 cached cells (columns
A to BE, rows 1 to 101, column M included). `<2010Cal` (`sheetId` 9) holds 1,900 cached cells. Only the sheets
this workbook does not look up are empty (`sheetId` 1-4, 7, 10-12, 14-16). The `#ERROR!` came from the
whole-column area `$A:$BE` passing the array bound, and T-0474 fixed it: VLOOKUP, HLOOKUP and MATCH now read a
cached linked range in place (`LookupTable::External`).

Evidence: `rustsheet-eval show corpus-bench/59932/1_59932_input.xlsx` at origin/main 397cf8d reports
`status=Pass checked=32948 passed=32948`. eval/baseline/corpus-bench.txt already lists the case as `C`
(complete). The expected values come from the file's cache of the linked sheet. That cache is what Excel reads
offline for a cached linked sheet (docs/excel-semantics/references.md, cached ranges, T-1460), so the
scoring is right as it stands. No decision record and no change to the harness are needed. The no-cache rule
(T-0457, `#N/A`) does not apply here. Eval delta: none (the case already passes on main).
