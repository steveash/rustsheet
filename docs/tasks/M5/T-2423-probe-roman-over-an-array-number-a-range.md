---
id: T-2423
title: "Probe ROMAN over an array number, a range and boolean form elements (T-2410 unprobed)"
milestone: M5
lane: functions-math
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's behavior is on record for ROMAN over an array number, a multi-cell range and an array of booleans as the form.

## Context
T-2410 lifted ROMAN (`LIFT_1_OF_2`); only the array form is oracle-evidenced (gen-m7/f7e94a817e5385ac A1563). Probe with the `excel-probe` skill: `ROMAN({1;4},0)`, `ROMAN(4,A1:A2)`, `ROMAN(4,{TRUE,FALSE})`, and which error wins for `ROMAN(-5,NA())` (numbers.md "ROMAN truncates before its sign check").

## Acceptance criteria
- [ ] A probe case under eval/suites/probes with oracle values for the formulas above; numbers.md records the results; any mismatch is filed as a task.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `probes/roman-lift-forms` filled by excel-graph; results in numbers.md (ROMAN section). Array/range/boolean forms lift element-wise as implemented. Whether ours matches on `ROMAN(-5,NA())` (`#N/A`) is checked by `make check`'s eval run; the one mismatch is filed as T-2523.
