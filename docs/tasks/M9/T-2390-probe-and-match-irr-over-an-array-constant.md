---
id: T-2390
title: "Probe and match IRR over an array constant holding an error: #VALUE!, not the error"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`IRR` over an array constant with an error element gives Excel's `#VALUE!`, not the element's error.

## Context
- Found by the T-2248 smoke check (one formula per financial edge-pool entry, excel-graph, 2026-10-22; the scratch
  suite was deleted, so the probe is this task's to commit). docs/excel-semantics/financial.md "Financial edge-pool
  smoke check (T-2248)" lists the observed values.
- Observed: `IRR({-100,#N/A,60})` is `#VALUE!` in Excel; ours `#N/A`. Text and TRUE in the array are skipped
  (`IRR({-100,"a",60})` = `IRR({-100,TRUE,60})` = -0.40000000000000013, both match).
- Probe also: other error kinds (`#DIV/0!`, `#NUM!`), an error cell in a range (`IRR(A1:A3)`), and MIRR, NPV, XIRR and
  XNPV with the same array, so the rule is known for the family. Code: `fn_irr` in functions/financial/mod.rs.

## Acceptance criteria
- [ ] A probe case (excel-probe skill) holds the cells above, filled through excel-graph.
- [ ] IRR matches Excel on every probed cell; the financial.md entry is VERIFIED with the rule.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Error order between arguments (T-2103).
