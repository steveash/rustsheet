---
id: T-2999
title: "Probe IFERROR, N and arithmetic over blank elements of FILTER, lifted INDEX and VSTACK arrays"
milestone: M7
lane: harness
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
docs/excel-semantics/arrays.md ("GROUPBY and PIVOTBY", the T-2854 paragraph) states, with excel-graph evidence, how
IFERROR, IFNA, N and arithmetic read a blank element of an array that is not a GROUPBY/PIVOTBY result.

## Context
Found by the T-2854 reviews. T-2854 made IFERROR and IFNA read every blank array element as 0 (`if_error_like`,
crates/rustsheet-engine/src/functions/logical/mod.rs), on evidence from GROUPBY blank keys only
(probes/groupby-sort-filter-args G!B15, B55) and the measured multi-cell range rule (T-2579). IF passes a blank element
through (probes/pivotby-ttest-array2 P!A41). Probe cases (tag M7, under eval/suites/probes/cases/), with D1:D5 holding
1, blank, "x", blank, 5:
- `COUNT(IFERROR(FILTER(D1:D5,TRUE),-1))`, `COUNT(IFERROR(INDEX(D1:D5,{1,2,3,4,5}),-1))`,
  `COUNT(IFERROR(VSTACK(D1:D5),-1))`, and the same with IFNA;
- `TEXTJOIN("|",FALSE,IFERROR(FILTER(D1:D5,TRUE),"#"))` and `TEXTJOIN("|",FALSE,FILTER(D1:D5,TRUE))`;
- `COUNT(N(FILTER(D1:D5,TRUE)))`, `COUNT(FILTER(D1:D5,TRUE)+0)`, `COUNT(IF(TRUE,FILTER(D1:D5,TRUE)))`.

## Acceptance criteria
- [ ] The probe cases are filled on excel-graph and in the probes baseline.
- [ ] arrays.md states the rule with the cells as evidence; an engine task is filed if ours differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
- GROUPBY/PIVOTBY blank keys (T-2854, done).
