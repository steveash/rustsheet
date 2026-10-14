---
id: T-1450
title: "Probe which unquoted letters a number format prints as literals (follow-up of T-1434)"
milestone: M1
lane: research
status: done
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
- [x] A probe of `TEXT(1,"0"&<letter>)` and `TEXT(1,"d"&<letter>)` for every ASCII letter (both cases) records which print as literals.
- [x] The same probe covers colon runs we collapse by inference: `TEXT(1,"0\:\:")`, `TEXT(0.5,"h::mm")`.
- [x] The finding is in docs/excel-semantics/number-formats.md; the lexer change is filed as a task if needed.
- [x] `make check` green; reviewers approve.

## Out of scope
- Non-ASCII letters.

## Outcome
Probed every ASCII letter after `0` and after `d`, plus the colon runs: probes/text-format-unquoted-letters (111 cells, excel-graph). Result and letter table are in docs/excel-semantics/number-formats.md. Eval delta: probes known failures 5142 -> 5217 (new coverage: 75 cells the lexer rejects; 0 regressions). Lexer change filed as T-1510.
