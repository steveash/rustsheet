---
id: T-1434
title: "TEXT complexity limit: probe quoted-text cost (91-149 chars), the three unexplained cells, and error precedence (follow-up of T-1393)"
milestone: M1
lane: functions-text
status: done
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
- [x] A probe bisects `"d"&""""&REPT("a",n)&""""` for n in 91..149 and the quoted-only boundary; `complexity.rs` uses the measured cost.
- [x] The three cells above are probed further (a date token in each of the four sections; colon runs of 2..84 after `d` and `0`; an unmatched condition with and without padding), and they are matched or recorded.
- [x] Error-vs-complexity precedence is probed for scalar and array values and matched.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- Probe rounds f to i (excel-graph) bisected quoted text after every kind of token. An integer fit over all 481 probed cells
  (MILP over per-token weights) gives a byte-like budget: at most 250, quoted character 2, unquoted or escaped character 3
  (`$` 1), placeholder run 5 plus 2 per `.`, other tokens 2, color 1, condition 7, `;` 0. It misclassifies 11 cells: a
  date or text token directly before a quoted string at its limit (6) and two conditions in a section (5); T-1449.
- `formatter/complexity.rs` uses that model, plus the fourth-section rule (a number or date token there is `#VALUE!`).
  Colon runs print as one (`formatter/parser.rs`); a single `@` section prints a number in General (`format_number`).
- Error precedence was already right for scalars and value arrays; an array of format codes is T-1451. Letters Excel
  accepts unquoted (`0:a:`) are T-1450. The unmatched condition with padding stays with T-0936.
- Eval: probes known failures +20 net with 4 new cases (fixed 2 old cells: `"d;d;d;d"`, the 84-colon run; 22 known
  failures in the new cases f to i out of 279 cells). Probe milestone M0-M2 known failures 221 → 101 (vs the unbaselined
  cases on the branch).
