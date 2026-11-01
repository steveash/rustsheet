---
id: T-3663
title: "Probe whether a cleared table cell is blank or empty text (T-3025 follow-up)"
milestone: M2
lane: harness
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Know whether the cleared C3 in probes/t3025-blank-beside-calculated is truly blank, and what the
excel-graph backend's `{"entered": ""}` (a PATCH of `values: [[""]]`, tools/src/rustsheet_tools/oracle/
backends.py) stores in a cell, so references.md rule 8(e) rests on a blank, not on empty text.

## Context
- From the T-3025 Excel-fidelity review (MINOR). It was first filed as T-3328 in draft PR #2495, which
  never merged; that id now names another task.
- probes/t3025-blank-beside-calculated reads only `FORMULATEXT(C3)` = `#N/A`. That proves C3 holds no
  formula, not that it is blank.
- If Graph stores empty text, T-3025's result only repeats probes/t2957-constant-then-refill. Then
  references.md rule 8(e) ("a blank beside a real calculated column blocks the re-fill") would rest on a
  constant, not on a blank.

## Acceptance criteria
- [ ] A new probe case modelled on t3025-blank-beside-calculated adds `=ISBLANK(C3)`, `=LEN(C3)` and
      `=ISTEXT(C3)`, and reads C2:C4 before the last edit if the spec adapter allows. It is filled on
      excel-graph and baselined (`make eval-baseline`).
- [ ] references.md rule 8(e) says whether the cleared cell is blank, and what a Graph `""` entry does.
- [ ] If ours differs, an engine task is filed with `make new-task`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other table-growth rules (T-3218, T-3296).
