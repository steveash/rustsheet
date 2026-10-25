---
id: T-2641
title: "IM* inumber: ISO date text is #NUM!, a currency prefix on a complex number is read (probes/eng-inumber-reading A4, A5)"
milestone: M9
lane: functions-engineering
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
IM* functions read `"2024-01-01"` as `#NUM!` and `"$1+2i"` as the complex number 1+2i, as Excel does.

## Context
Found while doing T-2541 (probes/eng-inumber-reading, excel-graph 2026-10-25). Excel: `IMABS("2024-01-01")` is `#NUM!` (ours 45292, VALUE's date reading), while `IMABS("1/2")` reads as a date serial (46024 in 2026, year dependent) and `IMABS("2:24:00")`-style clock text is read. `IMABS("$1+2i")` is 2.23606797749979, i.e. the `$` prefix is accepted before a complex number (ours `#NUM!`). `IMABS("$1,234.50")` is 1234.5 and `IMABS("50%")` 0.5 (already matching). Unprobed: which other date shapes are refused (ISO with slashes, month names), a `$` on the imaginary part, `€`.
Code: `complex_of` and `parse_complex_number` in complex/mod.rs.

## Acceptance criteria
- [ ] probes/eng-inumber-reading A4 and A5 pass; any extra probe cells are oracle-filled.
- [ ] docs/excel-semantics/engineering.md "How an inumber is read" updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Date-dependent cells (a probe must not depend on the current year).
