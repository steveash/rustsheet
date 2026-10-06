---
id: T-1021
title: "Probe overflow of array-criteria SUMIF, VAR.S, STDEV.S, VAR.P, STDEV.P, VARA, STDEVA"
milestone: M1
lane: harness
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Probes record whether array-criteria SUMIF, VAR.S, STDEV.S, VAR.P, STDEV.P, VARA and STDEVA overflow to #NUM! in Excel.

## Context
Follow-up of T-0991, which made SUMIF (scalar and spilling), VAR and STDEV (also VAR.S, STDEV.S, same code) return #NUM! on overflow with probes only for the scalar legacy forms. Add cells to probes/aggregate-overflow and record findings in numbers.md (Aggregate overflow).

## Acceptance criteria
- [ ] The probe cells have oracle values and the engine matches them or the gaps are baselined.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
