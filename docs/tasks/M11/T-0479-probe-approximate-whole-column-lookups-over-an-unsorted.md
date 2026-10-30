---
id: T-0479
title: "Probe approximate whole-column lookups over an unsorted cached linked column"
milestone: M11
lane: functions-lookup
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for what Excel returns from an approximate VLOOKUP, HLOOKUP and MATCH (types 1 and -1) over a
whole column or row of a cached linked sheet whose data is unsorted, so docs/excel-semantics/references.md can drop
its "Unprobed" note (T-0474).

## Context
- T-0474 reads whole columns of a cached linked sheet at full length with an empty tail, so approximate searches
  probe the same midpoints as over a local whole column. A unit test (test_external_links.rs
  `whole_column_lookups_keep_full_length`) pins this to local-range behaviour, but no oracle cell covers it.
- With cache column A = 1,5,2,9,3, the engine gives 1 for `VLOOKUP(4,[1]Uses!$A:$C,1,TRUE)`. A vector cut to the
  cache's extent would give 2.
- Build: a new fixture derived from `external-link-uncached-lookups.xlsx` (eval/suites/fixtures/README.md describes how
  that one was built), with an unsorted cached column and a row on the linked sheet. Fill it with
  `cd tools && uv run rustsheet-oracle fill`.

## Acceptance criteria
- [x] The fixture has oracle values for at least VLOOKUP TRUE, MATCH 1, MATCH -1 and HLOOKUP TRUE over unsorted cached
      whole columns/rows.
- [x] references.md cites those cells. A mismatch is recorded as a known failure, with a follow-up task.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing the engine. If Excel disagrees, file a fix task.

## Outcome
Built `external-link-unsorted-approx-lookups.xlsx` (23 cells) and filled it from excel-graph. The engine matches Excel
on every cell, so no engine change or follow-up; references.md now cites the cells. Fixtures suite 325 -> 348 cells,
all passing; no known-failure change (the baseline gains the new case's C line only).
