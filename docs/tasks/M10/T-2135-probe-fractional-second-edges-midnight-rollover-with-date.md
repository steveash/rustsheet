---
id: T-2135
title: "Probe fractional-second edges: midnight rollover with date tokens, 9999-12-31 cutoff in 1904, m after [m], s.0000"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for the fractional-second choices T-0934 made without evidence, and the engine matching it.

## Context
T-0934 (`FractionalSeconds`, formatter/elapsed.rs; `TextToken::SecondFraction`, formatter/parser.rs) rounds the
value to the shown digit of a second and cuts the clock fields from that, but date tokens still read the unrounded
serial, so `TEXT(45000.9999999,"yyyy-mm-dd hh:mm:ss.0")` shows the old day with "00:00:00.0". Also unprobed
(docs/excel-semantics/number-formats.md "Elapsed-time tokens"): where the "fraction past 9999-12-31 is #VALUE!"
cutoff sits (the engine uses serial 2958466 in both date systems; in 1904 9999-12-31 is 2957003), `m`/`mm` after
`[m]` (the engine keeps a clock minute), and `s.0000` (the engine gives #VALUE!). Related: T-1731, T-1732.

## Acceptance criteria
- [ ] A probe case (excel-probe skill), 1900 and 1904, with date + `ss.0`/`.000` tokens on values within half a
  unit of midnight, a minute and an hour; `h:mm:ss.0` at 2958465.99999 and 2958466 (and the 1904 equivalents);
  `[m] m`, `[mm]:mm`; `s.0000`, `[s].0000`. Oracle-filled.
- [ ] The engine matches (or failures are filed as tasks) and number-formats.md is updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Whole-second rounding without a fraction token (T-1732).
