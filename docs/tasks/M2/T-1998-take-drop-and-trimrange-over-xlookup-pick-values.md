---
id: T-1998
title: "TAKE, DROP and TRIMRANGE over XLOOKUP pick values instead of the cells XLOOKUP returns"
milestone: M2
lane: functions-lookup
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
TAKE, DROP and TRIMRANGE over an XLOOKUP (and an XLOOKUP nested under IF/CHOOSE/LET in their first
argument) see what Excel sees: the cells XLOOKUP returns, if Excel does that, or its values, with
oracle evidence either way.

## Context
Found in T-1987. Only a direct SUBTOTAL/AREAS argument (and an intersection operand) evaluates
XLOOKUP for its reference (`Model::evaluate_reference_form`, `xlookup_impl(.., true)`);
`take_drop_impl` and `trimrange_impl` evaluate their first argument with
`evaluate_node_in_context`, so `fn_xlookup` gives values: `SUBTOTAL(9,TAKE(XLOOKUP(1,{1},H1:I1),1))`
spills `#VALUE!` and `SUBTOTAL(9,TAKE(XLOOKUP(XLOOKUP(1,H1:H2,H1:H2),H1:H2,H1:H2),1))` is `#VALUE!`.
Excel's XLOOKUP returns a reference, so Excel most likely sums the cells (unprobed: use the
`excel-probe` skill first). Static analysis (T-1987) counts a nested XLOOKUP as lifting
(`lifts_to_values` in expressions/parser/static_analysis/aggregate.rs); if the runtime starts
picking cells, that arm can narrow to match.

## Acceptance criteria
- [ ] Probe of SUBTOTAL/AREAS over TAKE, DROP and TRIMRANGE of XLOOKUP (scalar and lifting
  lookup values, direct and under IF/LET) recorded in `docs/excel-semantics/`.
- [ ] Engine matches the probe; debug tests show no scalar-context assertion (decision 0035).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The static analysis of T-1987, unless the runtime change lets it narrow.
