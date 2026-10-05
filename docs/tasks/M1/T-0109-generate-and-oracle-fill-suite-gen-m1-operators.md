---
id: T-0109
title: "Generate and oracle-fill suite gen-m1 (operators and references only)"
milestone: M1
lane: harness
status: done
priority: P1
size: S
tier: opus
depends_on: [T-0117]
---

## Goal
≥ 3,000 oracle-backed cells exercising M1 semantics.

## Context
`rustsheet-gen --milestone M1`.

## Acceptance criteria
- [x] Before the bulk fill, one smoke workbook with one formula per `category/form` tag of
  `rustsheet-gen --policy operators-refs` is filled through the oracle. Excel must load it without
  repairing it. Check especially the non-canonical literals (`.5`, `1.`, `007`, `1e3`),
  `Data!A1:A1:Data!C3`, `Data!A1:Data!C3`, cross-sheet intersections and error literals as
  operands. Any form Excel rejects is fixed in the generator first. Any surprise goes to
  docs/excel-semantics/ (syntax.md or references.md).
- [x] Suite gen-m1 with sidecars committed; gate = true once filled; baseline recorded; gap tasks filed by gap-triager.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`gen-m1` is generated with `rustsheet-gen --policy operators-refs --seed 0`: 15 cases, 3,750
cells. The excel-graph oracle filled it on 2026-10-05 with 0 failures. The suite is now
`gate = true`, and `eval/baseline/gen-m1.txt` is recorded. The engine passes 2,772 of 3,750
(73.9%); the other suites are unchanged.

A smoke workbook with one formula per form tag (109 formulas) loaded in Excel without repair
before the bulk fill.

The gap-triager split the 978 failures into these tasks:
- T-0105, intersection: 632 cells.
- T-0135, union: 250 (value-context `#VALUE!`; general multi-area design in spec T-0133).
- T-0130, range with both corners sheet-qualified: 41.
- T-0107, number→text in `&`: 34.
- T-0131, text in range arithmetic: 10.
- T-0221, last digit of `^`: 8.
- T-0132, `--x`: 3.

The new quirks are in docs/excel-semantics/: references.md, coercion.md, syntax.md and
numbers.md.
