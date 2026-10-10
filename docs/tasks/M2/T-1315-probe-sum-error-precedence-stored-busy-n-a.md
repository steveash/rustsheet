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
- [ ] A probe case, oracle-filled (excel-graph), with the cells above.
- [ ] The rule is recorded in docs/excel-semantics and the conflicting syntax.md sentence is fixed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `probes/sum-error-precedence` (39 cells, excel-graph). In SUM and MAX an error in a direct argument beats
an error held in a referenced cell in either order; among cells the leftmost wins; the operators stay leftmost.
`#BUSY!` has no special rank. Recorded in syntax.md; engine task T-1317. Eval delta: probes known failures
931 -> 949 (18 cells, owned by T-1317). The conflicting syntax.md sentence turned out to rest on direct-versus-direct pairs, so it stays.
