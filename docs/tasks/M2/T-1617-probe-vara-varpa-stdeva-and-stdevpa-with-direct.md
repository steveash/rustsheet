---
id: T-1617
title: "Probe VARA, VARPA, STDEVA and STDEVPA with direct booleans, direct text, array text and picked cells"
milestone: M2
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
VARA, VARPA, STDEVA and STDEVPA match Excel for single-cell reference and picked-cell arguments, with oracle probes behind the rule.

## Context
`variance_of_values_a` (functions/statistical/variance.rs, T-1567) still ignores a direct boolean, direct text and the text and booleans of an array ("A semantics to be added"), and reads a cell picked by IF/INDEX/XLOOKUP and a linked-workbook reference through the walker (`ErrorRule::First`). Update: T-1291 switched the direct forms to `DirectNumbers::CastA`. The direct forms were owned by T-1291 (it also cites gen-m9/288b99797f58f149 Calc!A7: a typed non-numeric text gives `#VALUE!`); this task covers only the single-reference and picked-cell forms. Evidence for the direct forms: Excel coerces a direct `"2"` and TRUE (probes/stat-argument-modes Sheet1!H69, H72, H75, H78 are known failures in eval/baseline/probes.txt). The single-cell reference and picked-cell forms changed in T-1567 (text 0, TRUE 1, as `A1:A1`) and are unprobed. Probe the rest (excel-probe skill), record the findings in docs/excel-semantics/coercion.md, and change the policy constants in `VARA_ARGS`/`VARA_NUMBERS` (the AVERAGEA rules `CastA`/`TextZero` are the likely answer for direct and array values).

## Acceptance criteria
- [ ] Probe cases cover the forms above and pass; the code follows the oracle values.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed (probes/vara-single-ref-picked, 41 cells, excel-graph): a single-cell reference and a cell picked by IF/INDEX/XLOOKUP coerce (text 0, boolean 1/0) exactly as the engine's `CellNumbers::CoercedA` already does, and array literals count only numbers (`ArrayNumbers::Numbers`), so no code change was needed. The three direct-form cells (`"x"`, `"2"`, `TRUE`) fail and are baselined; T-1291 owns them. Recorded in docs/excel-semantics/statistics.md. Eval: 38 new passing cells, 3 new known failures (direct forms, T-1291).
