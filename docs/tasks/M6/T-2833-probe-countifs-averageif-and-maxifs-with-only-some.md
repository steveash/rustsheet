---
id: T-2833
title: "Probe COUNTIFS, AVERAGEIF and MAXIFS with only some ranges lifted over an array of references"
milestone: M6
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for which arguments of COUNTIFS, AVERAGEIF and MAXIFS lift per element over an array of references
when only some of their ranges are lifted, so the positions T-2823 inferred from SUMIF/SUMIFS are verified or corrected.

## Context
T-2823 made COUNTIFS (every criteria range), AVERAGEIF (range and average_range) and MAXIFS (max_range and every
criteria range) per-element callers (`Function::per_reference_element`, functions/lookup_and_reference/ref_lift.rs),
copying SUMIF's and SUMIFS's probed positions. The T-2065 probes (probes/t2065-lift-*) cover only one lifted COUNTIFS
range, AVERAGEIF's lifted range and MAXIFS with both ranges lifted. Unprobed: `MAXIFS(OFFSET(A1,{0;1},0),A1:A2,">0")`
and `MAXIFS(A1,OFFSET(A1,{0;1},0),">0")`; COUNTIFS with two criteria pairs, one range lifted;
`AVERAGEIF(A1,">0",OFFSET(A1,{0;1},0))`. Data as probes/reference-lift-consumers-b2. Use the excel-probe skill, one
formula per workbook (Excel for the web refused some combined workbooks, T-2065). docs/excel-semantics/lookup.md
"Statistical aggregates over a lifted OFFSET".

## Acceptance criteria
- [ ] A probe case per form above with excel-graph oracle values committed under eval/suites/probes.
- [ ] lookup.md records the values; any form ours gets wrong has a named engine follow-up task.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
