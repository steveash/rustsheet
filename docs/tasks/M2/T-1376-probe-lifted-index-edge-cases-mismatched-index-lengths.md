---
id: T-1376
title: "Probe lifted INDEX edge cases: mismatched index lengths, negative and boolean elements, scalar error next to an array, empty cells"
milestone: M2
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for the lifted-INDEX cases T-1260 implemented by analogy, and the engine matching it.

## Context
T-1260 lifts INDEX over array and range indices (`index_lifted`, functions/lookup_and_reference/index.rs;
lookup.md "INDEX lifts over array and range indices"). probes/index-array-indices backs the main rules. These cases
follow the arithmetic-lift rules but have no oracle cell (excel-fidelity review of T-1260):
- mismatched index lengths, e.g. `INDEX(A1:B5,{1,2,3},{1,2})`; the engine gives `#N/A` past the shorter;
- a negative element (`{1,-1}`; the engine gives `#VALUE!`) and boolean elements (`{TRUE,2}`);
- a scalar error index next to an array index (`INDEX(A1:A5,{1,3},#N/A)`; the engine broadcasts the error);
- an empty selected cell (`INDEX(A1:A5,{1,2})` with A2 blank): does it spill as 0 or as blank?

## Acceptance criteria
- [x] A probe case (excel-graph) covers the cases above; lookup.md's Unprobed line is replaced by the findings.
- [x] Any engine mismatch is fixed here if small, otherwise filed as its own task.
- [x] `make check` green; reviewers approve.

## Out of scope
- The legacy, implicitly intersected form (T-0294).

## Outcome
Added probes/index-lifted-edge-cases (28 questions, excel-graph). Excel follows the arithmetic broadcast for mismatched lengths,
puts `#VALUE!` in negative slots, takes booleans as 0/1, spreads a scalar error index over the result and spills an empty
cell as 0 while keeping its blankness. The engine matches all of it except COUNTBLANK (Excel `#VALUE!` per element) and
COUNTA (counts the blank element) over a lifted INDEX, filed as T-1437 (with T-1395). lookup.md's Unprobed line is replaced
by the findings. Eval delta: 4 known-failure cells recorded in the new case, 0 regressions.
