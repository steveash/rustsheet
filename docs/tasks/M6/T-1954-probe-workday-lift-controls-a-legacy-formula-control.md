---
id: T-1954
title: "Probe WORKDAY lift controls: a legacy-formula control cell and holidays error vs all-weekend mask order"
milestone: M6
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
probes/workday-lift-errors proves that its legacy formulas stayed legacy, and the order of a holidays error against an all-weekend mask in WORKDAY.INTL is evidenced.

## Context
T-1236 review (Excel fidelity, minor). probes/workday-lift-errors P3:T70 show legacy WORKDAY/NETWORKDAYS(.INTL) as `#VALUE!` with no `@`. The case has no control cell, though, so it relies on the oracle keeping `legacy_formulas` legacy, as T-1053's `=D2:D5*1` control showed for another file (dates.md "WORKDAY holidays take the whole range"). Separately, `Model::workday_of` (functions/date_and_time/workday.rs) returns a holidays error before the all-weekend `#VALUE!`, as main's scalar code did. In `WORKDAY.INTL(45000,1,"1111111",NA())` that order is unprobed, and K12/K13 cannot tell the two apart because both are `#VALUE!`.

## Acceptance criteria
- [ ] probes/workday-lift-errors gains a legacy control `U3: =A1:A60*1` (expected 45002 if legacy), plus `WORKDAY.INTL(45000,1,"1111111",NA())` and `NETWORKDAYS.INTL(45000,46000,"1111111",NA())`, filled from excel-graph.
- [ ] The results are recorded in docs/excel-semantics/dates.md "Date function argument modes". If a cell fails, a follow-up task is filed or the fix is made.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
