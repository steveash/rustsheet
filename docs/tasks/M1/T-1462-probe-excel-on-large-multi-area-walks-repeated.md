---
id: T-1462
title: "Probe Excel on large multi-area walks repeated inside MAP and BYROW"
milestone: M1
lane: research
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence for what Excel returns when a formula repeats a large multi-area walk inside MAP or BYROW, so we know whether the per-formula `MAX_AREA_CELLS` budget (T-1162) turns realistic formulas into `#VALUE!` where Excel gives a number.

## Context
T-1162 Excel-fidelity review: since T-1162 the cell budget (2^24 cells) is shared by the whole formula, so `=BYROW(A1:A1000,LAMBDA(r,SUM((B:B,C:C))))` over 10,000 used rows (2*10^7 cells) is `#VALUE!` in the engine (DEVIATION, references.md "Union area cap"), while `SUM(B:B)` in the same place is not counted. Probe with the `excel-probe` skill: the BYROW case, a MAP over a union, and the same with a single range, and record the values and timing notes in references.md. If Excel computes them, propose (in the Outcome, as a follow-up task) a bound that grows with the data rather than with repetition, e.g. counting each distinct union once per formula.

## Acceptance criteria
- [x] Probe case with oracle values for the formulas above, recorded in references.md "Union area cap".
- [x] Follow-up task filed if the budget is shown to be stricter than needed.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing the budget in this task.

## Outcome
- Probe probes/union-walk-repeated-in-lambda (15 cells, excel-graph, 2026-10-13): Excel computes
  every BYROW/MAP formula that repeats a `(B:B,C:C)` walk past the engine's 2^24-cell budget
  (2*10^7 and 2*10^8 cells), and even a single `SUM` of nine whole-column unions over a sheet whose
  data reaches row 1,048,576. The engine is `#VALUE!` on 9 of the 15 cells; single ranges and
  MAP over a union (`#VALUE!` in both) agree. Recorded in references.md "Union area cap".
- The budget is stricter than needed: follow-up T-1497 (bound the walks by the data they visit,
  e.g. a sparse walk charged by populated cells; caching each distinct union once is the weaker
  option, since it does not fix the nine-argument case).
- Eval delta: probes gains 15 checked cells, 9 known failures (the new case); no other change.
