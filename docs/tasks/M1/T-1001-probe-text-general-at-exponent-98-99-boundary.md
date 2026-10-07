---
id: T-1001
title: "Probe TEXT General at exponent 98/99 boundary and mantissa carry (E+98, E+99, 9.9999996E+98)"
milestone: M1
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's TEXT(x,"General") digit count at the exponent 98/99 boundary and for a mantissa that rounds up to 10 is known, and the engine matches it.

## Context
T-0986 cuts TEXT General to 5 digits at |exp| >= 99. The oracle has General cells only at +-5, -50 and +-99 and beyond; the 98/99 boundary is carried over from the `&""` path (probes/number-text-exponent-digits B36, B42). The exponent also comes from `log10().floor()` before rounding, so `9.9999996E+98` may print "10E+98" (pre-existing). See docs/excel-semantics/numbers.md "Exponent width"; code: the General branch in formatter/format.rs. Use the `excel-probe` skill.

## Acceptance criteria
- [ ] Probe cells for TEXT(x,"General") at E+98, E+99, E-98, 9.87654321098765E-99 and 9.9999996E+98 with oracle values; numbers.md updated; engine fixed if it differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `probes/text-general-exponent-boundary` (10 cells, excel-graph) confirmed the 98/99 boundary already implemented and showed the General branch printed "10E-10"-style text when the mantissa rounds up to 10 (5 cells failed). The branch now carries into the exponent. Eval: probes +10 passing cells (was 5/10 before the fix), 0 regressions.
