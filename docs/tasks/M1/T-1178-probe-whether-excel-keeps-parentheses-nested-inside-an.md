---
id: T-1178
title: "Probe whether Excel keeps parentheses nested inside an @ operand"
milestone: M1
lane: research
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence for whether Excel keeps parentheses nested inside an `@` operand, with the engine matching it.

## Context
T-1167 found the engine drops groups nested inside an `@` operand (`@(A1+(B1))` prints `=@(A1+B1)`, `@((A1) (B1))` prints `=@(A1 B1)`). No probe covers them; see the UNVERIFIED note in docs/excel-semantics/references.md and the test `at_operand_adds_no_level_for_a_nested_inner_group`.

## Acceptance criteria
- [x] Probe `=@(A1+(B1))`, `=@((A1) (B1))` and `=@(((A1)) B1)` with FORMULATEXT (excel-probe skill); the references.md entry is VERIFIED.
- [x] The engine and the T-1167 test match: superseded by T-1240 (see Outcome).
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
probes/at-operand-nested-groups (36 cells, excel-graph, 2026-10-09): Excel keeps every written parenthesis, inside `@`
operands and in plain formulas alike (`=@(A1+(B1))`, `=@((A1) (B1))`, `=@(((A1)) B1)`, `=A1+(B1)`, `=-(A1)`, `=((A1))`).
Recorded in syntax.md "FORMULATEXT keeps every written parenthesis"; references.md's UNVERIFIED note now points there.
The engine drops redundant groups everywhere, not only in `@` operands, so matching Excel means keeping groups in the
parse tree, an architectural change that needs a spec first (directive 4): filed as T-1240 (engine, opus), which also
subsumes T-1200. The T-1167 test still pins the engine's current output; its comment now cites the evidence and T-1240.
Eval delta: probes +1 case, 36 cells, 22 pass (all 18 values and 4 FORMULATEXT cells), 14 new known failures (T-1240).
Follow-ups: T-1240.
