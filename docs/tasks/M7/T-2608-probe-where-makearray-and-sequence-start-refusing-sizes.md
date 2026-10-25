---
id: T-2608
title: "Probe where MAKEARRAY and SEQUENCE start refusing sizes past the grid (columns 16385..123456789, rows 1048577..1E15)"
milestone: M7
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
arrays.md "Arrays past the grid" states where Excel's MAKEARRAY and SEQUENCE start answering `#VALUE!` for a size past the
grid, from oracle values.

## Context
T-2600 measured that Excel builds `MAKEARRAY(1048577,1,..)` and `SEQUENCE(1,16385)` (probes/t2579-array-value-edges B23,
B24) but answers `#VALUE!` for `SEQUENCE(3,123456789)` (gen-m9/f2bd5f380ec002bf A177), `MAKEARRAY(3,1E15,..)` (gen-m7
b2078b5d702c691a A2553) and `MAKEARRAY(1E15,4,..)` (gen-m7/8c4570978f18f4c2 A2736). `limits::check_built_array_size`
guesses: above our cell bound, SEQUENCE's columns past the grid are `#VALUE!`, and MAKEARRAY's columns and both functions'
rows `#VALUE!` from 2^31 (EXPAND's threshold). T-2452 found that Excel builds `MAKEARRAY(1,1048577,..)` and `MAKEARRAY(2,524289,..)`
(probes/t2452-makearray-*; `MAKEARRAY(1,123456789,..)` never opened in excel-graph). Probe, as `ROWS`/`COLUMNS` of the call, so
nothing spills, one cell per case (as probes/t2452-*): `SEQUENCE(1,N)` for N = 16386, 1048577, 2^24, 123456789;
`MAKEARRAY(1,N,LAMBDA(r,c,1))` for N = 2^24, 2^31-1, 2^31; `MAKEARRAY(N,1,LAMBDA(r,c,1))` for N = 2^24, 2^31-1, 2^31; and
`SEQUENCE(2,524289)`. Sizes Excel
builds may hang excel-graph (arrays.md "Stacked results past the grid"): split the probe so a hang loses few cells.

## Acceptance criteria
- [ ] A probe case with oracle values for the sizes above (or as many as excel-graph answers), cited in arrays.md.
- [ ] If the evidence contradicts `check_built_array_size`, an engine task is filed for it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Raising the cell bound (decision 0014).
