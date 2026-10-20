---
id: T-2120
title: "Probe TRIMRANGE lifted over an error, multi-area or empty source with an invalid trim element"
milestone: M6
lane: functions-lookup
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's results for the three TRIMRANGE lift cases T-2089 left unprobed are known, recorded and matched by the engine.

## Context
T-2089 (probes/trimrange-lift-sources) verified TRIMRANGE lifted over array, computed-reference and error sources.
Its lookup.md entry "TRIMRANGE lifted over an array, a computed reference or an error" lists three cases as unprobed,
each implemented by guess: an error source with an invalid trim element (`TRIMRANGE(NA(),{0;5})`; ours: `#N/A` in
both elements, the source's error winning), a multi-area source (`TRIMRANGE((A1,B2),{0;1})`; ours: `#VALUE!` per
element) and a source with no cells under a lifted trim (ours: one `#REF!`). Probe each with `ROWS`,
`ERROR.TYPE(INDEX(…,i,1))` and a bare spill, following that probe's layout. Code: `Model::lifted_trimrange`
(functions/lookup_and_reference/trimrange.rs).

## Acceptance criteria
- [ ] A probe case holds the three questions with excel-graph values; lookup.md records them as VERIFIED and the engine matches (no regression).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other producers' lifts (OFFSET, INDIRECT).
