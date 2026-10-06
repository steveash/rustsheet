---
id: T-0160
title: "Year-less date text reads the current year (\"Jul-26\", \"7/1\", \"1-Jul\")"
milestone: M1
lane: functions-date
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Date text without a year is read in the current year, as Excel does: `DATEVALUE("Jul-26")` is
26 July of this year (not July 2026).

## Context
A first fill of probes/date-text-grammar on 2026-10-05 gave `DATEVALUE("Jul-26")` =
`VALUE("Jul-26")` = 46229 (2026-07-26); the cell was dropped from the case because it depends on
the clock (docs/excel-semantics/dates.md, "The en-US date-text grammar", Current year). The
engine rejects year-less forms (#VALUE!) in `formatter::date_text::parse_date` (T-0024). The
parser has no clock today; the harness pins one (`volatile.rs`, METADATA `NOW`), and cells
reading the clock must count as volatile. Design the clock access first (an engine-level
"today" passed in, not I/O from the parser; CLAUDE.md directive 5), then probe the forms
("Jul-26", "7/1", "1-Jul", "July 1", "Jul-45") in a workbook whose expected values are
computed from a pinned date.

## Acceptance criteria
- [x] Evidence for each year-less form, recorded in dates.md.
- [x] Eval cells are clock-independent (MONTH/DAY, or forms that can never be a day), so nothing needs pinning.
- [x] The engine reads the two-digit-year forms (clock-free); the current-year reading is split to
      T-0167 (clock access design).
- [x] `make check` green; reviewers approve.

## Out of scope
Locales other than en-US.

## Outcome
Probed 20 year-less forms in Excel for the web (2026-10-06). A one- or two-digit number after a month
is a day of the current year when it can be one this year ("Jul-26", "7/1", "1-Jul", "July 1", "Jul/26",
"7-1", "12/31"), otherwise a two-digit year (<30 is 20xx): "Jul-45" is 1945-07-01, "Jul-32" 1932,
"Jul-0" 2000, "Jul-99" 1999, "2/30" 1930-02-01, and in 2026 "2/29" and "Feb-29" are 2029-02-01.
New case probes/date-text-yearless (clock-independent cells only; oracle-filled). Implemented the
clock-free half in `parse_month_year`; the current-year half needs engine clock access and is split
to T-0167. Eval: probes fixed 0, new-coverage failures 36 (B1:D12, the current-year forms, T-0167), 0
regressions; the 10 two-digit-year cells pass.
