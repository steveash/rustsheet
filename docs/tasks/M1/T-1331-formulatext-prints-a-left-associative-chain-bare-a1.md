---
id: T-1331
title: "FORMULATEXT prints a left-associative ^ chain bare: =@A1^2^3 and =A1^2^3 (engine wraps =(@A1^2)^3; T-1320)"
milestone: M1
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-1244]
---

## Goal
FORMULATEXT of a left-associative `^` chain prints without added parentheses, as Excel does.

## Context
Excel prints `=@A1^2^3` (value 729, left-associative) as written; the engine prints `=(@A1^2)^3` because the power arm of `operator_text` (parser/stringify.rs) wraps a power-chain left operand. Failing cell probes/at-power-text.case.json Sheet1!I26 (T-1320). Check plain `=A1^2^3` with a probe first.

## Acceptance criteria
- [ ] `probes/at-power-text.case.json` I26 passes and leaves the baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Handoff
Probe probes/power-chain-text (excel-graph, 8 formulas, baseline recorded: 4 new failures I11, I26, I41, I46) shows Excel prints a `^` chain as written, so `=A1^2^3` and `=(A1^2)^3` print differently while the engine parses both to the same `Node`. Printing a left power chain bare fixes I11/I26/I46 but regresses `=(A1^2)^3`, `=(2^3)^2` and `(A1^2)^3^2`; no printer-only fix is right. This task needs the written-parentheses `Group` of T-1244 (spec first), so it now depends on it. Rule recorded in docs/excel-semantics/references.md. Next: after T-1244 lands, make the power arm print the written groups and take the four cells out of the baseline.
