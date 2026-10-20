---
id: T-2193
title: "Probe TAKE and DROP with a lifted count over an array of references"
milestone: M7
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for TAKE and DROP whose array is an array of references and whose rows or cols count is an array,
such as `SUM(TAKE(OFFSET(A1,{0;1;2},0),{1;2}))` and `ROWS(DROP(OFFSET(A1,{0;1;2},0),{1;2}))`.

## Context
T-2174 made TAKE and DROP keep the elements of an array of references for an unlifted count (SPEC-0014 rule 7).
A lifted count over such a grid is unprobed. For now the engine reads the grid's value view (`take_drop_reference`
in functions/lookup_and_reference/drop_take.rs, `TODO(T-2193)`). Write the probe with the `excel-probe` skill.

## Acceptance criteria
- [ ] A probes case covers TAKE and DROP with lifted rows, lifted cols and both, over a lifted OFFSET, read by SUM,
  ROWS and INDEX. It is oracle-filled (excel-graph) and committed.
- [ ] The finding is in `docs/excel-semantics/`, and an engine task is filed if the engine differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The engine change itself, which goes in the follow-up task.
