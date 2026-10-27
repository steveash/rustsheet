---
id: T-2764
title: "Calendar digits under LCID 0 keep the calendar with en-US names"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A calendar digit over LCID 0 applies its calendar with en-US names: `TEXT(45000,"[$-110000]mmmm|d")` and `"[$-120000]mmmm|d"` are "February|24", `"[$-E0000]mmmm|dddd"` "February|Wednesday".

## Context
Today `resolve_lcid` (formatter/locale_tag.rs) returns `Silent` for LCID 0 before reading the calendar digit, so these print the Gregorian date. Failing: probes/text-locale-lunar-edges A59, text-locale-lunar-tokens A57, A58 (T-1922). Probe whether other calendar digits (3, 5, 7, 9) over LCID 0 also apply before generalizing. See number-formats.md "Calendars of a locale tag", lunar bullet on LCID 0.

T-1924 (2026-10-26) keeps a calendar digit over any LCID without a locale, LCID 0 included (`without_locale`): A59, A57 and
A58 pass since then. Only the B2 cell (A79) remains.
T-2729 (2026-10-27) lets any calendar digit but 0 and 6 win over a `B2` before the tag, so A79 passes too: every
cell of this task passes; what is left is closing it.

## Acceptance criteria
- [ ] text-locale-lunar-edges A59 and text-locale-lunar-tokens A57, A58 pass.
- [ ] text-locale-lunar-tokens A79 passes: `B2` before a lunar tag gives the lunar date (`TEXT(45300,"B2[$-110409]yyyy|m")` is "2023|12"; T-2729 leaves the lunar digits out).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
