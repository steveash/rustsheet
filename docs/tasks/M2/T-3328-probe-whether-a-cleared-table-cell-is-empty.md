---
id: T-3328
title: "Probe whether a cleared table cell is empty or empty text (T-3025 follow-up)"
milestone: M2
lane: harness
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Establish whether the cleared C3 in probes/t3025-blank-beside-calculated is a truly empty cell or empty text, and what the Graph backend's `{"entered": ""}` (PATCH `values: [[""]]`) does to a cell.

## Context
The T-3025 Excel-fidelity review (MINOR): the case reads only `FORMULATEXT(C3)` = `#N/A`, which shows C3 holds no formula but not that it is blank. If Graph stored empty text, the probe would repeat t2957-constant-then-refill. Add `=ISBLANK(C3)` and `=LEN(C3)` cells to a new probe modelled on t3025 (also read C2:C4 before the last edit if the adapter allows), fill it on excel-graph, and record in references.md rule 8(e) what a Graph `""` entry does. Backend code: tools/src/rustsheet_tools/oracle/backends.py.

## Acceptance criteria
- [ ] Probe case filled on excel-graph and baselined; references.md rule 8(e) states whether the cleared cell is blank; an engine task is filed if ours differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
