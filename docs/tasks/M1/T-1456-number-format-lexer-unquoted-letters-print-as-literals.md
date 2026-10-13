---
id: T-1456
title: "Number-format lexer: unquoted letters print as literals (follow-up of T-1450)"
milestone: M1
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-1262]
---

## Goal
The number-format lexer accepts the letters Excel prints as literals without quotes, so `TEXT(1,"0a")` is "1a".

## Context
See docs/excel-semantics/number-formats.md "Unquoted ASCII letters in a number format". Evidence: probes/format-unquoted-letters (all 108 cells), plus probes/text-format-length-limit-g A42, A43 and -i A100:A104. Code: formatter/lexer.rs `next_token`. Watch the complexity cost of these literals (3 per unquoted character).

## Acceptance criteria
- [ ] Every cell of probes/format-unquoted-letters passes, including the `#VALUE!` cases for token letters.
- [ ] The newly passing cells of text-format-length-limit-g and -i are in the baseline diff.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Digits 1-9, the integer-before-point rule and the empty format: those stay with T-1262 (same lexer code). T-1456 is the ASCII-letter set only, and lands first or together with T-1262; whoever lands second merges, not re-implements.
