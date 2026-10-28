---
id: T-2388
title: "TEXT of a negative number with a lone B1 calendar prefix is a minus sign (ours #VALUE!)"
milestone: M5
lane: functions-text
status: done
priority: P0
size: S
tier: sonnet
depends_on: []
---

## Goal
`TEXT(-3,"b1")` is "-" as in Excel. Ours is `#VALUE!`.

## Context
Found by T-2377's probe: probes/text-format-digits-date-token C25 (excel-graph, 2026-10-22) is "-". The cell is a known
failure in eval/baseline/probes.txt. Notes: docs/excel-semantics/number-formats.md "Bare digits beside a date token", last
sentence. `Token::Calendar('1')` sets `is_date` in `Parser::parse_part` (formatter/parser.rs), so a negative value takes the
date path and is refused. In Excel, a section whose only date-like token is the calendar prefix seems to act like an empty
number section (`TEXT(-0.0001,"")` = "-", "Empty format" in number-formats.md). Probe "b1", "B1", "b1 x" and "B1@" on -3, 0 and 3,
and `"[$-40C]B1"` on -3, before choosing the rule.

## Acceptance criteria
- [ ] Probe cells (excel-graph) recorded in number-formats.md; probes/text-format-digits-date-token C25 passes and leaves
      eval/baseline/probes.txt (`make eval-baseline`), with no new known failure.
- [ ] `make check` green; reviewers approve.

## Out of scope
- B2 (Hijri), T-1613.

## Outcome
A date section holding only a calendar prefix and literals now prints the minus and the literals for a negative value
(`TEXT(-3,"b1")` is "-", `"b1 x"` "- x"); change in `format_number` (formatter/format.rs). New probe
probes/t2388-b1-negative (28 cells, excel-graph, all pass); probes/text-format-digits-date-token C25 left the baseline
(known failures -1, no regressions). Rule recorded in docs/excel-semantics/number-formats.md.
