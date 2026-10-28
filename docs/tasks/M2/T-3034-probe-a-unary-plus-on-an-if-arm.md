---
id: T-3034
title: "Probe a unary plus on an IF arm and on OFFSET's reference under legacy N (T-2885 follow-up)"
milestone: M2
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Know what legacy `N(IF(TRUE,+B1:B3))` and `N(OFFSET(+B1:B3,0,0))` do in Excel (value or `@`, FORMULATEXT), and make
`is_plain_reference` match.

## Context
T-2885 made a plus on the whole N/T argument intersect (probes/t2772-unary-plus-in-range-rows J2, J5). A plus on an IF
arm or on OFFSET's first argument is unprobed; `is_plain_reference` (static_analysis/legacy.rs) still reads them as plain
references. Use the excel-probe skill; see docs/excel-semantics/syntax.md "A typed unary plus stays in the formula text".

## Acceptance criteria
- [ ] A probe case commits oracle values for both shapes, on and off the range's rows.
- [ ] `is_plain_reference` matches them; syntax.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
None. Also probe `N((+B1:B3))` (a plus under written parentheses).
