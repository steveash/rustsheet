---
id: T-1434
title: "TEXT complexity limit: probe quoted-text cost (91-149 chars), the three unexplained cells, and error precedence (follow-up of T-1393)"
milestone: M1
lane: functions-text
status: in-progress
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

## Progress (in flight, 2026-10-12)
- Probe rounds f and g (excel-graph sidecars committed) answer the error-precedence question and explain the three
  cells: the fourth-section rule, colon-run collapse, and the unmatched condition printing General (a condition
  costs 5 or 6). See docs/excel-semantics/number-formats.md "TEXT format code complexity limit".
- The quoted-text cost is not one additive weight: `d` seems free next to quoted text (`d`+125 quoted works, like
  quoted alone) but `d`+50 quoted+50 spaces fails. Next: probe whether a long literal run has its own limit
  (125 quoted characters, or a byte budget where unquoted characters count double). Bisect `d`+n quoted+k spaces
  for several n, and quoted text after `0` and after `[Red]`, before changing `complexity.rs`.
- Round h (mixed quoted/unquoted runs, quoted after `0` and `[Red]`) fits about 4/3 per quoted character on its own,
  but `0`+122 quoted works / 123 fails while `d`+125 works. So the quoted cost depends on what precedes it (a
  placeholder vs a date token vs a color). Next hypothesis to test: the code is rebuilt into a normalized string
  (quoted text re-escaped, a placeholder followed by a literal getting a separator), with a byte budget.
  Round i did that: the longest quoted string is 125 after nothing, a date token or `@`; 124 after `%` or a color;
  122 after a placeholder or an escaped character; 121 after `0.0` or `0 `. A plausible model is floor(4n/3) per
  quoted run plus the usual costs, except that a date or text section, or a long quoted run, is cheaper. Pin it with
  `d`/`0` + quoted + spaces at several lengths before coding.
- A brute-force fit over 42 measured boundaries found no integer cost model that fits them all. The space searched:
  `d`, `0`, color, `%` and `@` costs; quoted cost n + o·ceil(n/k) + s, adjacent quoted strings merged or not;
  limit 160..174. The best models miss 4 boundaries: limit 167, `0` 3, color 1, `%` 1, quoted n + ceil(n/3) or
  n + 2·ceil(n/6). They miss `d`+83 spaces, two 62-char strings after `d`, 50 spaces + 50 quoted, and `0 ` + 121 quoted.
  The rule is probably not one additive budget. Consider modeling Excel's internal format buffer instead (e.g. a
  per-section byte length after normalization).
- Not done yet: the code changes (fourth-section `#VALUE!`, colon collapse, condition cost, error precedence over
  arrays if it differs), and `make eval-baseline` for cases f to i.
