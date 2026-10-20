---
id: T-1699
title: "Resolve #This Row off-table #VALUE! at evaluation, not parse time"
milestone: M8
lane: engine
status: blocked
priority: P1
size: S
tier: opus
campaign: visible
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

## Visible-parity stage
Decision 0083: wrong values here are visible to anyone using defined names or tables (gen-m8b `[#This Row]` off-table cells), so this task is in the campaign. Its acceptance criteria are already the visible-parity stage.

## Blocked because
Needs the evaluation model of SPEC-0016 (T-2194): the parser lowers every structured reference to a
range at parse time (`structured_reference_operand`, parser/operands.rs), `to_rc_format` stores the
lowered range, and shared formulas parse at (1, 1), so neither the off-table `#VALUE!` nor the grown
table can be decided later without a kept structured-reference node. T-2194 (the spec) is blocked at
the review-round limit on its pushed branch `agent/engine/T-2194-spec-structured-refs`, which also
re-scopes this task as rollout step 2 with `depends_on: [T-2195]`. Unblock when T-2195 is done.
