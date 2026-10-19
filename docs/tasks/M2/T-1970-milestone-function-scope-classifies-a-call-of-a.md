---
id: T-1970
title: "Milestone function scope classifies a call of a workbook defined name by the name's formula (POWER.xlsx inside_a_lambda C3)"
milestone: M2
lane: harness
status: in-progress
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
The report's "Milestone function scope (ironcalc-calc)" section (the M2 exit criterion "ironcalc-calc
cells using only M1/M2 functions") classifies a formula that calls a workbook defined name by the
functions in that name's formula. So `gelu(A1:C1)`, where `gelu` is a LAMBDA name over TANH, SQRT,
PI and POWER, is not counted as an M0-M2 cell.

## Context
At main 30bee011 the section lists `ironcalc-calc/MATH_AND_TRIGONOMETRY/POWER.xlsx!inside_a_lambda!C3`
as failing: C3 is 1 ulp off, 0.34571400982514394 against Excel's 0.3457140098251439. C3 is a
member of the array anchored at A3, `=gelu(A1:C1)`. The workbook defines
`gelu = _xlfn.LAMBDA(_xlpm.x, 0.5*_xlpm.x*(1 + TANH(SQRT(2/PI()) * (_xlpm.x + 0.044715 * POWER(_xlpm.x,3)))))`.
`function_scope::summarize` (crates/rustsheet-eval/src/function_scope.rs) takes `functions_in`
of the anchor formula. That yields only `GELU`, which the catalog does not list, so the cell
counts as in scope. Its real calls are LAMBDA (M7), TANH and POWER (M5). The twin row 4
(`gelu(OFFSET(...))`) is already out because of OFFSET. Decision 0051 item 2 already treats
defined-name bodies as part of a formula's subject (M8 as a feature). This is a
misclassification, not an M2 engine gap. The numeric gap is owned by T-0959 (M9, POWER/ulp
follow-ups; its AC covers the POWER.xlsx NumericUlp cells).

`CaseResult` has no defined names today. The xlsx runner path has to pass them on (for example,
a map from upper-case name to formula text, filled only for workbook-scope names of xlsx
cases). Keep the unknown-function rule (a name neither in the catalog nor defined still counts
as in scope), so that no cell drops out silently.

## Acceptance criteria
- [ ] A formula that calls a defined name is classified by the union of its own catalog functions
  and those of the name's formula. A name that calls another name is followed too, with a depth
  bound and a cycle guard (a self-referencing name test terminates). `LAMBDA`/`LET` parameter
  names (`_xlpm.`) and the `_xlfn.` prefix are read like elsewhere in `functions_in`.
- [ ] Unit tests in function_scope.rs: a cell calling a LAMBDA name over an M5 function is out of
  scope, a name over M2 functions only is in scope, and an undefined unknown name stays in scope.
- [ ] `make eval` report: the section no longer lists `POWER.xlsx!inside_a_lambda!C3`. The PR
  states the old and new "scored cells" counts and every cell that left the scope.
  eval/baseline is unchanged.
- [ ] The module doc comment and the section's lead text say that defined names are followed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing gelu's last digit (T-0959).
- Following cell precedents (still not followed).
