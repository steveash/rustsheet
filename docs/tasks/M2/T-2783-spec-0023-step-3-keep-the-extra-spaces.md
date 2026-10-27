---
id: T-2783
title: "SPEC-0023 step 3: keep the extra spaces of an intersection operator in formula text"
milestone: M2
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: [T-2781]
---

## Goal
An intersection written with more than one space prints them all, as Excel does: `=B1:B2  B2`.

## Context
SPEC-0023 step 3. The first space is the operator; Excel keeps the rest. The parser already reads the
separator run (`Lexer::separator`, reference_operators.rs). Evidence: probes/t2776-formula-text-whitespace-shapes
C3, C5 (typed) and E9 (stored).

## Acceptance criteria
- [x] t2776-formula-text-whitespace-shapes C3, C5, E9 pass; 0 regressions.
- [x] `make check` green; reviewers approve.

## Out of scope
- Line feeds as an intersection separator beyond what references.md records.

## Outcome
`parse_intersection` keeps the separator's spaces after the first as the right operand's leading run
(`Run::of`), and `intersection_operand` peels `Spaced` so the operand is not parenthesised.
Eval delta: probes 2961 -> 2958 known failures (t2776-formula-text-whitespace-shapes C3, C5, E9), 0 regressions.
