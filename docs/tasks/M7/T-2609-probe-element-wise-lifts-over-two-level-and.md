---
id: T-2609
title: "Probe element-wise lifts over two-level and reference-body GROUPBY cells (SPEC-0018 rules 4, 7, 8)"
milestone: M7
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
arrays.md and SPEC-0018 state, from oracle values, what an element-wise lift reads from a two-level nested GROUPBY cell and from a
reference-body cell.

## Context
SPEC-0018 (revised by T-2602) assumes that `ArrayNode::scalar()` reads a two-level nested cell's top-left recursively and that a
reference-body cell (`LAMBDA(x,D!D2)`) is `#VALUE!` to every read but INDEX (rule 7, probes/t2592-nested-groupby-cells A21).
Neither is probed. Probe with the t2592 data sheet D (`GROUPBY(D!A1:A5,D!C1:D5,..)`):
- `SUM(--ISERROR(GROUPBY(D!A1:A5,D!C1:D5,LAMBDA(x,D!D2))))`, `SUM(--ISNUMBER(..))` and `ROWS(..)`;
- the same over the A23 two-level formula, a LAMBDA returning a GROUPBY whose inner top-left is an error;
- `REGEXEXTRACT("a1",GROUPBY(..,LAMBDA(x,D!D2)))`;
- SUM, COUNT and COUNTA over the reference-body result;
- INDEX of a reference-body cell whose referenced cell is blank or an error (T-2593 holds the blank or the error, unprobed);
- an operator over the whole result read element by element (`SUM(GROUPBY(..,LAMBDA(v,v))+0)`, `ISERROR(GROUPBY(..)+0)`) and a
  lifted INDEX element of a nested cell (`TYPE(INDEX(GROUPBY(..,LAMBDA(v,v)),{2;3},2))`): T-2593 reads both as `#VALUE!`.

Partial evidence already on file (T-2623): over a `LAMBDA(z,z)` result, `MEDIAN({1;2;"";4},GROUPBY(..)-x,..)` is #N/A in Excel
and `TEXTJOIN(";",FALSE,IFERROR((GROUPBY(..)-x)&"",..))` fails (probes/t2475-stat-subexpressions P!A62, A63); ours reads each nested
cell as `#VALUE!` and gives #VALUE!. The fix is T-2686.
Record the answers in arrays.md (the T-2592 paragraph) and update SPEC-0018's "One reader" and rule 7. If the evidence changes
what T-2593 or T-2594 must build, update those task bodies too.

## Acceptance criteria
- [ ] A probe case with oracle values for the formulas above, cited in arrays.md and SPEC-0018.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (T-2593, T-2594).
