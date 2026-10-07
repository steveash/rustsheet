---
id: T-1111
title: "Probe : right-operand errors, ANCHORARRAY of a non-reference and @-operand parentheses"
milestone: M1
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Oracle values for the forms T-1109 left UNVERIFIED, so the engine can be made to match.

## Context
T-1109 made `:` return its left operand's error and kept `#VALUE!` for a right-operand error without evidence. Probe: `=A1:(1/0)`, `=A1:INDEX(A1:A3,5)`, `=(1/0):(A1 B1)`, `=_xlfn.ANCHORARRAY((@A1:B2))`, `=ANCHORARRAY(1)` (value and FORMULATEXT), and FORMULATEXT of `=@((A1 B1))`, `=@(A1:B1)`, `=@(A1)`. Also `=A1:(1/0)` prints `=A1:1/0` in the engine, which does not re-parse to the same formula.

## Acceptance criteria
- [x] A probe case in eval/suites/probes with oracle values for each form above; findings in docs/excel-semantics/references.md (replace the UNVERIFIED notes); implementation tasks filed for any mismatch.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed on excel-graph (probes/range-right-operand-errors, at-paren-operands, anchorarray-at-operand):
a right-operand error of `:` is the result (`#REF!`, `#NULL!`); `ANCHORARRAY((@A1:B2))` prints
`=(@A1:B2)#` and is `#VALUE!`; FORMULATEXT keeps the parentheses of every `@` operand. Excel for the
web refuses workbooks holding `=A1:(1/0)`, `=(1/0):(A1 B1)`, `=(1/0):(A1 A2)` and `=ANCHORARRAY(1)`,
so those forms are not cases. Recorded in references.md. Eval delta: probes +3 cases, 6 new-coverage
known failures, 0 regressions. Follow-ups: T-1115, T-1116 (sonnet), T-1117 (opus).
