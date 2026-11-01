---
id: T-3751
title: "Probe typed mixed-separator date input and a leading space before a leading clock"
milestone: M6
lane: functions-date
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-3646]
---

## Goal
Typed cell entry of a date that mixes `/` and `-` (`7/1-2025`), and a leading space before a clock that comes first
(` 10:00 7/1/2025` typed, `0+" 10:00 7/1/2025"` coerced), give Excel's value and number format, settled by an
oracle probe rather than inferred from the function readers.

## Context
- Follow-up of T-3646 (PR #2762). That task made the shared date-text reader accept `/` and `-` mixed in one date
  (`"7/1-2025"`, `"7-1/2025"`, `"2025/07-01"`, `"1-Jul/2025"`) and a clock before the date (`"10:00 7/1/2025"`,
  `"10 AM 7/1/2025"`). Its evidence is only through DATEVALUE, VALUE, YEAR and TIMEVALUE: probes/date-text-shape-survey
  and probes/date-text-shape-survey-2 (excel-graph, 2026-11-01; docs/excel-semantics/dates.md, "Shape survey").
- Unprobed, and inferred by the engine:
  - Typed entry of a mixed-separator date: its value and, above all, its cell number format. The engine picks the
    format from the first separator it finds (`7/1-2025` takes the slash date's `m/d/yyyy`, `7-1/2025` the dash
    one). Excel may choose by the other separator, by the year's position, or keep the text.
  - A leading space before a leading clock: `0+" 10:00 7/1/2025"` (coercion) and a typed ` 10:00 7/1/2025`
    (entered cell: value, type and format). With the date first, `0+text` rejects a leading space and VALUE trims
    it (dates.md, probes/date-clock-text-length-limit; `SUM(" 10:00")` is #VALUE!, probes/clock-text-oddities);
    with the clock first, neither coercion nor typed entry is probed.
- Code (after #2762 merges): crates/rustsheet-engine/src/formatter/date_text.rs (the shared reader) and
  crates/rustsheet-engine/src/formatter/parse_formatted.rs (typed entry, number format of a typed date and clock).
  Earlier typed-entry probes to copy the shape from: probes/date-text-typed-clock-shapes (T-1979, formats read
  from the ENTERED log, docs/research/typed-clock-shapes-2026-10-19.md).

## Acceptance criteria
- [ ] One excel-graph probe case (`excel-probe` skill) with typed entries `7/1-2025`, `7-1/2025`, `2025/07-01`,
      `2025-07/01`, `1-Jul/2025`, `1/Jul-2025`, `7/1-25`, ` 10:00 7/1/2025`, ` 10:00 AM 7/1/2025` and
      `10:00 7/1/2025`, each scored for value and type with its cell number format recorded, plus formula cells
      `=0+" 10:00 7/1/2025"`, `=DATEVALUE(" 10:00 7/1/2025")` and `=VALUE(" 10:00 7/1/2025")`.
- [ ] Every cell of the new case passes, or each failing one is listed in a filed follow-up task.
- [ ] docs/excel-semantics/dates.md records the typed format of a mixed-separator date and the leading-space rule,
      citing the probe's cells; the engine-inferred wording above is replaced by the evidence.
- [ ] `make check` green; eval known failures only shrink (no new F lines outside the new case); reviewers approve.

## Out of scope
- Other date-text shapes T-3646 already evidenced through the function readers.
- Glued month-name shapes (T-3299).
