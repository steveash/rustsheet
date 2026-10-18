---
id: T-1809
title: "Probe legacy ISEVEN/ISODD formulas: bare range, range+0 and {5} (values and FORMULATEXT)"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's behavior for legacy (non-dynamic-array) `ISEVEN`/`ISODD` formulas is evidenced: values and FORMULATEXT for `ISEVEN(C1:C5)`, `ISEVEN(C1:C5+0)` and `ISEVEN({5})`, and the same for ISODD.

## Context
T-1774 moved ISEVEN/ISODD onto the SPEC-0011 `LiftArrays` entry, so static analysis now follows the Analysis ToolPak legacy rule (no `@` on a bare range, `@` on a value operation, none on `{5}`). That is derived from EDATE/YEARFRAC probes only and is UNVERIFIED for ISEVEN/ISODD (docs/excel-semantics/information.md "ISEVEN/ISODD over a range"). Use the `excel-probe` skill.

## Acceptance criteria
- [ ] A probe case under eval/suites/probes covers the three forms as legacy formulas, with oracle values.
- [ ] information.md drops the UNVERIFIED mark or states the real rule; a follow-up fix task is filed if they differ.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
