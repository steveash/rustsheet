---
id: T-2400
title: "Probe F.TEST with boolean, error and error-holding-range arguments"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Establish with the excel-probe skill what Excel returns for F.TEST/FTEST with a boolean or error scalar and with a scalar next to an error-holding range.

## Context
Follow-up of T-2093: unprobed cases F.TEST(TRUE,B1:B5), F.TEST(NA(),B1:B5), F.TEST(3,{1,#N/A}), F.TEST("a",B1:B5) with an error cell in B1:B5. fisher.rs fn_f_test currently gives #VALUE! for boolean/error scalars.

## Acceptance criteria
- [ ] Probe cases with oracle values added; findings recorded in docs/excel-semantics/statistics.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
