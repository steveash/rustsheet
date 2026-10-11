---
id: T-1315
title: "Probe SUM error precedence: stored #BUSY!/#N/A cell vs literal or computed error, both orders"
milestone: M2
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for which error SUM returns when a referenced cell holds an error and another argument is a different error.

## Context
probes/error-stored-busy-behaviors C11/C12: `SUM(A1,1/0)` and `SUM(1/0,A1)` with A1 a stored `#BUSY!` are both ERROR.TYPE 2, against syntax.md's "leftmost error wins for SUM". Probe SUM with a referenced cell holding `#N/A`, `#VALUE!` and a stored `#BUSY!`, against a literal (`#DIV/0!`) and a computed (`1/0`) error argument, in both orders, and with two referenced cells. Record the rule in syntax.md/arrays.md and file an engine task for any mismatch.

## Acceptance criteria
- [x] A probe case, oracle-filled (excel-graph), with the cells above.
- [x] The rule is recorded in docs/excel-semantics and the conflicting syntax.md sentence is fixed.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Done within T-1314: probes/sum-error-precedence (24 cells, excel-graph). A direct-argument error beats a referenced-cell error in SUM in either order; among references the leftmost wins. Recorded in syntax.md; engine fixed in T-1314.
