---
id: T-1924
title: "Underscore locale tags such as [$-fr_FR] select a calendar"
milestone: M10
lane: functions-text
status: done
priority: P3
size: S
tier: opus
depends_on: [T-1922, T-1923]
---

## Goal
A locale tag with an underscore (`[$-fr_FR]`, `[$-de_DE]`, `[$-_]`) resolves as Excel does: `TEXT(45000,"[$-fr_FR]mmmm")` is "December".

## Context
Excel reads these tags as something other than a BCP-47 tag, and they select calendars. `[$-fr_FR]` gives the Saka date with English Gregorian month names ("December|1944-12-24"). `[$-de_DE]`, `[$-en_US]` and `[$-ja_JP]` give the lunar date with English names ("February|2023-02-24"). `[$-x_y]` and `[$-fr_]` give plain en-US. `[$-_]` gives Russian names and `[$-a_b_c]` Arabic. The parse rule is unknown. Probe enough tags to find it (perhaps a hash, or a hex reading of the characters), then implement it. Evidence: probes/text-locale-calendars A135-A145, probes/text-locale-name-tables A596. Depends on the lunar and Saka calendars (T-1922, T-1923) for the dates.

## Acceptance criteria
- [x] probes/text-locale-name-tables A596 and text-locale-calendars A135-A145 pass, and the rule is recorded in number-formats.md.
      Amended 2026-10-26: the Saka cells (A135, A143-A145, A596) need the Saka calendar and moved to T-2779; A140 (`[$-_]`,
      Macedonian) needs a locale table and moved to T-1875.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
The parse rule: a tag that is neither hex nor BCP-47 is read as hex anyway, with every character a digit. A letter counts from
10 in either case, any other character below `a` counts from `0` (`_` 47), and one above `z` counts from `a`. So `[$-fr_FR]` is
0x10E00B, the Saka calendar over an LCID with no locale. A calendar digit over an LCID without a locale keeps its date with en-US
names (Thai and Arabic calendars keep their own), and `A/P` stays as written. Probed in probes/t1924-locale-tag-extended-hex (60
cells, excel-graph). Implemented in formatter/locale_tag.rs (`extended_hex`, `resolve_extended`, `without_locale`;
`Localized::day_half` is now an `Option`). Recorded in number-formats.md.
Eval: probes fixed 7 (text-locale-calendars A136-A138 and A142; T-2764's text-locale-lunar-edges A59 and
text-locale-lunar-tokens A57, A58), new-coverage failures 20 (the new
probe's cells owned by T-1875, T-1925 and T-2779), 0 regressions. Handed on: the Saka cells to T-2779, the missing tables and
the open 0xF409 fallback to T-1875, Myanmar numerals to T-1925, and a check note to T-2764.
