---
id: T-1434
title: "TEXT complexity limit: probe quoted-text cost (91-149 chars), the three unexplained cells, and error precedence (follow-up of T-1393)"
milestone: M1
lane: functions-text
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The fitted TEXT format complexity limit (T-1393) is pinned where it is only bounded today, and the three cells it does not explain are understood and matched.

## Context
docs/excel-semantics/number-formats.md "TEXT format code complexity limit"; code in formatter/complexity.rs (`exceeds_complexity_limit`) and the TEXT call in functions/text/common.rs.
- The cost of a quoted character is only bounded (1.08–1.8; the engine uses n + ceil(n/2)). `d"<n a's>"` works at n=90 and fails at n=150.
- Known failures:
  - probes/text-format-length-limit-c A26 `TEXT(1,"d;d;d;d")` is `#VALUE!`; probably a date token in the text section.
  - probes/text-format-length-limit-d A15 `TEXT(1,"d"&REPT(":",84))` = "1:".
  - text-format-length-limit-d A31 `TEXT(1,"[>5]0"&REPT(" ",79))` = "1".
- Unprobed: whether an error value or a too-complex format wins (`TEXT(NA(),REPT("dm",17))`).

## Acceptance criteria
- [ ] A probe bisects `"d"&""""&REPT("a",n)&""""` for n in 91..149 and the quoted-only boundary; `complexity.rs` uses the measured cost.
- [ ] The three cells above are probed further (a date token in each of the four sections; colon runs of 2..84 after `d` and `0`; an unmatched condition with and without padding), and they are matched or recorded.
- [ ] Error-vs-complexity precedence is probed for scalar and array values and matched.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
