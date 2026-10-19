---
id: T-1956
title: "OFFSET lifts rows, cols, height and width into an array of references"
milestone: M6
lane: functions-lookup
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-1955]
---

## Goal
OFFSET's rows, cols, height and width lift over a multi-cell range or an array into an array of references, as Excel does, on the carrier of the T-1955 spec.

## Context
Split from T-1810. Evidence: docs/excel-semantics/lookup.md "OFFSET, INDIRECT and TRIMRANGE lift into arrays of references" (probes/reference-lift-elements, excel-graph, 2026-10-19). Failing cells: probes/lookup-argument-modes F20:H21; probes/reference-lift-elements F1:H5, H11, G12, H12, F15:H15 and the spills K1, K4, K13 (and their members); gen-m7/cb748b6a9b81fbb7 Calc!A28 `=OFFSET(..,OFFSET(Data!D6:D9,3,(Data!D$4+7)))` (Excel `#SPILL!`, the lift winning over the `#N/A` scalar argument; engine `#N/A`). `rustsheet-eval show probes/reference-lift-elements.case.json` lists them.

## Acceptance criteria
- [ ] The OFFSET cells above pass; eval/baseline updated with no new known failures.
- [ ] lookup.md's "Implemented in" line names the code.
- [ ] `make check` green; reviewers approve.

## Out of scope
- INDIRECT and TRIMRANGE (T-1957).
