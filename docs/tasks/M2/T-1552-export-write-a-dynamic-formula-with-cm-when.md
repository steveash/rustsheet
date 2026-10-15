---
id: T-1552
title: "Export: write a dynamic formula with cm when a legacy reading would intersect it"
milestone: M2
lane: engine
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A formula typed in RustSheet keeps its dynamic-array value after an xlsx round trip through Excel.

## Context
Found in T-1526's review. A dynamic formula with a scalar static result, such as `=SUM(B1:B6+0)`
or `=SUMPRODUCT(IF(B1:B6>1,B1:B6,0))`, is exported as a plain `<f>` with no `cm`. Excel (and our
own import, `add_implicit_intersection`) reads it back as legacy and intersects it
(docs/excel-semantics/references.md "Legacy formulas intersect the ranges in value arguments"), so
the value changes. Export should write the dynamic-array metadata (`cm`, or whatever Excel writes for
a single-cell dynamic formula; check an Excel-saved workbook) whenever `add_implicit_intersection` on
the formula would add an `@`, or otherwise preserve the meaning.

## Acceptance criteria
- [ ] Export then import of `=SUM(B1:B6+0)` and `=SUMPRODUCT(IF(B1:B6>1,B1:B6,0))` gives the same value
      off the range's rows (12 and 10 with B1:B6 = 1,2,3,1,2,3).
- [ ] The written xlsx carries what Excel writes for such a formula (evidence cited).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
