---
id: T-2604
title: "Probe other unknown-table shapes on load (Nope[#All], Nope[], Sales[[#This Row],[Nope]], column ranges) (T-2322)"
milestone: M8
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's load of a formula naming an unknown table or column is probed for more shapes than `SUM(Nope[Qty])` and `SUM(Sales[Nope])`.

## Context
T-2322 rewrites every stored structured reference that fails with an unknown table or column to `#REF!` (`Parser::set_loaded`, parser/operands.rs), which extends past the two probed shapes: `Nope[#All]`, `Nope[]`, `Sales[[#This Row],[Nope]]`, `Sales[[Qty]:[Nope]]`. Use the `excel-probe` skill; record the finding in docs/excel-semantics/references.md.

## Acceptance criteria
- [ ] Probe cases with oracle values exist for those shapes, and the references.md entry states what Excel does; any divergence from the T-2322 rewrite becomes a follow-up fix task.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
