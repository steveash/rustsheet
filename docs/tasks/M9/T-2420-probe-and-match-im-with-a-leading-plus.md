---
id: T-2420
title: "Probe and match IM* with a leading plus, OCT2DEC of 11 digits and OCT/HEX conversions of TRUE"
milestone: M9
lane: functions-engineering
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
IM* parsing, OCT2DEC and the OCT/HEX conversions give Excel's values on the three inputs where they differ.

## Context
Found by the T-2249 smoke check: a scratch suite of 790 cells, one formula per entry of the M9 engineering and database pools, filled through excel-graph on 2026-10-22 and deleted afterwards (docs/excel-semantics/engineering.md and criteria.md, "edge-pool smoke check (T-2249)"). Commit a probe case with these cells first (excel-probe skill).
- `IMABS("+i")` is 1 and `IMSUB("+i","1")` is "-1+i": a leading `+` before a bare `i` is read. Ours `#NUM!`. Probe `"+3+4i"`, `"+4i"`, `"+3"` and the other IM* functions too.
- `OCT2DEC(77777777777)` (11 octal digits, a number) is `#NUM!`. Ours 7516192767.
- `OCT2DEC(TRUE)`, `OCT2BIN(TRUE)`, `HEX2DEC(TRUE)`, `HEX2BIN(TRUE)` and `HEX2OCT(TRUE)` are `#VALUE!`, as `BIN2DEC(TRUE)` already is (engineering.md "Engineering function argument modes"). Ours `#NUM!`.
Code: functions/engineering (complex parsing, base conversions).

## Acceptance criteria
- [ ] A probe case holds the cells above, plus neighbours (a leading + on other IM* forms, OCT2HEX/OCT2BIN of 11 digits, a TRUE cell).
- [ ] Ours matches every probe cell; docs/excel-semantics/engineering.md is updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other base-conversion or IM* parsing changes.
