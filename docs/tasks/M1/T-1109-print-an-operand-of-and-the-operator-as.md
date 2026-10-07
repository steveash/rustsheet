---
id: T-1109
title: "Print an @ operand of :, @ and the # operator as Excel does; support ANCHORARRAY"
milestone: M1
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
FORMULATEXT prints `@` operands of `:` and `@` with the parentheses Excel keeps, and `ANCHORARRAY(x)` evaluates and prints as `x#`.

## Context
Oracle evidence (T-1104, probes/at-operand-formulatext): Excel keeps the user's parentheses, so FORMULATEXT of `=(@(A1 B1)):C3` is `=(@(A1 B1)):C3` and of `=@(@(A1 B1))` is `=@(@(A1 B1))`; the engine prints `=(@A1 B1):C3` and `=@(@A1 B1)` (`tight_operand`, `at_operand` in crates/rustsheet-engine/src/expressions/parser/reference_operators.rs). `=ANCHORARRAY(D1)` prints `=D1#` and `=ANCHORARRAY(INDEX(D1:D2,1))` prints `=INDEX(D1:D2,1)#`; the engine gives `#NAME?` for ANCHORARRAY and prints `=anchorarray(D1)`. `=(@A1:B2)#` is refused by Excel at load. Failing cells: H12, H13, H17, J1, J30, K1, K30 (H12 is `=(@(A1 B1)):C3` itself; its value should be `#NULL!`).

## Acceptance criteria
- [ ] The seven failing cells of probes/at-operand-formulatext pass; baseline updated.
- [ ] A literal `D1#` and `ANCHORARRAY(D1)` print the same text; the automatic `@` inside `#` on import is dropped from the text.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
