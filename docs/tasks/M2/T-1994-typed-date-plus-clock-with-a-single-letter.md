---
id: T-1994
title: "Typed date plus clock with a single-letter A or P suffix is a number (1/2/2020 10:00 A)"
milestone: M2
lane: functions-date
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Typed `1/2/2020 10:00 A` is a number (43832.41666...) with format `m/d/yyyy h:mm`, as Excel does.

## Context
Found by T-1986: probes/date-text-rejected-clock-formats N7 (excel-graph 2026-10-19) is a number with `m/d/yyyy h:mm`; ours stays text (baseline F line S!O7). Probe the neighbours first (`10:00 P`, `10:00 a`, glued `10:00A`, `10 A`, `10:00 Am`) with the excel-probe skill; `typed_clock_shape` in formatter/parse_formatted.rs and the date-clock parser in functions/date_and_time/time_text.rs are the code paths. Record the rule in docs/excel-semantics/dates.md.

## Acceptance criteria
- [ ] N7 of probes/date-text-rejected-clock-formats leaves eval/baseline/probes.txt, with no new F lines.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The cell format of rejected clock text (T-1986).
