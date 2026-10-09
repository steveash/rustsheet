---
id: T-1117
title: "Refuse non-reference operands of : and ANCHORARRAY as Excel's loader does"
milestone: M1
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The engine treats `A1:(1/0)`, `(1/0):(A1 B1)` and `ANCHORARRAY(1)` the way Excel treats them, i.e. not as valid formulas, consistently with the union rule (T-0193).

## Context
Probed in T-1111: Excel for the web refuses a workbook holding `=A1:(1/0)`, `=(1/0):(A1 B1)`,
`=(1/0):(A1 A2)` or `=ANCHORARRAY(1)` (createSession 501 FileCorruptTryRepair, one scratch workbook per
formula; docs/excel-semantics/references.md). The same holds for non-reference union operands, which the
engine reads as syntax errors since T-0193 (references.md "Union: parsing and evaluation"). The engine today
evaluates these and prints `=A1:(1/0)` as `=A1:1/0`, which does not re-parse to the same formula. Decide which
operands `:` takes (references, `@`/`#`/INDEX-like reference-returning calls, intersections, which all load:
probes/at-operand-formulatext, probes/range-right-operand-errors) and refuse the rest at parse time. Typed-entry
behavior is a desktop question (M14).

## Acceptance criteria
- [x] Parser unit tests: the four refused forms are syntax errors; every `:` form in eval/suites/probes still parses and evaluates as before (`make eval-check` no regressions).
- [x] references.md updated with the engine rule.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
The parser reads a non-reference operand of `:` (either side), of a typed `#` and of ANCHORARRAY as a
syntax error, using the intersection's `is_reference_operand` rule (`range_operand_error` in
parser/reference_operators.rs). The four probed refusals are now parse errors; every `:` form in the
probes suite still parses (eval-check: 0 regressions, 0 fixed; eval delta none). references.md records the
engine rule and marks the unprobed value operands as inferred (UNVERIFIED).
