---
id: T-1685
title: "Probe AND/OR/XOR of text TRUE in a cell picked by IF, INDEX, XLOOKUP, CHOOSE, LET, TAKE or OFFSET"
milestone: M2
lane: functions-logic
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for whether AND, OR and XOR count a text "TRUE"/"FALSE" cell that a function picks
as a reference, and the engine matching it.

## Context
AND/OR/XOR count direct text "TRUE"/"FALSE" and skip referenced text (probes/and-or-xor-text-arguments,
T-0258). They read arguments with `ArgReading::InContext` (`evaluate_node_in_context`; T-1677,
SPEC-0009 "Consumer mapping"), not `ArgReading::Aggregate`, which reads picked cells as references.
With A1 `="TRUE"` ours gives today (T-1677, unprobed): `AND(IF(TRUE,A1))`, `AND(INDEX(A1:A2,1))`,
`AND(XLOOKUP(1,{1},A1:A1))` and `AND(LET(x,A1,x))` TRUE (the text counts), but `AND(CHOOSE(1,A1))`,
`AND(TAKE(A1:A2,1))` and `AND(OFFSET(A1,0,0))` `#VALUE!` (skipped). Excel reads picked cells as
references for SUM (probes/aggregate-picked-direct-a-variants), so the first group may be wrong.
Write a probe case (excel-probe skill) with those seven formulas, for AND, OR and XOR, plus a
"FALSE" cell; fill it with the oracle. If Excel skips every pick, switch the logical policy to
`ArgReading::Aggregate` (and keep the cross-sheet and LAMBDA errors) in
functions/logical/and_or_xor_not.rs.

## Acceptance criteria
- [ ] An oracle-filled probe case covers the picks above; ours matches every cell, or the
  remaining gap is filed as a task.
- [ ] docs/excel-semantics/coercion.md records the rule with the probe as evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe probes/and-or-xor-picked-text (61 cells) filled by excel-graph: Excel skips a text TRUE/FALSE cell for every pick, so ours was wrong for IF, INDEX, XLOOKUP and LET. AND/OR/XOR now use `ArgReading::Aggregate`; `InContext` removed. Rule recorded in coercion.md and SPEC-0009.
