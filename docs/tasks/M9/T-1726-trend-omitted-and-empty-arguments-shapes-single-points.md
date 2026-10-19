---
id: T-1726
title: "TREND: omitted and empty arguments, shapes, single points, text and error inputs as Excel (T-1620 probes)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
TREND matches Excel for omitted and empty arguments, shapes, single points, text, blanks and error inputs.

## Context
Found by T-1620 (probes/regression-trend, ROWS/COLUMNS/SUM/INDEX of each TREND call in columns L-O; `./target/release/rustsheet-eval show probes/regression-trend.case.json`, excel-graph 2026-10-16). Highlights (ours in parentheses):
- empty arguments: `TREND(A1:A6,,K1:K2)` has 2 rows (`#REF!`); `TREND(A1:A6,B1:B6,,FALSE)` and `,,TRUE` return 6 rows, the fit to the known x's (0 rows; `TREND(A1:A6,,,FALSE)` `#REF!`).
- array constants: `TREND({2,4,5},{1,2,3})` 1 row, 3 columns, first 2.1666666666666665 (`#REF!`); `TREND({2,4,5,4,5,7},{1,2,3,4,5,6},{7,8})` 1x2, first 7.2 (`#REF!`).
- a single point: `TREND(A1:A1,B1:B1)`, `TREND(5,5)` and `TREND(5)` are 2.0, 5.0, 5.0 (`#VALUE!`).
- zero-variance x: `TREND(A1:A6,G1:G6)` 6 rows with first 4.5 (`#NUM!`).
- text, blank or error in a range: `#VALUE!` (a blank known_y `D1:D6` or known_x gives numbers in ours; an error cell gives `#DIV/0!`), a text scalar known_x is `#REF!` (ours `#VALUE!`).
- 2-D known_y (`TREND(A1:B3,H1:I3)`): 3x2 result, first 3.6666666665000003 (`#REF!`).
Group the work by cause; record the rules in docs/excel-semantics/statistics.md.

Overlaps T-1402 (single-value known_y, `#REF!` before `#VALUE!`). Also: an error cell in new_x is `#VALUE!` for that element only (`TREND(A1:A6,B1:B6,F1:F2)` keeps 2 rows, first 3.3428571428571425), unequal sizes are `#REF!`, zero-variance x returns the mean.

Also: a blank new_x makes the whole result `#VALUE!` (ours 2 rows of numbers, trend L39) and a row-shaped new_x gives a row result (`TREND(A1:A6,B1:B6,{7,8})` 1x2, SUM 15.17142857142857; ours 1 column).

## Acceptance criteria
- [ ] The mismatching non-ULP cells of probes/regression-trend pass (or the remaining ones are listed in an Outcome note) and the baseline shrinks.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Probing further inputs (file a probe task instead).
- The ulp cells of probes/regression-trend: they come from the shared least-squares solve, which T-2006 fits (gap
  triage 2026-10-19).
