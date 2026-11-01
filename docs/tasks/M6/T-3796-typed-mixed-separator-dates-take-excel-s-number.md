---
id: T-3796
title: "Typed mixed-separator dates take Excel's number format; typed clock-first date-time is a number (T-3751 probe misses)"
milestone: M6
lane: functions-date
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Typed cell entry in the engine (`parse_formatted_number`, `crates/rustsheet-engine/src/formatter/parse_formatted.rs`) gives the
number format Excel gives for the mixed-separator dates, and a typed clock-first date-time (`10:00 7/1/2025`) is a number.

## Context
- Found by T-3751: probes/date-text-mixed-separator-typed (excel-graph, 2026-11-01) and the ENTERED log in
  docs/research/typed-mixed-separator-dates-2026-11-01.md; rule in docs/excel-semantics/dates.md ("Typed mixed-separator dates").
- Excel's typed format against the engine's (`7/1-2025` and `7-1/2025` already agree on `m/d/yyyy`):
  - `2025/07-01`, `2025-07/01`: Excel `m/d/yyyy`, engine `yyyy/mm/dd`.
  - `1-Jul/2025`, `1/Jul-2025`: Excel `d-mmm-yy`, engine `d/mmm/yyyy`.
  - `7/1-25`: Excel `m/d/yyyy`, engine `m/d/yy`.
  - `10:00 7/1/2025` typed: Excel 45839.4166666667 with `m/d/yyyy h:mm`; the engine keeps text
    (probes/date-text-mixed-separator-typed B10, C10 fail in `make eval-show`).
  - A leading space before a clock-first date (` 10:00 7/1/2025`, ` 10:00 AM 7/1/2025`) stays text in Excel and in the engine.

## Acceptance criteria
- [ ] The typed formats above match Excel (unit tests in test_parse_formatted_number.rs citing the probe).
- [ ] probes/date-text-mixed-separator-typed B10 and C10 pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The coerced and function readers (DATEVALUE, VALUE, `0+text`), which already match.
