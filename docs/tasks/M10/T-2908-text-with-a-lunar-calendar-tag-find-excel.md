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
- Gap triage 2026-11-01 (`make eval` on origin/main fafb6c6f): since T-2759 (done) extended the lunar tables past
  2100, the nine probes/t2759-lunar-packed-2101 ... -9101 cases are the slowest of the run, 7.4-8.6 s each (8.3 s
  alone for t2759-lunar-packed-9101), about 73 of the probes suite's 84 s. Each formats a century of days under
  `[$-110409]`/`[$-E0412]` years 2101-9200, so a walk from 1899 is longer still there. The same lookup fixes them;
  confirm with the profile.
If the profile shows the cost is elsewhere (parsing the format code per call, `FILTER`/`SEQUENCE`), fix that hot
spot instead and say so in the Outcome, or hand the task up if it is not an S-sized change.

## Acceptance criteria
- [ ] A unit test checks the new lookup against the old walk (kept as the test's reference) for every serial
      from -1 to `LAST_SERIAL` + 1 under both `LunarTable::Chinese` and `LunarTable::Eastern`.
- [ ] The existing tests in formatter/test/test_lunar_calendar.rs and test_locale_tag.rs pass unchanged.
- [ ] probes/text-locale-lunar-months-1900 runs in under 2 s with the command above (before/after timings in the
      Outcome), and `make eval` shows no new known failure in any suite.
- [ ] Each of the nine probes/t2759-lunar-packed-*101 cases runs in under 2 s alone (`--filter t2759-lunar-packed`;
      before/after timings in the Outcome), with every cell's verdict unchanged.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any lunar or other calendar's values (T-2759 settled the dates past 2100; this task only makes them faster).
