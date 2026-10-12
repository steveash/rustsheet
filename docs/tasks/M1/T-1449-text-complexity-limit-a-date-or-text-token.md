---
id: T-1449
title: "TEXT complexity limit: a date or text token before a quoted string at its limit, and two conditions in a section (follow-up of T-1434)"
milestone: M1
lane: functions-text
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The TEXT complexity model explains the cells T-1434's byte-budget fit still misses, and the engine matches them.

## Context
T-1434 fitted the limit as a 250 budget (quoted character 2, unquoted 3, placeholder run 5, other token 2; docs/excel-semantics/number-formats.md "TEXT format code complexity limit", formatter/complexity.rs). It misses:
- a date token followed by a quoted string at the string's own limit: `TEXT(1,"d"&""""&REPT("a",125)&"""")` works (probes/text-format-length-limit-f A7; also `m`, `h`, `dd` + 125 and `d ` + 123, -i A44, A53, A78, A98), costing 252 in the model. The same token followed by unquoted text costs 2 (`"d"&REPT(" ",83)` fails), so the discount is specific to quoted text; maybe a per-run limit of 250 plus a total over a different measure.
- two conditions in a section: `"[>5][<9]0"&REPT(" ",73)` to 77 are `#VALUE!` (-g A37:A41) while one condition costs 6 to 8. Probe `TEXT(1,"[>5][<9]0")` short: two conditions may simply be invalid.

## Acceptance criteria
- [ ] Probes bisect a date token + quoted string + k spaces for several lengths, and two conditions on a short code.
- [ ] `complexity.rs` matches the probed cells (or the misses are recorded with the evidence).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Conditional section selection (T-0936).
