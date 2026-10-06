---
id: T-0160
title: "Year-less date text reads the current year (\"Jul-26\", \"7/1\", \"1-Jul\")"
milestone: M1
lane: functions-date
status: in-progress
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
- [ ] Evidence for each year-less form, recorded in dates.md.
- [ ] The engine reads them in the current year; eval cells are skipped or pinned like NOW/TODAY.
- [ ] `make check` green; reviewers approve.

## Out of scope
Locales other than en-US.
