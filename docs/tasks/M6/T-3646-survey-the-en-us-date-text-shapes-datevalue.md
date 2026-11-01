---
id: T-3646
title: "Survey the en-US date text shapes DATEVALUE accepts and match them through the shared date-text reader"
milestone: M6
lane: functions-date
status: in-progress
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
DATEVALUE, VALUE and date arguments accept exactly the en-US date texts Excel accepts and refuse the rest with `#VALUE!`, across a broad survey of shapes, with the grammar in dates.md updated to match.

## Context
Lead: the 2026-11-01 holdout themes note (docs/research/holdout-themes-2026-11-01.md). It names a class of input that fails, never a case, and it is not Excel evidence: every behaviour change here needs fresh oracle cells from the excel-probe skill, and the probe cases must not copy the note's wording. If the probes show the engine already matches Excel across the class, record that in the Outcome and close the task.

The holdout themes note reports DATEVALUE giving `#VALUE!` for date texts that Excel accepts. dates.md "The en-US
date-text grammar (DATEVALUE, VALUE, date arguments)" and "Date text with a month name glued to digits" hold the
probed rules (T-0024, T-0159, T-0160, T-1136, T-1144, T-1788, T-1906); open T-3299 owns the glued month-name shapes
dates.md leaves UNVERIFIED. No task has surveyed the shapes the grammar may be missing. Survey at least: ISO
yyyy-mm-dd and yyyy/mm/dd with and without leading zeros; dotted dates (1.7.2026); two-digit and one-digit years and
their century pivot; abbreviated and full month names in every position (Jul 1 2026, 1 Jul 2026, 2026 Jul 1, July
2026, Jul-2026); month-name abbreviations with a period (Sept., Jul.); weekday names before a date (Wednesday, July
1, 2026); ordinal suffixes (1st, July 1st); four-digit years at 1900, 1899, 9999 and 10000; mixed separators
(7/1-2026); extra internal spaces; leading/trailing whitespace kinds; day overflow (2/30/2026); and date plus time
forms (T separator, am/pm, seconds, fractions). Use DATEVALUE and VALUE side by side and one date-argument function
(e.g. YEAR) to show the three share one reader.
Root cause first: DATEVALUE, VALUE, NUMBERVALUE and date arguments share the text-to-number reader (coercion.md,
functions/date_and_time.rs `fn_datevalue` and the shared parser it calls; `rg` for the date-text parser before adding
any rule). Fix the shared reader, never DATEVALUE alone.

## Acceptance criteria
- [ ] A probe case (excel-graph) covers every shape family above through DATEVALUE, VALUE and YEAR.
- [ ] The engine matches every probed cell; dates.md's grammar section states each accepted and refused family.
- [ ] Shapes T-3299 owns are left to it.
- [ ] Every quirk found is written up in docs/excel-semantics/dates.md in the same PR, with its probe cells cited.
- [ ] `make check` green; eval known failures only shrink (baseline updated); reviewers approve.

## Out of scope
- Typed-entry cell formats for date text (typed input work).
- 1904 workbooks.
