---
id: T-1766
title: "IM* text edge cases: empty-string argument, mixed i/j suffix, boolean argument, tiny-divisor magnitude, IMPOWER zero power and tiny-part formatting (probes eng-impower, eng-imsub, eng-imdiv)"
milestone: M9
lane: functions-math
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
IM* functions match Excel on empty-string, mixed-suffix, boolean and extreme-magnitude arguments, and number formatting of tiny parts.

## Context
Probes `probes/eng-impower`, `eng-imsub`, `eng-imdiv` failing cells (A3/A4/A12/A13/A17 IMSUB; A3/A4/A12/A17/A19/A23/A24 IMDIV; A6/A11/A13/A16/A22/A9 IMPOWER). See docs/excel-semantics/engineering.md (T-1625 entry). IMPOWER computes different values from our polar form (tiny parts 1.22514845490862E-16 vs 1.22464679914735E-16; A22 differs in both parts), so the fix is the algorithm, not just the formatter.

## Acceptance criteria
- [ ] all failing cells of the three probes pass; baseline shrinks
- [ ] `make check` green; reviewers approve.

## Out of scope
-
