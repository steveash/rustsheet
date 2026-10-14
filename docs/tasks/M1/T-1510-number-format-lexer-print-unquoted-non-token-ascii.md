---
id: T-1510
title: "Number-format lexer: print unquoted non-token ASCII letters as literals (T-1450 evidence)"
milestone: M1
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The number-format lexer prints the unquoted ASCII letters Excel accepts as literals, and rejects the rest, as Excel does.

## Context
Evidence and the full letter table: docs/excel-semantics/number-formats.md "Unquoted ASCII letters in a number format" (probes/text-format-unquoted-letters, T-1450). `formatter/lexer.rs` `next_token` treats every non-token letter as illegal. Also covers the escaped `\:\:` and `h::mm` colon runs, and `0:a:` / `0::a` (probes/text-format-length-limit-g A42, A43 and -i A100:A104).

## Acceptance criteria
- [ ] Every cell of probes/text-format-unquoted-letters passes; known failures only shrink.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
