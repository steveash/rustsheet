---
id: T-0206
title: "Oracle-confirm criteria grammar (D-function prefix/trim, error/boolean operands)"
milestone: M2
lane: functions
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-0205]
---

## Goal
Fill `eval/suites/probes/cases/criteria-grammar.case.json` from the Excel oracle, add D-function
(prefix equality, `<>text`, numeric/boolean/error criteria) and `<`/`>` with
error/boolean operand cases, then fix `functions/criteria.rs` where Excel differs and flip the
`docs/excel-semantics/criteria.md` entries from UNVERIFIED.

## Context
D-function criteria text is NOT trimmed (settled for the both-sides-padded case by the T-0004 smoke fill, smoke/d-function-criteria J2; leading-/trailing-only padding is T-0011); the engine no longer trims.

T-0205 unified the D-function criteria matcher with COUNTIF's grammar without oracle access; the
smoke case `d-function-criteria` is hand-expected. Four `TODO(T-0206)` markers in criteria.rs track
the unimplemented `<`/`>` ERROR/BOOLEAN operands.

## Acceptance criteria
- [ ] Probe expected values come from the oracle; discrepancies fixed or baselined.
- [ ] Semantics entries updated with evidence.
