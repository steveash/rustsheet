---
id: T-2641
title: "IM* inumber: a currency prefix on a complex number is read (probes/eng-inumber-reading A5); probe neighbouring whitespace and date shapes"
milestone: M9
lane: functions-engineering
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
IM* functions read `"$1+2i"` as the complex number 1+2i, as Excel does.

## Context
Found while doing T-2541 (probes/eng-inumber-reading, excel-graph 2026-10-25). Excel: `IMABS("2024-01-01")` is `#NUM!` (T-2541: text with an inner minus that is not an exponent's stays unreadable; slash dates are read, `IMABS("1/2/2020")` 43832, probes/eng-inumber-date-boundary). `IMABS("$1+2i")` is 2.23606797749979, i.e. the `$` prefix is accepted before a complex number (ours `#NUM!`). `IMABS("$1,234.50")` is 1234.5 and `IMABS("50%")` 0.5 (already matching). Unprobed: leading NBSP, a trailing tab, LF/CR; a `$` on the imaginary part, `€`.
Code: `complex_of` and `parse_complex_number` in complex/mod.rs.

## Acceptance criteria
- [x] probes/eng-inumber-reading A5 passes; any extra probe cells are oracle-filled.
- [x] docs/excel-semantics/engineering.md "How an inumber is read" updated.
- [x] `make check` green; reviewers approve.

## Out of scope
- Date-dependent cells (a probe must not depend on the current year).

## Outcome
`parse_complex_number` strips a leading `$` after the optional sign, so `IMABS("$1+2i")` is 2.23606797749979 and `"$-5"` stays `#NUM!`. probes/eng-inumber-reading A5 now passes (probes known failures 2926 -> 2925, no regressions). The neighbouring shapes (`$` on the imaginary part, `€`, NBSP, trailing tab, LF/CR, `-$1+2i`) are unprobed and listed under "Unprobed" in docs/excel-semantics/engineering.md.
