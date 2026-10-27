---
id: T-1613
title: "TEXT: probe and implement the B2 (Hijri) calendar prefix with date tokens"
milestone: M10
lane: functions-text
status: done
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
- [x] A probe case covers the formats above (excel-graph) and passes; number-formats.md records the rule (VERIFIED).
- [x] `make check` green; reviewers approve.

## Out of scope
- Other calendars and locale prefixes (`[$-x-...]`).

## Outcome
TEXT and cell formats with a bare `B2` prefix now print Hijri dates: the tabular Islamic calendar with leap year 15 (not
16) and epoch 622-07-18, Arabic month and weekday names, and Excel's name-state quirk (a weekday after a month name is
weekday min(month, 7); a month name after a weekday repeats it). Serial 0 shows serial 1's date and 60 is `#VALUE!`.
New probes probes/t1613-hijri-b2 (144 cells) and t1613-hijri-b2-weekday (78), excel-graph, both passing; number-formats.md
"The `B2` (Hijri) calendar". The day and month name tokens moved from format.rs into formatter/date_names.rs. Eval delta: probes
+2 cases, 222 cells, all passing; 0 regressions. Follow-up T-2729: `B2` before a locale tag and calendar digit 6.
