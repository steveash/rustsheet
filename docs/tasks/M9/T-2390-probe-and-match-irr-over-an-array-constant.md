---
id: T-2390
title: "Probe and match IRR over an array constant holding an error: #VALUE!, not the error"
milestone: M9
lane: functions-financial
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: visible
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

T-2257 sweep (gen-m9 financial; gap triage 2026-10-23): `IRR({-100,#N/A,60})` and `IRR({-100,#N/A,60},DAY(42519))` are `#VALUE!` in the gen-m9 financial sweep (ours `#N/A`): the evidence this task asked for.

Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 2 known-failure lines in eval/baseline/gen-m9.txt, 2 visibly off.
Examples:
- gen-m9/79d87fc527927626 Calc!A320 `=IRR({-100,#N/A,60})`: Excel #VALUE!, ours #N/A
Cells:
- gen-m9/79d87fc527927626.case.json (1): Calc!A320
- gen-m9/abb7d565af89b849.case.json (1): Calc!A352

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2257): the 2 visibly-off known-failure lines listed under "T-2257 sweep" in Context disappear
      from eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] A probe case (excel-probe skill) holds the cells above, filled through excel-graph.
- [ ] IRR matches Excel on every probed cell; the financial.md entry is VERIFIED with the rule.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Error order between arguments (T-2103).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2257): the gen-m9 cells listed under "T-2257 sweep" in Context are visibly off (wrong
type, error, text, or a value off by more than half a unit of the 15th significant digit), so the task is in the campaign. Its
target is that every one of them has Excel's type, error and text, or is within half a unit of the 15th significant digit of
Excel's value; the T-2257 visible-parity criterion under Acceptance criteria states it. Bit-level criteria still unmet then move
to a follow-up task filed without the key.
