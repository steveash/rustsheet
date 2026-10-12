---
id: T-1450
title: "Probe which unquoted letters a number format prints as literals (follow-up of T-1434)"
milestone: M1
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
We know which unquoted letters Excel accepts as literals in a number format, and the formatter's lexer accepts exactly those.

## Context
Excel accepts `TEXT(1,"0:a:")` = "1:a:", `"0::a"` = "1:a" (probes/text-format-length-limit-g A42, A43) and the format `\\x` followed by a quoted string = "\xaaa..." (-i A100:A104: an escaped backslash, then an unquoted `x`). Our lexer (formatter/lexer.rs `next_token`) treats `a` and `x` as ILLEGAL, so these are `#VALUE!`. Which letters are accepted unquoted is unknown (some are tokens: d m y h s e b g n, A/P...).

## Acceptance criteria
- [ ] A probe of `TEXT(1,"0"&<letter>)` and `TEXT(1,"d"&<letter>)` for every ASCII letter (both cases) records which print as literals.
- [ ] The same probe covers colon runs we collapse by inference: `TEXT(1,"0\:\:")`, `TEXT(0.5,"h::mm")`.
- [ ] The finding is in docs/excel-semantics/number-formats.md; the lexer change is filed as a task if needed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Non-ASCII letters.
