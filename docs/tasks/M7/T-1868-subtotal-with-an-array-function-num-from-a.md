---
id: T-1868
title: "SUBTOTAL with an array function_num (from a dynamic-array function) lifts over its elements"
milestone: M7
lane: functions-math
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
SUBTOTAL whose function_num is an array (a dynamic-array function result such as TEXTSPLIT) gives what Excel gives instead of `#N/IMPL!`.

## Context
Found by T-1361's fidelity review. gen-m9/d5bd4272655c5211 Calc!A14
`=SUBTOTAL(TEXTSPLIT((Data!D6&Data!C8),Data!C11:E11,"-"),Data!C1:E1)`: Excel `#VALUE!`, ours `#N/IMPL!` since T-1361. Before
T-1361 the TEXTSPLIT was `#VALUE!` (the boolean in its delimiter range C11:E11 = 3.14159, TRUE, 1E-15 was rejected), so the cell
passed by accident; TEXTSPLIT now returns an array and SUBTOTAL has no lift over an array function_num. probes/subtotal-array-function-num (excel-graph, 2026-10-18, T-1361) settles
the rule: an array or range function_num lifts and spills (references.md "SUBTOTAL lifts over an array function_num").
Semantics: docs/excel-semantics/references.md ("SUBTOTAL lifts over an array function_num", "SUBTOTAL and AREAS take references only"). Code:
crates/rustsheet-engine/src/functions/subtotal.rs.

## Acceptance criteria
- [ ] gen-m9/d5bd4272655c5211 A14 passes in `./target/release/rustsheet-eval show gen-m9/d5bd4272655c5211.case.json`, no new failing gen-m9 cell.
- [ ] probes/subtotal-array-function-num passes (its failures are known in eval/baseline); references.md "Ours" updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
