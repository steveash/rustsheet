---
id: T-1364
title: "Probe reference-only checks through written parentheses and the (INDEX(..))# shape"
milestone: M1
lane: research
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence that a reference survives written parentheses for ranges and LET bindings, and for a group directly before `#`.

## Context
T-1339 (probes/written-parens-reference-context) could not tell a range from an array with ROWS, and excel-graph rejects `=(INDEX(D1:D3,1))#`. Probe reference-only checks: `ISREF((A1:A3))`, `ISREF(LET(x,(A1:A3),x))`, `OFFSET((A1:A3),1,0)`, `CELL("address",(A1))`, `ANCHORARRAY(@IF(TRUE,D1))`; for the `)#` shape try `=ANCHORARRAY((INDEX(D1:D3,1)))` and read FORMULATEXT, or note it for desktop Excel (M14, decision 0039). Record in references.md; flag any non-transparent cell in SPEC-0007 before T-1340.

## Acceptance criteria
- [ ] A probes case (excel-graph) covers the cells above; findings recorded in references.md and SPEC-0007.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
