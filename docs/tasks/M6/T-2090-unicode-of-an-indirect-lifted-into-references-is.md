---
id: T-2090
title: "UNICODE of an INDIRECT lifted into references is a SPILL"
milestone: M6
lane: functions-text
status: done
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
- [x] gen-m9/675763979c1defb6 Calc!A25 passes; baseline updated with no new known failures.
- [x] `make check` green; reviewers approve.

## Out of scope
- The other text functions unless the same branch serves them.

## Outcome
UNICODE reads its argument with `text_arg` and broadcasts like CODE and UNICHAR (`unicode_element`, which keeps the
lone-surrogate rule), and its static shape is `scalar_arguments`. A lifted INDIRECT, an array or a range now spills,
so gen-m9/675763979c1defb6 Calc!A25 is `#SPILL!` (the case passes 40/40). gen-m9 is not gated, so no baseline
changed; every gated suite: no regressions (probes 1680 -> 1680). text.md "UNICODE lifts over arrays and ranges".
