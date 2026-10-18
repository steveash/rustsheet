---
id: T-1699
title: "Resolve #This Row off-table #VALUE! at evaluation, not parse time"
milestone: M8
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`#This Row` off the table's data rows is decided when the table is resolved at evaluation, so a formula stays right after the table grows, shrinks or has rows inserted, and serialises back to the structured reference.

## Context
T-1675 returns `Node::ErrorKind(VALUE)` at parse time in `structured_reference_operand` (operands.rs), as `#Totals` without a totals row already returns `#REF!`; the table is not re-resolved on later edits. Excel evidence: gen-m8b this-row e1bf5aa0ffb4ed87.

## Acceptance criteria
- [ ] Unit test: grow the table so a former off-table row becomes a data row and the formula reads it; export keeps the structured reference.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
