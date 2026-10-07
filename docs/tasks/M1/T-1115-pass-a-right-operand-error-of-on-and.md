---
id: T-1115
title: "Pass a right-operand error of : on, and give #VALUE! for ANCHORARRAY of an @ error"
milestone: M1
lane: engine
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
An error in the right operand of `:` is the result, as for the left operand, and `ANCHORARRAY((@A1:B2))` (an `@` that finds no cell) is `#VALUE!`, as Excel gives.

## Context
Probed in T-1111 (docs/excel-semantics/references.md, "FORMULATEXT keeps the parentheses of an `@` operand"):
`=A1:INDEX(A1:A3,5)` is `#REF!` and `=A1:(A1 B2)` is `#NULL!` (probes/range-right-operand-errors D11, D14),
where the engine gives `#VALUE!` (`Model::get_range`, crates/rustsheet-engine/src/reference_operators.rs, the
`_ => #VALUE!` arm; T-1109 already passes a left-operand error on). `=ANCHORARRAY((@A1:B2))` in D20 is
`#VALUE!` (probes/anchorarray-at-operand D20); the engine gives `#ERROR!` (`evaluate_spill_anchor`).

## Acceptance criteria
- [ ] probes/range-right-operand-errors D11, D14 and probes/anchorarray-at-operand D20 pass; `make eval-baseline` records them.
- [ ] Unit tests for both rules; references.md drops "the engine gives" notes for them.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
