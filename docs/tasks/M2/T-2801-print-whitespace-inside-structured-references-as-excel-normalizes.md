---
id: T-2801
title: "Print whitespace inside structured references as Excel normalizes it"
milestone: M2
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Whitespace inside a structured reference prints as Excel normalizes it.

## Context
T-2786 shapes: `T[ [Col] ]` prints `T[ Col ]`; `T[ [#Data] , [Col] ]` prints `T[ [#Data], [Col] ]`; `T[[#Data]  ,  [Col]]` prints `T[[#Data], [Col]]`; `T[ [Col]:[Other] ]` is kept. Refused typed: `T[ Col]`, `T[Col ]`, `T [Col]`, `T[[Col] : [Other]]`; Excel itself saves `T[ Col ]` (loading a hand-written stored one is unprobed, T-2800). See syntax.md "Whitespace beside a missing argument is kept; a structured reference..."; probes/t2786-whitespace-structured-refs-runs F4:F15.

## Acceptance criteria
- [x] F4:F15 of probes/t2786-whitespace-structured-refs-runs pass (formula text and values).
- [x] `make check` green; reviewers approve.

## Out of scope
- This-row shapes (T-2800).

## Outcome
- The lexer keeps a structured reference's whitespace in `TableSpacing` (token and
  `StructuredReference`): the runs after the outer `[` and before its `]`, and whether a run follows
  the comma before the column. `StructuredReference::text` prints them as Excel does: `T[ [Col] ]`
  prints `T[ Col ]`, `T[ [#Data] , [Col] ]` prints `T[ [#Data], [Col] ]`, `T[[#Data]  ,  [Col]]` prints
  `T[[#Data], [Col]]`, `T[ [Col]:[Other] ]` stays. Outer-padded references used to be parse errors.
  The lexer reads the saved `T[ Col ]` back, so the export round-trips (syntax.md "Ours").
- Harness: an `{"entered":"=..."}` formula is now set with the formulas, after the tables, so a typed
  structured reference finds its table (every E cell of this case was `#ERROR!` before).
- Eval: probes known failures 2983 -> 2971 (t2786 E4, E5, E7:E9, E12:E15, F4, F7, F15 fixed; F8
  passes for the right reason now), no regressions. C2:C4 stay with T-2800, H/I with T-2944.
