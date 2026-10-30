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
The lexer records the whitespace inside a structured reference's outer brackets (`TableSpacing` on the token and on
`StructuredReference`): the runs after `[` and before `]` as typed, and whether whitespace followed the comma.
`StructuredReference::text` prints them as Excel normalizes them: `T[ [Col] ]` prints `T[ Col ]`, the space before
the comma is dropped, and a run after the comma prints as one space. A stored `T[ Col ]` reads back the same, so
the export round-trips. The harness had set `{"entered": "=..."}` cells before the tables existed, so every typed
structured reference in the case was a parse error, and FORMULATEXT echoed the raw text. Typed formulas now go in
with the other formulas, after the tables (crates/rustsheet-eval/src/engine.rs).
Eval: probes 12 fixed (t2786 E4:E15 and F4, F7, F15; F5, F8, F9, F12:F14 pass now for the right reason), no
regressions. C2:C4 (this-row shapes) stay T-2800's. syntax.md records the two unprobed choices we made.
