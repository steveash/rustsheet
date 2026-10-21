---
id: T-2229
title: "Probe IMAGE grid error option past-end elements in column shapes"
milestone: M1
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's elements are known for the column-shaped results of IMAGE with a grid source, a scalar error option and a wider array option, and the engine matches.

## Context
T-2205's probe (probes/rich-errors-image-grid-error-option) read column-shaped results with `INDEX(...,1,2)`, which is `#REF!` past the result's width, so rows 2-3 of the 3x2 result and the past-end element (3,1) of `IMAGE({url;url},"alt",1/0,{5;5;5})` are unprobed. Read them with `INDEX(...,r,c)` inside the result's bounds; record in information.md and fix `offline_args`/`fn_image` if the engine differs.

## Acceptance criteria
- [ ] Probe cells for elements (2..3,1..2) of the row-source/column-option case and (3,1) of the column/column case have excel-graph values; information.md states the rule; the engine matches or a follow-up is filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed (excel-graph, probes/rich-errors-image-grid-error-past-end, 18 cells): the row-source/column-option result is 3x2 of `#DIV/0!`; the column/column result is 3x1 with `#N/A` in the source's past-end row 3 (`NA()` option: `#N/A` throughout). The engine already matched, so no code change; information.md updated.
