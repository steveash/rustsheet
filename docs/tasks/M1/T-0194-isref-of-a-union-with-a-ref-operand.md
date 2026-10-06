---
id: T-0194
title: "ISREF of a union with a #REF! operand is FALSE"
milestone: M1
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`ISREF((A1,#REF!))` is FALSE, as in Excel.

## Context
T-0187 probe (excel-graph, 2026-10-06): probes/union-accepted H9 `=ISREF((A1,#REF!))` is FALSE; the engine gives TRUE. `=(A1,#REF!)` and `=(#REF!,A1) A1` are `#REF!` (H1, H2), so a union with an error operand evaluates to the error, not a reference. See docs/excel-semantics/references.md, "Union: parsing and evaluation".

## Acceptance criteria
- [ ] probes/union-accepted H9 passes; baseline shrinks.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other ISREF operand kinds.
