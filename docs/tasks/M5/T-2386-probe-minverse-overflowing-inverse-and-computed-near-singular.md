---
id: T-2386
title: "Probe MINVERSE overflowing inverse and computed near-singular pivots"
milestone: M5
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel oracle values pin MINVERSE where the inverse overflows and where a near-singular pivot comes from cell values rather than 15-digit-cut literals, and arrays.md "MINVERSE: singularity rule" records them.

## Context
T-2362 made `fn_minverse` (functions/math_and_trigonometry/matrix.rs) treat only an exactly-zero LU pivot as singular and
return `#NUM!` for the whole result when any inverse element is not finite. The second rule is unprobed, and the
explanation of `{1,1;1,1.0000000000000002}` = `#NUM!` (the literal is cut to 1) is inferred from numbers.md, not probed
directly (reviewer-excel-fidelity, T-2362 review). Use the excel-probe skill: a probe case with e.g. `MINVERSE({1E-200,1;0,1E-200})`
(substitution overflows), `MINVERSE(A1)` with A1 = `=1E-300*1E-10` (subnormal), `MINVERSE(B1:C2)` with cells 1, 1, 1, `=1+2^-52`,
and the 3x3 `{1,2,3;4,5,6;7,8,9}` from cells.

## Acceptance criteria
- [ ] The probe case is oracle-filled (excel-graph) and its findings are in docs/excel-semantics/arrays.md; any engine mismatch is filed as a functions-math task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (file them as a follow-up).
