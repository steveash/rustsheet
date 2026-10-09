---
id: T-0165
title: "XLOOKUP, TAKE, DROP and TRIMRANGE return references usable as intersection operands"
milestone: M1
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
XLOOKUP, TAKE, DROP and TRIMRANGE over a range return a reference, so `=A1:A3 XLOOKUP(2,A1:A3,A1:B3)`, `=A1:A3 TAKE(A2:B3,1)` `=A1:A3 DROP(A1:B2,1)` and `=A1:A3 TRIMRANGE(A2:B2)` give 2 as in Excel.

## Context
T-0161 found that Excel accepts these calls as intersection operands and evaluates them as references
(probes/intersection-function-operands H8, H11, H12, H14 (TRIMRANGE, T-0166), excel-graph 2026-10-06). The parser accepts them
(`is_reference_operand`), but the engine evaluates them to arrays, so the intersection gives `#VALUE!`
(4 probes known failures). INDEX, OFFSET, IF, INDIRECT, CHOOSE, SWITCH and IFS already pass. A reference
result should also make `ISREF(TAKE(A1:B3,1))` TRUE; probe that if no evidence exists.

## Acceptance criteria
- [x] probes/intersection-function-operands H8, H11, H12, H14 pass; baseline shrinks.
- [x] Other uses of these functions (spills, aggregates over them) unchanged in every suite.
- [x] `make check` green; reviewers approve.

## Out of scope
- CHOOSEROWS/CHOOSECOLS (Excel refuses them as operands).

## Outcome
XLOOKUP, TAKE, DROP and TRIMRANGE have a reference form (`Model::evaluate_reference_form`, the
`as_reference` pattern of INDEX) used by intersection operands and ISREF; everywhere else they keep
their value path. TAKE and DROP now share one span helper (also fixing a debug overflow on a count of
-1E300). New probes: isref-reference-lookups (ISREF of these calls) and
intersection-lookup-operand-errors (ends, errors, range if_not_found), both excel-graph. Eval: probes
fixed 4 (intersection-function-operands H8, H11, H12, H14), new coverage 21 cells with 1 known failure
(isref-reference-lookups H10, XLOOKUP over an array lookup vector: follow-up T-1126), 0 regressions.
