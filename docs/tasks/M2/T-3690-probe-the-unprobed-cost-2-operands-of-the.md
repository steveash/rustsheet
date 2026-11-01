---
id: T-3690
title: "Probe the unprobed cost-2 operands of the sign cap: LAMBDA definition, called name, called LAMBDA, +( and --( to 256, signs inside call arguments"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for the sign-cap cost (docs/excel-semantics/syntax.md "Formula nesting depth cap", T-3266) of the
operands T-3338 left unprobed, and the engine's `is_call_operand` (expressions/parser/depth.rs) matched or corrected.

## Context
- T-3266 evidenced: signs plus the operand's cost at most 1,024; a number literal right after the signs costs 0, a
  reference, `TRUE`, a string or a spaced number 1, a parenthesis or `ABS(1)` 2.
- T-3338 also charges 2 for a LAMBDA definition (`LAMBDA(x,x)`), a called LAMBDA (`LAMBDA(x,x)(1)`), a called name
  or LET variable, and a called group; unprobed. Also unprobed: `+(` and `--(` nested to 256 levels (only `-(` was),
  and sign runs inside call arguments (`ABS(---...1)`), which T-3338 counts together with the signs outside the call.
- Probe at the boundary (1,022 / 1,023 and 1,023 / 1,024 signs), stored and typed; typed refusals now score as blank
  cells (T-3338), so they can be kept.

## Acceptance criteria
- [ ] One excel-graph probe set covers each operand above at both sides of its boundary.
- [ ] `is_call_operand` and its TODO(T-3690) updated to the evidence; syntax.md records the rule.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The 65-call cap itself (T-3097).
