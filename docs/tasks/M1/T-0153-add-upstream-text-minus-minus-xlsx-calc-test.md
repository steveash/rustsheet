---
id: T-0153
title: "Add upstream TEXT/MINUS_MINUS.xlsx calc_test and the 10:11.5:12 clock oddity"
milestone: M1
lane: engine
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The upstream IronCalc `TEXT/MINUS_MINUS.xlsx` workbook passes whole as a calc_test, and `"10:11.5:12"` (timevalue-component-limits B26/C26) matches Excel.

## Context
Left over from T-0146. The workbook is `git show 9b1ae3fd:xlsx/tests/calc_tests/TEXT/MINUS_MINUS.xlsx` in the IronCalc upstream clone (`scripts/upstream-log.sh`); it needs time-text coercion (done in T-0146). The oddity: `TIMEVALUE("10:11.5:12")` = 0.424369212962963 (36665.5 s), see docs/excel-semantics/dates.md "Time text".

## Acceptance criteria
- [ ] Probe and settle the unverified edges listed under dates.md "Time text" (hour-only AM/PM forms, VALUE of time text, direct arguments of AVERAGE/SUM, `+10:00` typed into a cell).
- [ ] MINUS_MINUS.xlsx added under crates/rustsheet-xlsx/tests/calc_tests/TEXT/ and passes; B26/C26 leave eval/baseline/probes.txt.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
