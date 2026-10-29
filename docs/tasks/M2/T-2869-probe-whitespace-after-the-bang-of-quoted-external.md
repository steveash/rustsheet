---
id: T-2869
title: "Probe whitespace after the bang of quoted, external and R1C1 sheet prefixes"
milestone: M2
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for whitespace after the `!` of quoted (`='My S'! B1`), external (`[1]S! A1`) and R1C1 sheet prefixes, with a line feed after the `!`; adjust the lexer if Excel differs.

## Context
T-2785 skips whitespace after every sheet prefix's `!` (lexer `consume_range`) but only the unquoted `=S! B1` form has oracle evidence (probes/t2776-ws-after-bang). Reviewer finding, excel-fidelity.

## Acceptance criteria
- [x] Probe cases with oracle values for each form above; engine matches; syntax.md entry updated.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Three probe cases (quoted space, quoted line feed, unquoted line feed after the bang) pass bit-exact with no engine change; the lexer already skips whitespace after every prefix. External `[1]S! A1` is accepted by Excel (value environment-specific, no case kept); `S! R1C2` is refused. Recorded in syntax.md. Eval: 3 new passing probe cells groups (6 cells), no regressions.
