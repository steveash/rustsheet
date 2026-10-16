---
id: T-1692
title: "INDIRECT text naming a sheet that does not exist is 0 and ISREF TRUE in excel-graph"
milestone: M8
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
INDIRECT text that names a sheet the workbook does not have gives what excel-graph gives:
`INDIRECT("Nope!A1")` is 0 and `ISREF(INDIRECT("Nope!A1"))` is TRUE, not `#REF!`.

## Context
Found by the T-1608 sheet-op probes:
- `probes/sheet-ops/sheet-ops-indirect-missing-sheet-control` (no sheet edits).
- `sheet-ops-indirect-missing-sheet-after-add`.
- `sheet-ops-rename-indirect-text` (`INDIRECT("Data!A1")` after Data was renamed).
- `sheet-ops-add-resolves-indirect`.
- `sheet-ops-add-then-rename`.
In excel-graph, INDIRECT of `Nope!A1`, `'No Sheet'!A1` or a range `Nope!A1:A2` is a reference
(ISREF TRUE, ISERROR FALSE) whose value is 0. Desktop Excel is commonly believed to give `#REF!`;
that is unverified, and decision 0039 makes excel-graph the oracle of record. The engine gives
`#REF!`. Find out what the reference is (a phantom sheet, an external reference?) by probing:
- ROWS and COLUMNS of it
- SUM and COUNTA over it
- CELL("address", ...)
Then implement the smallest rule that matches.
docs/excel-semantics/references.md, "INDIRECT text naming a missing sheet".

## Acceptance criteria
- [ ] A follow-up probe pins down the reference's shape and values.
- [ ] The five probe cases' INDIRECT cells pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The desktop question (M14).
