---
id: T-1141
title: "Probe IMAGE, STOCKHISTORY and FIELDVALUE with range, array, boolean and multiple-error arguments"
milestone: M1
lane: research
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence settles how IMAGE, STOCKHISTORY and FIELDVALUE treat the argument kinds T-1013 did not probe. docs/excel-semantics/information.md ("IMAGE and STOCKHISTORY never fetch") then records each finding as VERIFIED, or a fix task is filed.

## Context
T-1013 implemented the three functions from probes/rich-errors-* (functions/information/offline_errors.rs; signatures in parser/static_analysis.rs say scalar arguments and a scalar result). The engine's choices for unprobed inputs are marked UNVERIFIED in information.md. Probe, with the excel-probe skill, cells such as `IMAGE({"a","b"})`, `IMAGE(A1:A2)` (text in A1), `IMAGE("")`, `IMAGE(TRUE)`, `IMAGE(1/0,NA())`, `STOCKHISTORY(5,DATE(2020,1,2))`, `STOCKHISTORY(TRUE,DATE(2020,1,2))`, `STOCKHISTORY({"A","B"},DATE(2020,1,2))`, `STOCKHISTORY(A1:A2,DATE(2020,1,2))` `FIELDVALUE(5,{"a","b"})`, `FIELDVALUE(A1:B2,"Price")` (in ROWS/COLUMNS), `FIELDVALUE(A1:A1,"Price")`, `FIELDVALUE({1},"Price")`, `FIELDVALUE("abc","Price")`, `FIELDVALUE(TRUE,"Price")` and `FIELDVALUE(<empty cell>,"Price")`. Wrap each in ERROR.TYPE/ROWS/COLUMNS as rich-errors-args does, so lifting or spilling shows.

## Acceptance criteria
- [ ] A probe case with oracle values covers the cells above. information.md records each result, and any engine difference has a fix task filed (including the static_analysis signatures if Excel lifts or spills).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (file them as tasks).
