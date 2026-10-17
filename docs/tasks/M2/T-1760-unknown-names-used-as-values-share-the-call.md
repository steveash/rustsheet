---
id: T-1760
title: "Unknown names used as values share the call-name spelling table"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
FORMULATEXT of an unknown name used as a value and an unknown call of the same name (either order) print one spelling, the first met.

## Context
Excel: `=Qux+1` then `=qux(1)` prints `=Qux(1)`; `=wib(1)` then `=Wib+1` prints `=wib+1`. Ours keeps value-use names out of the table. Failing cells: probes/unknown-function-name-reach Sheet1!B9, B11. Semantics: docs/excel-semantics/references.md "Unknown function call names" (T-1756); registry `CallNameSpellings` (T-1719).

## Acceptance criteria
- [ ] Sheet1!B9 and B11 of probes/unknown-function-name-reach pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other cells of probes/unknown-function-name-reach.
