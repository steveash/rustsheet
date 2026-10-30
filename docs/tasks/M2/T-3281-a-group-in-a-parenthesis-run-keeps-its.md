---
id: T-3281
title: "A group in a parenthesis run keeps its text in a U+3000 intersection name"
milestone: M2
lane: engine
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
An intersection whose separator holds U+3000 (read back as an undefined name, `#NAME?`) keeps its
whole written text when its left operand is a group inside a parenthesis run:
`=((A1:A3)　A2:B2)` prints as written, not `=(A2:B2)`.

## Context
Excel evaluates `=A1:A3　A2:B2` to `#NAME?` and keeps the formula text
(probes/intersection-whitespace H7, T-0148); RustSheet builds a synthetic name from the source
text (`Parser::parse_intersection`, `source_since(start)`). Since T-3191 a run of parentheses
hands each inner group to its enclosing group's content as a pending operand that has already
been read, so `start` is taken after it and the name loses the group's text. Found while working on
T-3197 (closed duplicate PR #2440). FORMULATEXT and export then print the wrong formula.

## Acceptance criteria
- [ ] `=((A1:A3)　A2:B2)` and `=(((A1:A3))　A2:B2)+1` are `#NAME?` and print as written (unit
      test); `=(A1:A3　A2:B2)` still does.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Which other separators Excel accepts (references.md).
