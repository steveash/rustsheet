---
id: T-1766
title: "IM* text edge cases: empty-string argument, mixed i/j suffix, boolean argument, tiny-divisor magnitude, IMPOWER zero power and tiny-part formatting (probes eng-impower, eng-imsub, eng-imdiv)"
milestone: M9
lane: functions-math
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
IM* functions match Excel on empty-string, mixed-suffix, boolean and extreme-magnitude arguments, and number formatting of tiny parts.

## Context
Probes `probes/eng-impower`, `eng-imsub`, `eng-imdiv` failing cells (A3/A4/A12/A13/A17 IMSUB; A3/A4/A12/A17/A19/A23/A24 IMDIV; A6/A11/A13/A16/A22/A9 IMPOWER). See docs/excel-semantics/engineering.md (T-1625 entry). IMPOWER computes different values from our polar form (tiny parts 1.22514845490862E-16 vs 1.22464679914735E-16; A22 differs in both parts), so the fix is the algorithm, not just the formatter.

## Acceptance criteria
- [x] IMSUB and IMDIV probes pass; IMPOWER A6, A16 pass (done in the first PR, baseline shrank by 12)
- [ ] IMPOWER A9, A11, A13, A22 (extended-precision angle, see Handoff) pass
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Handoff (resumed)
Done: empty-text/mixed-suffix `#NUM!`, blank cell = 0, `IMPOWER(0,0)` `#NUM!`, IMDIV Smith's algorithm, scientific text for large parts. Remaining: IMPOWER A9/A11/A13/A22 differ numerically. Analysis (engineering.md T-1625 entry): Excel's `sin(pi)` is 1.22514845490862E-16 = pi minus the 64-bit x87 pi, so the angle (atan2, times n) is evidently held in 80-bit extended precision; A22 needs radius, power and trig all emulated. Next: emulate extended precision (double-double rounded to 64 mantissa bits) for atan2/pow/sin/cos and fit against the four cells, without special-casing them.
