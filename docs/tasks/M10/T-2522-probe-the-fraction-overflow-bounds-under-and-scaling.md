---
id: T-2522
title: "Probe the fraction overflow bounds under % and , scaling and large fixed denominators"
milestone: M10
lane: functions-text
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Fraction formats with `%` or `,` scaling and with large fixed denominators hit Excel's overflow bounds exactly where Excel does.

## Context
T-1740 extrapolated the fraction overflow bounds (the 2^31 - 1 placeholder value bound, 32767 for fixed numerators) to scaled values (`?/?%`, `?/100,`) and to denominators past 32767 (`?/40000`) with no evidence. T-2233 fitted the placeholder stop rule (the product of a partial quotient and the last numerator past 2^31 - 1) on unscaled values only. Both are marked UNVERIFIED in docs/excel-semantics/number-formats.md ("Fraction formats", "Placeholder fractions past 2^30"). Code: `format_fraction` and `last_convergent` in crates/rustsheet-engine/src/formatter/fraction.rs. Probe with the excel-probe skill and commit the probe cells with whatever fix makes them pass.

## Acceptance criteria
- [ ] A probe case covers `?/?%`, `??/??%`, `?/100,`, `?/?,` and `?/40000` around each bound (values just under and past the bound once scaled), as displayed text and TEXT, and every cell passes.
- [ ] number-formats.md records the result and drops the UNVERIFIED notes it settles.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
