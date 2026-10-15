---
id: T-1617
title: "Probe VARA, VARPA, STDEVA and STDEVPA with direct booleans, direct text, array text and picked cells"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
VARA, VARPA, STDEVA and STDEVPA match Excel for every argument form, with oracle probes behind each rule.

## Context
`variance_of_values_a` (functions/statistical/variance.rs, T-1567) still ignores a direct boolean, direct text and the text and booleans of an array ("A semantics to be added"), and reads a cell picked by IF/INDEX/XLOOKUP and a linked-workbook reference through the walker (`DirectNumbers::NumbersOnly`, `ErrorRule::First`). None of these is probed. Probe each form (excel-probe skill), record the findings in docs/excel-semantics/coercion.md, and change the policy constants in `VARA_ARGS`/`VARA_NUMBERS` (the AVERAGEA rules `CastA`/`TextZero` are the likely answer for direct and array values).

## Acceptance criteria
- [ ] Probe cases cover the forms above and pass; the code follows the oracle values.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
