---
id: T-2277
title: "HYPERLINK: an error link_location is the result, whatever friendly_name is (no spill)"
milestone: M6
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
HYPERLINK returns an error link_location as its single result, whatever friendly_name is (number, array or range), with no spill, as Excel does.

## Context
Gap triage of gen-m6 (T-2180, excel-graph oracle, 2026-10-21). Rule and cells: docs/excel-semantics/lookup.md "HYPERLINK: an error link_location
is the result" (added by this triage). `fn_hyperlink` (crates/rustsheet-engine/src/functions/lookup_and_reference/mod.rs) returns
friendly_name when given, and lifts over an array or range friendly_name even when link_location is an error (so `#SPILL!`, or a
spilled `#N/A` into B214). Check the lift driver's argument kinds for HYPERLINK (lookup.md "probed scalar arguments lift") so that a
lifted *non-error* link_location still lifts.

Failing cells (`make eval-show CASE=gen-m6/<case>.case.json`), 4 in eval/baseline/gen-m6.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m6/0bcac2d96e96b3df Calc!A17 | `=HYPERLINK(TEXTJOIN(VLOOKUP(Data!$B5,Data!B2:C10,1,Data!E10),(1e-15=Data!D10),Data!C5:E10,Data!$C11),GCD(Da...` | "#VALUE!" | 1.0 |
| gen-m6/44194b2fefa30de2 Calc!A169 | `=HYPERLINK(XLOOKUP(AVERAGE(Data!E7:E7,100),Data!E7:E12,Data!D9:E12),MUNIT(2))` | "#VALUE!" | "#SPILL!" |
| gen-m6/c6621490a4d67f4e Calc!B214 | `(spill cell of A214)` | (empty) | "#N/A" |
| gen-m6/d0c30225243fde60 Calc!A377 | `=HYPERLINK((HLOOKUP(100,Data!A6:C11,7)&(Data!B$9&Data!E8)),OFFSET(Data!A6:C12,6,42))` | "#REF!" | "#SPILL!" |

## Acceptance criteria
- [ ] These 4 known-failure lines disappear from eval/baseline/gen-m6.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] Unit tests cover `HYPERLINK(NA(),5)`, `HYPERLINK(NA(),{1,2})` (one `#N/A`) and `HYPERLINK("#A1",{1,2})` (still spills).
- [ ] lookup.md's entry says Ours: implemented (T-2277).
- [ ] `make check` green; reviewers approve.

## Out of scope
- HYPERLINK's displayed text and link storage.

## Visible-parity stage
Decision 0083: every cell above is visibly off (a wrong error, a wrong type or a value off in a leading digit). The acceptance criteria above are the stage.
