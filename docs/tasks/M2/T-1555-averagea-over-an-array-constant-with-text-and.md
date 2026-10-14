---
id: T-1555
title: "AVERAGEA over an array constant with text and a boolean: {1,a,TRUE} is 0.5"
milestone: M2
lane: functions-stat
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Dynamic `AVERAGEA({1,"a",TRUE})` gives Excel's 0.5, not 0.667.

## Context
Found by T-1551: probes/legacy-value-argument-intersection-b AI2 (excel-graph) is 0.5; we give 0.6666666666666666 (T-1526
engine choice). Work out Excel's rule for text and booleans inside an array constant under AVERAGEA (probe variants:
`{1,"a"}`, `{1,TRUE}`, `{"a",TRUE}`, `{1,"1"}`), record it in docs/excel-semantics/ (statistical) and match it.

## Acceptance criteria
- [x] AI2 of probes/legacy-value-argument-intersection-b passes and the variant probes are recorded.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Excel reads an array value's elements under AVERAGEA, MAXA and MINA unlike range cells: booleans are
skipped, text (numeric text and "" included) is 0 for AVERAGEA and skipped by MAXA/MINA. New probe
probes/averagea-array-constant-elements (25 cells, excel-graph) evidences it; `for_each_value_a` now
takes an `ArrayText` mode, and a cached range of a linked workbook keeps the reference coercion via
`external_reference_cells`. MAXA and MINA with no value read are 0, as MAX is (they were `#VALUE!`). All 25 cells pass and legacy-value-argument-intersection-b AI2 is fixed
(probes known failures 5618 -> 5617); no regressions in any suite, poi included. Recorded in
statistics.md "AVERAGEA, MAXA and MINA over an array".
