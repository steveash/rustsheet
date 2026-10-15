---
id: T-1613
title: "TEXT: probe and implement the B2 (Hijri) calendar prefix with date tokens"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
TEXT and cell number formats with the `B2` (Hijri calendar) prefix show the same Hijri date components as Excel, instead of `#VALUE!`.

## Context
T-1262 made `B1`/`B2` calendar prefixes that print nothing (`TEXT(100,"B2")` = "", probes/text-format-bare-literals A48, A50),
but a `B2` section with any printing date or time token is `#VALUE!` in the engine (`Parser::parse_part`, formatter/parser.rs,
the `hijri` flag) because the Hijri calendar is not implemented. Semantics: docs/excel-semantics/number-formats.md
"Buddhist-era year, era and calendar tokens". Probe first (excel-probe skill): `TEXT(45000,"B2yyyy-mm-dd")`, `"B2d"`,
`"B2mmmm"`, `"B2dddd"`, `"B2e"`, a time-only `"B2h:mm"`, serials 1, 60 and 2958465, and `B2` after other tokens (`"dB2"`).

## Acceptance criteria
- [ ] A probe case covers the formats above (excel-graph) and passes; number-formats.md records the rule (VERIFIED).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other calendars and locale prefixes (`[$-x-...]`).
