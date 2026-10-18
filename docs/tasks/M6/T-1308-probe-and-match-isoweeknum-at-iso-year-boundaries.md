---
id: T-1308
title: "Probe and match ISOWEEKNUM and WEEKNUM at modern year boundaries and week-53 years"
milestone: M6
lane: functions-date
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
ISOWEEKNUM and WEEKNUM match Excel at the year boundaries where week numbering is hardest:
ISO week 53 years, early-January days in the previous ISO year, and late-December days in week 1
of the next. ISOWEEKNUM gets at least 25 Excel cells (decision 0031).

## Context
Evidence gap (eval/runs/latest/report.md, 2026-10-10): ISOWEEKNUM has 18 Excel cells, and every
WEEKNUM/ISOWEEKNUM Excel cell is a 1900-era serial or an argument-mode cell (probes/weeknum-serial-0,
weeknum-1900-later, dates-early-1900-edges, weeknum-invalid-type, date-argument-modes F33/F34/F93-F96;
docs/excel-semantics/dates.md "WEEKNUM"). Not one Excel cell checks a modern year boundary. The engine's
ISO week (`weeknum::iso_week`, used by `fn_isoweeknum` in crates/rustsheet-engine/src/functions/
date_and_time.rs and by WEEKNUM return type 21) and the system-1 week anchors for types 1, 2 and 11-17
are unverified there.

Suggested cells (write dates as `DATE(y,m,d)` so the case reads clearly):
- ISOWEEKNUM and WEEKNUM(…,21) of 2020-12-31 (53), 2021-01-01, 2021-01-03 (53 of 2020),
  2021-01-04 (1), 2024-12-29, 2024-12-30 (1 of 2025), 2026-12-31, 2027-01-01, 2027-01-03 (53),
  2015-12-31, 2016-01-03, 2032-12-31, 2033-01-02, and 9999-12-31 (serial 2958465).
- WEEKNUM types 1, 2, 11, 12, 13, 14, 15, 16 and 17 on 2023-01-01 (a Sunday), 2022-01-01
  (a Saturday), 2021-12-31 and 2024-12-31.
- A fractional serial (`ISOWEEKNUM(44197.99)`), a serial past the last date (`ISOWEEKNUM(2958466)`)
  and a date as text (`ISOWEEKNUM("2021-01-03")`).

## Acceptance criteria
- [ ] A probe case (excel-probe skill, excel-graph sidecar) with the cells above; report.md lists
      ISOWEEKNUM with at least 25 Excel cells.
- [ ] The engine matches every cell, or each mismatch has its own functions-date task naming its
      cells.
- [ ] docs/excel-semantics/dates.md records the modern-date ISO and WEEKNUM rules, citing the probe
      cells.
- [ ] `make check` green; no new F lines in other suites; reviewers approve.

## Out of scope
- Lifting over ranges and arrays (T-1235, T-1236).
- Stale WEEKNUM return types (decision 0038, probes/weeknum-stale-return-type).
- The 1904 date system (SPEC-0005, T-1266 follow-ups).

## Outcome
Added probes/weeknum-modern-boundaries (67 cells: ISOWEEKNUM and WEEKNUM(,21) at 14 boundary dates, WEEKNUM types 1, 2, 11-17 on four dates, a fractional serial, a serial past the last date and date text), filled by excel-graph. The engine matched all 67 cells, so no mismatch tasks. dates.md records the rule. Eval: 67 new passing cells, 0 regressions.
