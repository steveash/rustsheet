---
id: T-2090
title: "UNICODE of an INDIRECT lifted into references is a SPILL"
milestone: M6
lane: functions-text
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
UNICODE (and any text function reading a range) of a lifted INDIRECT gives Excel's `#SPILL!`, not `#N/IMPL!`.

## Context
Split from T-1957. gen-m9/675763979c1defb6 Calc!A25 `=UNICODE(INDIRECT("Data!A2:A4",VSTACK(Data!C6:E7)))` is `#SPILL!` in Excel and `#N/IMPL!` in the engine, as is `=UNICODE(INDIRECT({"A1";"A2"}))`: UNICODE's `CalcResult::Range` branch returns NIMPL and it has no array lift (functions/text/common.rs ~455). Lifted INDIRECT now hands it the value view of the grid.

## Acceptance criteria
- [ ] gen-m9/675763979c1defb6 Calc!A25 passes; baseline updated with no new known failures.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other text functions unless the same branch serves them.
