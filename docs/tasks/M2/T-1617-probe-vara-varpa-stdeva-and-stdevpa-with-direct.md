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
`variance_of_values_a` (functions/statistical/variance.rs, T-1567) still ignores a direct boolean, direct text and the text and booleans of an array ("A semantics to be added"), and reads a cell picked by IF/INDEX/XLOOKUP and a linked-workbook reference through the walker (`DirectNumbers::NumbersOnly`, `ErrorRule::First`). The direct forms have evidence already: Excel coerces a direct `"2"` and TRUE (probes/stat-argument-modes Sheet1!H69, H72, H75, H78 are known failures in eval/baseline/probes.txt). The single-cell reference and picked-cell forms changed in T-1567 (text 0, TRUE 1, as `A1:A1`) and are unprobed. Probe the rest (excel-probe skill), record the findings in docs/excel-semantics/coercion.md, and change the policy constants in `VARA_ARGS`/`VARA_NUMBERS` (the AVERAGEA rules `CastA`/`TextZero` are the likely answer for direct and array values).

## Acceptance criteria
- [ ] The four stat-argument-modes baseline lines (H69, H72, H75, H78) are cleared with `make eval-baseline`.
- [ ] Probe cases cover the forms above and pass; the code follows the oracle values.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
