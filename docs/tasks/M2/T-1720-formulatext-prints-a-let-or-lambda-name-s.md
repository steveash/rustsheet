---
id: T-1720
title: "FORMULATEXT prints a LET or LAMBDA name's uses in the spelling of its declaration"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The formula text of a LET or LAMBDA shows each use of a bound name in the spelling of its declaration, as Excel does: `=LET(a1.b2,5,A1.B2*2)` reads back as `=LET(a1.b2,5,a1.b2*2)`.

## Context
Found by T-1659: probes/lotus-dot-name-let-lower E2 (excel-graph; the file holds the uses as `_xlpm.A1.B2`, spec2xlsx `prefix_bound_names`). Ours prints each use as typed (`variable_text`, parser/stringify.rs). Probe first whether this holds for plain names too (`=LET(x,5,X*2)`, `=LAMBDA(Ab,aB*2)(4)`, a nested shadowing `LET(x,1,LET(X,5,x))`) with FORMULATEXT, then implement for every bound name. Record the rule in docs/excel-semantics/references.md next to "LET variable names and LAMBDA parameters are case-insensitive".

## Acceptance criteria
- [ ] probes/lotus-dot-name-let-lower E2 and the new probe cells pass; a reload keeps the text.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
