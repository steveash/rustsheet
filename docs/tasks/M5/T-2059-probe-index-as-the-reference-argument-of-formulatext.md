---
id: T-2059
title: "Probe INDEX as the reference argument of FORMULATEXT, CELL and OFFSET"
milestone: M5
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for what Excel gives when a single-cell `INDEX(range,n)` is the reference argument of FORMULATEXT, CELL and OFFSET.

## Context
T-1453 made `evaluate_call_with_reference` (functions/logical/lambda.rs) evaluate INDEX in its reference form, so every `get_reference`/`evaluate_node_with_reference` reader now sees the picked cell as a reference. Only ISFORMULA is oracle-backed (probes/ifna-isformula-edges Calc!B12). Unprobed: `FORMULATEXT(INDEX(A1:A3,2))` (A2 a formula), `CELL("address",INDEX(A1:B3,2,2))`, `CELL("row",INDEX(..))`, `OFFSET(INDEX(A1:A3,2),1,0)`, and their error forms (`INDEX(A1:A3,9)`, INDEX over an array constant). See docs/excel-semantics/references.md "IF and INDEX return references to aggregates".

## Acceptance criteria
- [ ] A probe case under eval/suites/probes with excel-graph oracle values covers the forms above.
- [ ] Findings recorded in docs/excel-semantics/references.md; any engine mismatch filed as its own task.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
