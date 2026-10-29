---
id: T-2770
title: "Probe IMSUB/IMDIV with an omitted LAMBDA parameter and a LET-bound empty value"
milestone: M9
lane: functions-engineering
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
IMSUB and IMDIV match Excel when an argument is an omitted LAMBDA parameter or a LET-bound empty value, or the mismatch is recorded.

## Context
T-2543 made `complex_pair_of` (complex/mod.rs) raise `#N/A` only for an argument node left out of the formula (`Node::EmptyArgKind`); an empty value from any other source is 0. Unprobed: `LAMBDA(a,b,IMSUB(a,b))("1",)` and `LET(e,IFERROR(Q1,"x"),IMSUB("1",e))` (found by T-2543's fidelity review). Probe them (excel-probe skill), match, and update engineering.md "T-1625 probes".

## Acceptance criteria
- [x] A probe case with oracle values for the shapes above; the engine matches them or each mismatch is recorded.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed 12 cells (probes/eng-imsub-imdiv-lambda-let-empty, excel-graph). Excel gives `#N/A` for an omitted LAMBDA parameter and for `LET(e,,...)` in IMSUB and IMDIV; ours returned a number or `#NUM!` in 5 cells. `complex_pair_of` now also treats a variable bound to an omitted value as an omitted argument. Rule recorded in engineering.md "T-1625 probes". Eval delta: +12 probe cells passing (5 fixed), 0 regressions.
