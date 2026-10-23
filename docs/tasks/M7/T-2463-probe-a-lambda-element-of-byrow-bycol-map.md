---
id: T-2463
title: "Probe a LAMBDA element of BYROW/BYCOL/MAP: ISERROR, IFERROR, ERROR.TYPE, TYPE over it; LET, group, @ and defined-name roots; MAKEARRAY 1x1 array and LAMBDA steps"
milestone: M7
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for what T-2406 left unprobed about a LAMBDA element of BYROW, BYCOL and MAP (a LAMBDA step), recorded in
docs/excel-semantics/arrays.md "MAKEARRAY with an array step; BYROW, BYCOL and MAP with a LAMBDA step", with a fix task filed for
every cell ours gets wrong.

## Context
Since T-2039 the element holds the LAMBDA itself (`ArrayNode::Lambda`): a cell shows `#CALC!`, a value read is `#VALUE!`, ISERROR
is FALSE over it and MAP can call it (arrays.md "Arrays of LAMBDAs", evidence from SCAN and MAKEARRAY). Probe that BYROW, BYCOL and
MAP elements behave the same, including `MAP(BYROW({1;2},LAMBDA(x,LAMBDA(v,v+x))),LAMBDA(g,g(10)))`.
Before T-2039, T-2406 made such an element `#CALC!` only when the call was the cell formula's outermost node and `#VALUE!` anywhere else, which
matches `&`, CELL and CONFIDENCE.NORM consumers (gen-m7/23b5367c18059af2 A2578, gen-m7/f7e94a817e5385ac A942, gen-m9/165db58d1540f014
A273). A bare LAMBDA value is not an error to ISERROR and is `#N/A` to ERROR.TYPE (arrays.md "An uncalled LAMBDA is one value"), so the
error-testing functions likely differ over the element. Probe with the `excel-probe` skill:
- `ISERROR`, `ISERR`, `ISNA`, `IFERROR`, `IFNA`, `ERROR.TYPE`, `TYPE`, `ROWS` over `BYROW({1;2},LAMBDA(x,LAMBDA(v,v)))`, and the same over
  BYCOL and MAP.
- The element where the call is not the bare root: `=LET(a,1,BYROW(..))`, `=(BYROW(..))`, `=@BYROW(..)`, a defined name holding the call,
  `=IF(TRUE,BYROW(..))`.
- MAKEARRAY with a 1x1 array step (`MAKEARRAY(2,2,LAMBDA(r,c,{5}))`) and with a LAMBDA step (`MAKEARRAY(2,2,LAMBDA(r,c,LAMBDA(v,v)))`).

## Acceptance criteria
- [ ] A probes case with oracle values for every formula above; the arrays.md entry states each result with its cells.
- [ ] A fix task filed (`make new-task`) for each group ours gets wrong, or the entry says ours matches.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing the fixes; arrays that hold LAMBDA values (T-2039).
