---
id: T-1879
title: "Probe glued month-name date text outside DATEVALUE: date+time text, criteria, typed-input formats and unprobed shapes"
milestone: M2
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for the glued month-name forms that T-1789 implements but probes/date-text-glued-month does not cover, so docs/excel-semantics/dates.md can drop its UNVERIFIED notes or record Excel's real behavior.

## Context
T-1789 splits a month name glued to digits (`"July1"`, `"1JUN2021"`) in `parse_date`
(crates/rustsheet-engine/src/formatter/date_text.rs). The probe covers DATEVALUE, VALUE, `0+`, MONTH
and DAY only, but `parse_date` is shared, so the rule also reaches (found by the T-1789 Excel-fidelity
review):
- date+time text through `time_text::parse_date_clock_parts`: `"July1 10:00"`, `"Jul1 10 AM"`,
  `"1JUN2021 6 PM"` (ours now give a date-time; before T-1789 #VALUE!), via VALUE and TIMEVALUE;
- criteria through `criteria.rs::parse_date_criterion`: COUNTIF/SUMIF with `"July1"`, `">1JUN2021"`;
- typed input through `parse_formatted.rs`: the value and inferred number format of typed `"July1"`,
  `"July01"` (ours "d-mmm"), `"Jul2026"`, `"Jun31"` (ours "mmm-yy") and `"1Jul 2026"` (ours "d-mmm-yy");
- shapes the rule decides without evidence: `"Jul1 , 2026"`, `"July1, 2026"`, `"1-Jul2026"`, `"1July1"`
  (ours #VALUE!), `"Jun31"` (ours 1931-06-01), `"Feb29"` (ours 2029-02-01 with a 2026 clock).
Typed-input formats may need the oracle's typed-entry path; check tools/ first. dates.md, glued-month
entry ("UNVERIFIED" paragraph).

## Acceptance criteria
- [ ] A probe case (excel-probe skill) covers every input above, oracle-filled.
- [ ] dates.md records the outcome for each; every mismatch with the engine is filed as a task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing the outcome.
