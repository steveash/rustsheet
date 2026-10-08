---
id: T-1163
title: "Probe SUBTOTAL(2/3,{1,2}) direct array and lambda arguments (follow-up of T-1154)"
milestone: M1
lane: functions-math
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
SUBTOTAL 2, 3, 102 and 103 with a direct array or lambda argument match Excel, with oracle evidence.

## Context
T-1154 moved COUNT/COUNTA onto `subtotal_walk_areas`; `subtotal_count_with` (functions/subtotal.rs) returns `#N/IMPL` for a direct array or lambda argument, where COUNT used to ignore it (0) and COUNTA already returned `#N/IMPL`. Neither is evidenced; Excel likely gives `#VALUE!` for a non-reference SUBTOTAL argument.

## Acceptance criteria
- [x] A probe case (excel-graph) covers `SUBTOTAL(2,{1,2})`, `SUBTOTAL(3,{1,2})`, 102/103 and a lambda argument; the finding is in docs/excel-semantics/references.md.
- [x] The engine matches the oracle; eval baseline shrinks accordingly.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Handoff (resumed)
Resumed by an opus run on 2026-10-08. The tier was raised from sonnet to opus by the handoff.

## Outcome
Excel for the web refuses a workbook holding a SUBTOTAL argument (after the function number) that is not a reference operand: an array, a computed array, a literal, a value function, an empty argument or a nested SUBTOTAL. It refuses `AREAS(1)` the same way. Each form was probed in its own scratch workbook. The parser now turns these into parse errors (`Parser::function_call`, decision 0045), which also settles the AREAS non-reference row that T-1063 left as an engine choice. Reference operands that evaluate to something else load in Excel. A value is `#VALUE!`, an error stays itself, and either beats an error cell inside a range. An array spills `#VALUE!` per element. A LAMBDA argument is `#VALUE!`. SUBTOTAL resolves every argument before it reads any cell (`subtotal_reference_args`), and the old `#N/IMPL` paths are gone. The new case probes/subtotal-reference-operands passes all 40 of its cells. Eval delta: +1 case, +40 passing cells, 0 regressions, known failures unchanged. Follow-ups: T-1174 (desktop Excel, blocked on the desktop oracle) and T-1175 (the UNVERIFIED rows).
