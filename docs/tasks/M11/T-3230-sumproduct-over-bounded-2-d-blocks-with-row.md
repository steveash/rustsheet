---
id: T-3230
title: "SUMPRODUCT over bounded 2-D blocks with row and column masks (calipers pv26_002: 2,000 calls over I11:AW1450)"
milestone: M11
lane: perf
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
SUMPRODUCT over a bounded block multiplied by a row mask and a column mask costs about its matching
cells per call, not the whole block, with bit-identical results. The form is
`SUMPRODUCT(data*(header_row=x)*(key_column=y))`.

## Context
After T-3227, calipers `pv26_002` calculates in 71 s (release, 2026-10-30) on top of a 17 s load.
Its remaining time is SUMPRODUCT, by stack sampling: `fn_sumproduct`, `evaluate_operator_chain`,
`evaluate_range`. About 1,500 calls have the form
`IFERROR(SUMPRODUCT('Quarterly ARR Rollup'!$I$11:$AW$1450*('Quarterly ARR Rollup'!$I$6:$AW$6=D$3)*('Quarterly ARR Rollup'!$C$11:$C$1450=$C6)),"")`.
Each one builds and multiplies three arrays of 1,440 x 41 cells, about 59,000 elements each. The
header row and key column masks change per call, and the data block is shared.

Possible approach, to be checked: evaluate the broadcast masks first. Only the cells where both
masks are nonzero can contribute, but every element's reading must stay the walk's. An error
element anywhere in the data block is the result even where a mask is 0, and text is 0. So a
skipped cell must provably contribute exactly 0 and hide no error. A cached error-and-number
summary of the shared data block would answer that.

## Acceptance criteria
- [ ] Equivalence tests against today's evaluation: errors and text in masked-out cells, and
      shape mismatches.
- [ ] calipers `pv26_002` calculation time in the Outcome. If load plus calculation is now within
      60 s, its and `pv26_2_002`'s `exclude` entries go and the calipers baseline is updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- `pv26_002`'s 17 s load.
