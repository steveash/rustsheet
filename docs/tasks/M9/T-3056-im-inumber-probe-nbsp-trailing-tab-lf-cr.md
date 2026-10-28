---
id: T-3056
title: "IM* inumber: probe NBSP, trailing tab, LF/CR and £ ¥ prefixes (follow-up of T-2641)"
milestone: M9
lane: functions-engineering
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
IM* functions read leading NBSP, a trailing tab, LF/CR and £ ¥ prefixed inumber text as Excel does.

## Context
T-2641 found Excel drops `$` and `€` before a digit (probes/eng-inumber-reading A20:A26); these neighbours are unprobed. Code: `complex_of`, `strip_currency_symbols` in complex/mod.rs; docs/excel-semantics/engineering.md "How an inumber is read".

## Acceptance criteria
- [ ] Probe cells for each shape are oracle-filled and pass; engineering.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Date-dependent cells.
