---
id: T-3284
title: "Probe U+3000 intersections after parenthesized operands"
milestone: M2
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for what Excel does with an intersection whose separator holds U+3000 when an
operand is parenthesized: `=((A1:A3)　A2:B2)`, `=(((A1:A3))　A2:B2)+1`, `=((A1)　(A2))`,
`=(A1:A3)　A2:B2`, stored and typed, with FORMULATEXT of each.

## Context
Excel evaluates `=A1:A3　A2:B2` to `#NAME?` and keeps the formula text (probes/intersection-whitespace
H7, T-0148; docs/excel-semantics/references.md marks the name's boundaries UNVERIFIED). T-3281
made RustSheet print the parenthesized shapes as written, also `#NAME?`; no probe covers them
(Excel-fidelity review of PR #2443). Use the `excel-probe` skill.

## Acceptance criteria
- [ ] A probe case with oracle values for the shapes above (value and FORMULATEXT); RustSheet
      matches, or each mismatch is filed as a task.
- [ ] references.md lists the shapes next to the H7 entry.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other separators (references.md "Intersection operator: separators").
