---
id: T-1756
title: "Probe the reach of the unknown-call-name spelling table"
milestone: M2
lane: research
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for how far Excel's workbook-wide spelling of unknown call names reaches, so `CallNameSpellings` can be checked against it; each finding recorded in docs/excel-semantics/references.md "Unknown function call names".

## Context
T-1719 implemented "first spelling met wins, case-insensitively" for free callees (probes/unknown-function-name-spelling). The reviewers flagged these gaps, none of them probed yet: (1) a call of a defined LAMBDA name typed in another case (`MyFn` defined, `=myfn(1)`): defined name's spelling or the first typed one? (2) a LET/LAMBDA-bound callee typed in another case (`=LET(f,LAMBDA(x,x*2),F(3))`; ours prints `f(3)`, a hand-written test in test_fn_formulatext.rs); (3) whether an unknown name used as a value (`=foo+1`) shares the table with callees; (4) first-met order across sheets (Sheet2 before Sheet1 in the xlsx?); (5) non-ASCII case folding (`=ÄBC(1)` then `=äbc(1)`). Load-order probes only: edits after load (edit order, deleting the first user) need the sheet_ops replay (T-1608) if it can express them.

## Acceptance criteria
- [ ] A probe case with oracle values covers points 1-5; the references.md entry states each result; any mismatch with ours is filed as an engine task.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
