---
id: T-1031
title: "Probe intersections of unions with nested or right-side negated operands"
milestone: M1
lane: harness
status: done
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
- [x] Each form is recorded as loaded (with its value) or refused in references.md; the engine matches or the gap is a filed task.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed with excel-graph: `A1 (-A1,B1)`, `((-A1,B1),C1) A1`, `(-(A1,B2),C1) A1`, `(--A1,B1) A1` and `(A1,-B1) A1` are refused (`FileCorruptTryRepair`); `(---A1,B1)` and `(----A1,B1)` load as `#VALUE!`. The engine now refuses a negated-union operand on either side of an intersection at any union depth, and accepts any number of minuses. Recorded in references.md. Eval delta: two new probe cases with oracle values.
