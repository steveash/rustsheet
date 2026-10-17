---
id: T-1355
title: "DATEDIF returns an error unit argument before rejecting a negative start date"
milestone: M6
lane: functions-date
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
DATEDIF returns an error value in its unit argument before it rejects an invalid (negative) start date, as Excel does.

## Context
T-1275's gen-m7 suite (excel-graph sidecars, 2026-10-11): gen-m7/f68fafa0ca2eba26 Calc!A1 `=(Data!D10^DATEDIF((Data!E$5+(-1)),38627,CHOOSE(3,Data!B12)))` with E5 blank:
Excel `#VALUE!` (the unit `CHOOSE(3,Data!B12)` is `#VALUE!`), engine `#NUM!` (start date -1 is rejected first;
`DATEDIF(1,38627,#VALUE!)` is already `#VALUE!`). docs/excel-semantics/dates.md "DATEDIF argument errors before date checks".
Code: `fn_datedif` in crates/rustsheet-engine/src/functions/date_and_time.rs: evaluate all three arguments, return the first
error, then validate. Unprobed (do not change unless evidenced): an invalid unit text with a negative start date.

## Acceptance criteria
- [ ] gen-m7/f68fafa0ca2eba26 A1 passes in `./target/release/rustsheet-eval show gen-m7/<case>.case.json`, with no new failing gen-m7 cell and no new known failures in eval/baseline/.
- [ ] A unit test with an error unit and a negative start date.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
