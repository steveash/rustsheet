---
id: T-2522
title: "Probe the fraction overflow bounds under % and , scaling and large fixed denominators"
milestone: M10
lane: functions-text
status: done
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
- [x] A probe case covers `?/?%`, `??/??%`, `?/100,`, `?/?,` and `?/40000` around each bound (values just under and past the bound once scaled), as displayed text and TEXT, and every cell passes.
- [x] number-formats.md records the result and drops the UNVERIFIED notes it settles.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed (excel-graph, 2026-10-27): probes/fraction-overflow-scaled (80 cells), fraction-large-fixed-denominator (61) and
fraction-six-digit-denominator (5); every cell passes. Findings (number-formats.md "Scaling commas in fraction formats",
"Fixed denominators past 32768"):
- `%` scaling: both overflow bounds and the 2^30 stop rule apply to the scaled value, as the engine already did.
- A `,` right after the denominator multiplies by 10^((-3n) mod 256) (10^253 for one comma, 10^250 for two), not
  divides by 1000; the scaled value is cut to 15 digits, and a result over 255 characters is 255 `#` / `#VALUE!`.
  A `,` after `%` or a literal is a literal (also `0%,`), and one after the integer is dropped.
- A fixed denominator past 32768 is picked as for placeholders (up to 9999999) and prints the code's digits cut to
  its length; six or more digits are `#VALUE!` from TEXT, and Excel for the web refuses a workbook that applies one.

This also settles T-2072 (#1950): probes/fraction-format-features rows 5-6 now pass.
Eval delta: probes known failures 2932 -> 2918 (14 fixed, fraction-format-features B5:I6), 146 new cells pass; 0 regressions.
