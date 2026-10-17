---
id: T-1645
title: "INDEX of a scalar reads it as a one-element array everywhere (12 probe cells pass; function-name-forms A4 was passing by accident)"
milestone: M2
lane: functions-lookup
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
INDEX reads a scalar first argument (a number, text or boolean) as a one-element array wherever it comes from, as Excel does, not only when IF or CHOOSE picks it.

## Context
T-1643 made `INDEX(CHOOSE(1,@B1:B6*1),3)` `#REF!` by wrapping a scalar picked by IF or CHOOSE (`index_impl` in functions/lookup_and_reference/index.rs). Generalising it to any scalar (the `index_select` fallback `Expecting a Range` #VALUE!) newly passed 12 more probe cells but regressed probes/catalog/function-name-forms A4 (`=IFERROR(INDEX(ERROR.TYPE(_xlfn.GROUPBY({1;2},{3;4},_xleta.SUM)),1,1),0)`, expected 0), which passed by accident: our GROUPBY result makes ERROR.TYPE a number where Excel's gives an error. Fix GROUPBY (or that cell's cause) first, then generalise. Find the 12 cells by trying the generalisation and diffing `make eval-baseline`.

## Acceptance criteria
- [x] INDEX of any scalar is a one-element array; function-name-forms A4 passes for its real reason; baseline shrinks, no regressions.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- `index_impl` (functions/lookup_and_reference/index.rs) wraps any scalar first argument with
  `scalar_as_array`, not only an IF or CHOOSE pick; `picks_a_branch` (functions/util.rs) lost its
  only caller and is removed.
- function-name-forms A4 needed no GROUPBY fix: ERROR.TYPE already lifts over GROUPBY's array, so
  `INDEX(...,1,1)` is `#N/A` and the IFERROR gives 0 for its real reason.
- The lifted-indices path wraps the scalar too (`scalar_as_one_element_array`, used by both paths).
- New probe probes/index-literal-scalar (excel-graph, 12 cells, all pass): literal and computed
  scalars, `#REF!` past them, lifted indices over a scalar.
- Eval: probes 2950 -> 2938 known failures (12 fixed: lookup-argument-modes H1, H18, H19, H24, H28,
  H30, H31, H39; m7-argument-modes H31, H39, H46, H55), no regressions; the new case adds 12
  passing cells. H1 passes only because our ADDRESS does not lift yet (T-1241); references.md says so.
