---
id: T-0109
title: "Generate and oracle-fill suite gen-m1 (operators and references only)"
milestone: M1
lane: harness
status: in-progress
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
- [ ] Before the bulk fill, one smoke workbook with one formula per `category/form` tag of
  `rustsheet-gen --policy operators-refs` is filled through the oracle. Excel must load it without
  repairing it. Check especially the non-canonical literals (`.5`, `1.`, `007`, `1e3`),
  `Data!A1:A1:Data!C3`, `Data!A1:Data!C3`, cross-sheet intersections and error literals as
  operands. Any form Excel rejects is fixed in the generator first. Any surprise goes to
  docs/excel-semantics/ (syntax.md or references.md).
- [ ] Suite gen-m1 with sidecars committed; gate = true once filled; baseline recorded; gap tasks filed by gap-triager.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
