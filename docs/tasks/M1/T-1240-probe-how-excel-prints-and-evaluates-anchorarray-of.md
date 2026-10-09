---
id: T-1240
title: "Probe how Excel prints and evaluates ANCHORARRAY of an @ operand (follow-up of T-1226)"
milestone: M1
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's printed text and value for stored `ANCHORARRAY` of an `@` operand are probed, and the engine rule (now: only `@<number>` prints bare) is made to match.

## Context
T-1226 evidence has two points that confound operand kind with written parentheses: `ANCHORARRAY(@1)` prints `=@1#` and `ANCHORARRAY((@A1:B2))` prints `=(@A1:B2)#`. Probe stored `ANCHORARRAY(@A1:B2)`, `ANCHORARRAY((@1))`, `ANCHORARRAY(@"a")`, `ANCHORARRAY(@SUM(A1))`, `ANCHORARRAY(@INDEX(D1:D2,1))`, `ANCHORARRAY(@LAMBDA(x,x))`, `ANCHORARRAY(@-1)` and `ANCHORARRAY(@1/0)` (value and FORMULATEXT; refused or loaded). Code: `spill_operand` (parser/reference_operators.rs), `spill_anchor_error`; UNVERIFIED note in docs/excel-semantics/references.md (T-1226). Use the `excel-probe` skill.

## Acceptance criteria
- [ ] Probe case with oracle values for every form above, committed under eval/suites/probes; references.md says which rule (operand kind or written parentheses) holds; engine matches or follow-up tasks are filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
