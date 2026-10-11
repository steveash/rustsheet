---
id: T-1337
title: "Probe ANCHORARRAY(@IF(TRUE,D1:D3)) and @IF picking a union as a spill anchor"
milestone: M1
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for what Excel gives when the `@` operand of a spill anchor is an IF that picks a multi-cell range or a union, recorded in docs/excel-semantics/references.md ("Signed and function operands of @").

## Context
T-1321 made `ANCHORARRAY(@IF(TRUE,D1))` spill D1# (`spill_anchor_operand`, reference_operators.rs). When the IF picks more than one cell, the engine implicitly intersects the pick at the formula cell (in H1, `ANCHORARRAY(@IF(TRUE,D1:D3))` spills from D1) and a picked union is #VALUE!. Neither is probed. Likewise `VLOOKUP(...,@IF(TRUE,B1))` (B1 text "2") and `LOOKUP(99,A1:A3,@IF(TRUE,F1))` keep IF as a value; unprobed.

## Acceptance criteria
- [ ] A probes case covers `ANCHORARRAY(@IF(TRUE,D1:D3))` on and off the D1:D3 rows, `ANCHORARRAY(@IF(TRUE,(D1,E1)))`, `ANCHORARRAY(@IF(FALSE,D1,5))` (engine #VALUE!, by analogy with `ANCHORARRAY(@1)`), `VLOOKUP(1,A1:C3,@IF(TRUE,B1))` with B1 text "2", and `LOOKUP(99,A1:A3,@IF(TRUE,F1))`, with oracle values.
- [ ] Findings recorded in docs/excel-semantics/references.md; engine mismatches filed as tasks.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Added probes/at-if-spill-anchor-operand (oracle-filled, excel-graph) and recorded the findings in docs/excel-semantics/references.md. One engine mismatch (LOOKUP result_vector `@IF`) filed as T-1368. Eval: probes +19 passing cells, 1 new failing cell (H50) pending T-1368.
