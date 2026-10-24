---
id: T-2477
title: "Probe UNIQUE, SORT, XMATCH and comparisons over arrays of LAMBDAs, and a LAMBDA element against an error"
milestone: M7
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for the consumers of an array of LAMBDAs that T-2039 left unprobed, recorded in docs/excel-semantics/arrays.md
"Arrays of LAMBDAs", with a fix task filed for every cell ours gets wrong.

## Context
T-2039 lets an array element hold a LAMBDA (`ArrayNode::Lambda`). Unprobed, with ours in brackets: UNIQUE over
`SCAN(LAMBDA(z,z),{1,2,3},LAMBDA(f,x,LAMBDA(z,f(z)*10+x)))` and over `VSTACK(L,L)` with the same LET-bound L (ours keys each LAMBDA
by its id, `CellKey::Lambda`: three, and one for the repeated L), SORT and SORTBY over it, XMATCH/MATCH of a LAMBDA in it, `=` and
`<` between two LAMBDA elements (ours `#VALUE!`), and a LAMBDA element against a real error in arithmetic,
`SCAN(..)+{#N/A,1,1}` (ours: the left one, `own_error` reads the LAMBDA as `#VALUE!`). Use the `excel-probe` skill; probes/lambda-arrays
holds the formulas probed so far.

## Acceptance criteria
- [ ] A probes case with oracle values for every formula above; the arrays.md entry states each result with its cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
