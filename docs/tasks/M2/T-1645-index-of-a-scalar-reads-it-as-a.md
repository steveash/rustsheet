---
id: T-1645
title: "INDEX of a scalar reads it as a one-element array everywhere (12 probe cells pass; function-name-forms A4 was passing by accident)"
milestone: M2
lane: functions-lookup
status: in-progress
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
- [ ] INDEX of any scalar is a one-element array; function-name-forms A4 passes for its real reason; baseline shrinks, no regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
