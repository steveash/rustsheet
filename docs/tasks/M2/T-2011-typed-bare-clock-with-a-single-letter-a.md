---
id: T-2011
title: "Typed bare clock with a single-letter A or P suffix and a trailing dot (10:00 A, 1/2/2020 10:00 A.) is a number"
milestone: M2
lane: functions-date
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Typed `10:00 A` is a number (0.41666...) with format `h:mm AM/PM`, and typed `1/2/2020 10:00 A.` (trailing dot) is a number (43832.41666...) with `m/d/yyyy h:mm`, as Excel does.

## Context
Found by T-1994: both seen in the ENTERED log of the excel-graph fill of probes/date-text-single-letter-ap (cells N9, N12 were dropped from the case). Add them as case cells (ISNUMBER, entered) and refill; `typed_clock_shape` in formatter/parse_formatted.rs is the code path. Record in docs/excel-semantics/dates.md (T-1994 section).

## Acceptance criteria
- [ ] The two shapes above are numbers with those formats, with probe cells in the case and no new F lines in eval/baseline/probes.txt.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Cells N9 (`10:00 A`) and N12 (`1/2/2020 10:00 A.`) added to probes/date-text-single-letter-ap and refilled (excel-graph); both are numbers. `parse_typed_number` drops the dot after a trailing single-letter A/P and gives a bare suffixed clock `h:mm AM/PM`. Eval: probes case 10 -> 12 cells, no new F lines.
