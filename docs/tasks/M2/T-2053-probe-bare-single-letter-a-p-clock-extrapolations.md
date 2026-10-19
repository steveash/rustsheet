---
id: T-2053
title: "Probe bare single-letter A/P clock extrapolations (10 A., 0:00 A, 13:00 A, 10:00:30.5 A, 10 AM)"
milestone: M2
lane: functions-date
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Probe the bare single-letter A/P clock shapes T-2011 extrapolates and make the engine match Excel.

## Context
T-2011 review: `parse_bare_letter_clock` (formatter/parse_formatted.rs) covers `10 A.`, `0:00 A`, `13:00 A`, `10:00:30.5 A`, `10:00.5 A`, `10 AM` and `10:00:30 AM` by extrapolation. Add cells to probes/date-text-single-letter-ap, refill with excel-graph, record formats in docs/excel-semantics/dates.md (T-2011 section).

## Acceptance criteria
- [x] Each shape above has a probe cell and the engine matches the oracle value and format; `make check` green.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Cells N21-N29 added to probes/date-text-single-letter-ap and refilled (excel-graph). All extrapolations held except fractional seconds before the letter, which take `mm:ss.0`; `parse_bare_letter_clock` now does that and also accepts a bare hour with AM/PM. Eval: probes case 30 -> 57 cells, no new F lines.
