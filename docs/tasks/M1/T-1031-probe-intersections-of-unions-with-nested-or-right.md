---
id: T-1031
title: "Probe intersections of unions with nested or right-side negated operands"
milestone: M1
lane: harness
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Probes record whether Excel loads intersections of a union with a negated operand beyond the two forms T-0998 found refused.

## Context
Follow-up of T-1024, which refuses only `(-A1,B1) A1` and `(-A1,B1) B1`. Unprobed (engine accepts): `A1 (-A1,B1)`, `((-A1,B1),C1) A1`, and `(-(A1,B2),C1) A1` (engine refuses), `(--A1,B1) A1` (engine refuses), `(A1,-B1) A1` (engine refuses), and `(---A1,B1)` (engine refuses). Excel refusals load-fail (`FileCorruptTryRepair`), so use one scratch workbook per formula like T-0187. See docs/excel-semantics/references.md "Union: parsing and evaluation".

## Acceptance criteria
- [ ] Each form is recorded as loaded (with its value) or refused in references.md; the engine matches or the gap is a filed task.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
