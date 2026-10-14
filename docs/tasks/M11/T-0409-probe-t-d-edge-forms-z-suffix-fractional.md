---
id: T-0409
title: "Probe t=d edge forms: Z suffix, fractional seconds, 1900-02-29, pre-1900, time-only"
milestone: M11
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Know what Excel gives `t="d"` cells holding a `Z` suffix, fractional seconds, `1900-02-29`, dates before 1900-01-01 and time-only text, and make the importer match.

## Context
T-0406 verified only the plain date, date-time, 1900-02-28 and 1900-03-01 forms (fixtures/t-d-iso-date-cells). `iso8601_to_serial` (crates/rustsheet-xlsx/src/import/iso_date.rs) guesses the rest; docs/excel-semantics/xlsx-storage.md lists them UNVERIFIED. Excel for the web rejects a workbook with unparsable text, so probe one form per fixture.

## Acceptance criteria
- [ ] One oracle-backed fixture per form (or a recorded rejection); semantics entry updated.
- [ ] `iso8601_to_serial` matches each result.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Five fixtures (fixtures/t-d-*) with excel-graph values: `Z` suffix ignored (32874.5), fractional seconds counted, `1900-02-29` = 60, time-only `12:00:00` = 0.5, pre-1900 date = `#VALUE!` cell. `iso8601_to_serial` now reads 1900-02-29 and time-only text and the importer (and eval snapshot) turn unparsable/pre-1900 `t="d"` text into `#VALUE!` instead of an empty cell. Semantics entry updated in xlsx-storage.md.
