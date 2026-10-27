---
id: T-2801
title: "Print whitespace inside structured references as Excel normalizes it"
milestone: M2
lane: engine
status: in-progress
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
- [ ] F4:F15 of probes/t2786-whitespace-structured-refs-runs pass (formula text and values).
- [ ] `make check` green; reviewers approve.

## Out of scope
- This-row shapes (T-2800).
