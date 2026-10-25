---
id: T-2641
title: "IM* inumber: a currency prefix on a complex number is read (probes/eng-inumber-reading A5); probe neighbouring whitespace and date shapes"
milestone: M9
lane: functions-engineering
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
IM* functions read `"$1+2i"` as the complex number 1+2i, as Excel does.

## Context
Found while doing T-2541 (probes/eng-inumber-reading, excel-graph 2026-10-25). Excel: `IMABS("2024-01-01")` is `#NUM!` (T-2541 keeps date-shaped text unreadable: a `/`, or a `-` past the first character). `IMABS("1/2")` is 46024 in 2026 (a year-dependent date serial; ours `#NUM!`, not probe-able). `IMABS("$1+2i")` is 2.23606797749979, i.e. the `$` prefix is accepted before a complex number (ours `#NUM!`). `IMABS("$1,234.50")` is 1234.5 and `IMABS("50%")` 0.5 (already matching). Unprobed: leading NBSP, a trailing tab, LF/CR; which other date shapes are refused (ISO with slashes, month names), a `$` on the imaginary part, `€`.
Code: `complex_of` and `parse_complex_number` in complex/mod.rs.

## Acceptance criteria
- [ ] probes/eng-inumber-reading A5 passes; any extra probe cells are oracle-filled.
- [ ] docs/excel-semantics/engineering.md "How an inumber is read" updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Date-dependent cells (a probe must not depend on the current year).
