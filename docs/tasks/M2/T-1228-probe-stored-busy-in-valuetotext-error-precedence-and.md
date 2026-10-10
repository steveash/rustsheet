---
id: T-1228
title: "Probe stored #BUSY! in VALUETOTEXT, ARRAYTOTEXT, IFERROR, error precedence and COUNTIF criteria"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for the stored-`#BUSY!` behaviors that SPEC-0003's amendment (T-1020) assumes,
with the engine matching once T-1222 lands.

## Context
SPEC-0003 ("Stored `#BUSY!`") treats a stored `<c t="e"><v>#BUSY!</v></c>` as an ordinary error kind
with ERROR.TYPE 0 (probes/error-stored-constants S!B1:I1). These parts are UNVERIFIED there:
- `VALUETOTEXT(A1)` and `ARRAYTOTEXT(A1)` give the text "#BUSY!" (as for the other newer kinds).
- Leftmost error wins: `ERROR.TYPE(A1+#DIV/0!)` 0 and `ERROR.TYPE(#DIV/0!+A1)` 2, also for `=`, `*`
  and `SUM(A1,1/0)`.
- COUNTIF criteria: `COUNTIF(A1:A2,"#BUSY!")` counts 0 and `"<>#BUSY!"` counts the cell, as for
  `#BLOCKED!` (T-1012); the spec keeps `#BUSY!` out of `is_english_error_string`.
- `IFERROR(A1,"y")` gives "y", and `ERROR.TYPE(A1:A2)` spills 0 for the `#BUSY!` cell.
- `UNIQUE` over a stored `#BUSY!` and a `#NULL!` keeps both (the engine keys errors by ERROR.TYPE).
Use stored constants (`{"t":"e","v":"#BUSY!","stored":true}`, see syntax.md "Newer error literals").
`=#BUSY!` cannot be used: Excel rejects the workbook.

## Acceptance criteria
- [ ] A probe case with these cells, oracle-filled (excel-graph, excel-probe skill).
- [ ] syntax.md and SPEC-0003 record the results (UNVERIFIED → VERIFIED, or the design is amended);
      any engine mismatch after T-1222 has its own task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Typed `#BUSY!` cell input (desktop-only, like T-1016).
