---
id: T-1847
title: "Probe the legacy @ on array values beyond the T-1717 evidence"
milestone: M2
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for where a legacy (non-array) formula puts the `@` on array values in the shapes T-1717 implemented by extrapolation, so each is either confirmed or filed as a fix.

## Context
T-1717 (static_analysis/legacy.rs `legacy_array_context`, `returns_an_argument`,
`legacy_array_call_at`, `leaves_call_whole`) generalizes beyond probes/legacy-array-result,
legacy-n-t-reference-args and t-n-range-top-left. Unprobed: IFNA, IFS and SWITCH taking the `@` on
the call (`=IFNA({2;3},0)`, `=IFS(1,{2;3})`, `=SWITCH(1,1,{2;3})`); an array-valued call inside
IFERROR (`=IFERROR(ABS({-2;3}),0)`: `@` on the IFERROR or on `{-2;3}`?); T over SWITCH
(`=T(SWITCH(1,1,{"a";"b"}))`); other scalar functions over an array constant
(`=ISNUMBER(SEARCH({"a","b"},A1))`, `=LEN({"ab";"c"})`); a one-cell array-valued non-CHOOSE call
(`=ABS(TRANSPOSE({5}))`, ours adds no `@`); an array constant inside an operator (`=N({2;3}+0)`,
`=ABS({2;3}+0)`, ours adds none and spills). docs/excel-semantics/references.md "Legacy formulas
put `@` on array-valued calls and array constants". Use the excel-probe skill: a legacy cell per
shape, its dynamic twin, and FORMULATEXT.

## Acceptance criteria
- [ ] A probe case under eval/suites/probes/cases with excel-graph oracle values for every shape above (legacy cell, the cell below it, FORMULATEXT).
- [ ] references.md records the outcome per shape; each mismatch with ours is filed as an engine task (known failures recorded with `make eval-baseline`).
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Added probes/legacy-array-at-shapes (excel-graph) for every shape; recorded per shape in
references.md. Confirmed: IFNA, IFS, IFERROR argument, T over SWITCH, SEARCH/LEN constants, one-cell
TRANSPOSE. Mismatches filed: T-1910 (top-level SWITCH), T-1911 (array constant inside an operator).
Eval: 40 cells checked, 36 pass, 6 known failures baselined (incl. the two unexpected spill cells I11, J11).
