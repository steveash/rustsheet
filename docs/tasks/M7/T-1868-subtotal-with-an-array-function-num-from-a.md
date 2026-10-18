---
id: T-1868
title: "SUBTOTAL with an array function_num (from a dynamic-array function) is not #N/IMPL!"
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
passed by accident; TEXTSPLIT now returns an array and SUBTOTAL has no lift over an array function_num. Probe first with the
excel-probe skill: `SUBTOTAL({9,1},A1:A3)`, `SUBTOTAL({9},A1:A3)`, `SUBTOTAL(TEXTSPLIT("9,1",","),A1:A3)`, `ROWS(SUBTOTAL({9;1},A1:A3))`,
`SUBTOTAL({"x"},A1:A3)`. Semantics: docs/excel-semantics/references.md (SUBTOTAL reference operands). Code:
crates/rustsheet-engine/src/functions/subtotal.rs.

## Acceptance criteria
- [ ] gen-m9/d5bd4272655c5211 A14 passes in `./target/release/rustsheet-eval show gen-m9/d5bd4272655c5211.case.json`, no new failing gen-m9 cell.
- [ ] The probe cells pass; the rule is recorded in references.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
