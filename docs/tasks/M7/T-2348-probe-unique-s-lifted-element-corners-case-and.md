---
id: T-2348
title: "Probe UNIQUE's lifted-element corners: case and -0 entries, call errors, INDEX of other empty values"
milestone: M7
lane: functions-lookup
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-2300]
---

## Goal
Excel evidence for the corners of UNIQUE's lifted-element rule that T-2300 implemented by inference, and the engine matching it.

## Context
From T-2300's Excel-fidelity review. `CallResult::ArrayFirst` (functions/lift/driver.rs, `Later`, `all_elements_equal`) decides "entries all equal" by `ArrayNode` equality. It treats an element whose call returns an error as one that stays and is not the kept first element. INDEX of any empty value is `#VALUE!` (`scalar_as_one_element_array` in lookup_and_reference/index.rs). Only UNIQUE's empty element is evidenced (probes/m7-lift-take-drop-sortby L25, probes/m7-unique-lift-elements). Unprobed:
- text entries that differ only in case, `UNIQUE(D1:D3,FALSE,{"a";"A"})` (ours: they differ, because `ArrayNode` equality is case-sensitive), and `{0;-0}` (ours: equal);
- a call error from a non-error entry before a good one, `UNIQUE(D1:D3,FALSE,{"abc";FALSE;TRUE})` (ours: `#VALUE!`, "x", empty);
- INDEX of an empty value from another source, `INDEX(IFERROR(B1,1),1,1)` and `LET(x,IFERROR(B1,1),INDEX(x,1,1))` with B1 blank (ours `#VALUE!`; arrays.md "UNIQUE's and SORTBY's lifted elements" marks it UNVERIFIED).
Write the probe case per the excel-probe skill (ROWS and per element ISBLANK, `&"|"`, TYPE and SUM, as m7-unique-lift-elements does), fill it on excel-graph, and fix whatever differs.

## Acceptance criteria
- [ ] A probe case with these cells, filled from excel-graph; its findings recorded in arrays.md (the UNVERIFIED line resolved).
- [ ] Every cell of it passes, or each failing cell has its own task.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
