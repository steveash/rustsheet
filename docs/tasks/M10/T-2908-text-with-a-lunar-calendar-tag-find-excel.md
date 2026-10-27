---
id: T-2908
title: "TEXT with a lunar calendar tag: find Excel's lunar month in O(log n) instead of walking every month since 1900 (probes/text-locale-lunar-months-1900 takes 7.3 s)"
milestone: M10
lane: perf
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A number format with a lunar calendar tag (`[$-110409]`, `[$-E0412]`, ...) costs about the same per value as a
Gregorian one, so probes/text-locale-lunar-months-1900 runs in well under 2 s, with every lunar date unchanged.

## Context
Gap triage 2026-10-27 (`make eval` on origin/main 6087413b): probes/text-locale-lunar-months-1900 is the slowest
vendored case of the run, 7.8 s in `make eval` and 7.3 s alone (`target/release/rustsheet-eval run --suite probes
--filter text-locale-lunar-months-1900`). Its 1,206 cells each format one year of days (about 730 TEXT calls a
cell, roughly 880,000 in all), so each call costs about 8 microseconds.

Suspected hot spot (confirm with a profile or a micro-bench before changing anything):
`formatter/lunar_calendar.rs` `lunar_ymd` walks the `CHINESE` month table from 1899 on every call, about 2,500
entries for a date near 2100, and for `LunarTable::Eastern` it binary-searches `EASTERN_CHANGES` once per month
walked. A table of cumulative month starts (built once, e.g. in a `std::sync::OnceLock`, or as a `const` array)
lets one binary search find the month, then year and month follow from a per-entry prefix count of `NEW_YEAR` flags;
`EARLY_YEARS` and the serial 0/60 handling stay as they are.
If the profile shows the cost is elsewhere (parsing the format code per call, `FILTER`/`SEQUENCE`), fix that hot
spot instead and say so in the Outcome, or hand the task up if it is not an S-sized change.

## Acceptance criteria
- [ ] A unit test checks the new lookup against the old walk (kept as the test's reference) for every serial
      from -1 to `LAST_SERIAL` + 1 under both `LunarTable::Chinese` and `LunarTable::Eastern`.
- [ ] The existing tests in formatter/test/test_lunar_calendar.rs and test_locale_tag.rs pass unchanged.
- [ ] probes/text-locale-lunar-months-1900 runs in under 2 s with the command above (before/after timings in the
      Outcome), and `make eval` shows no new known failure in any suite.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Lunar dates past 2100 (T-2759) and any other calendar's values.
