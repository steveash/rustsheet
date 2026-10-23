---
id: T-2233
title: "Placeholder fractions past 2^30: fit Excel's choice of fraction"
milestone: M10
lane: functions-text
status: done
priority: P3
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
A placeholder-denominator fraction (`?/?`, `??/??`) over a value past 2^30 picks the fraction Excel picks.

## Context
Found by T-1740. Excel's values are in docs/excel-semantics/number-formats.md "Fraction formats", "Placeholder
fractions past 2^30" (excel-graph 2026-10-21). They were kept out of probes/fraction-numerator-overflow2 because
the visible-parity lock refuses new visibly-off cells. Excel does not take the last convergent there:
`?/?` on 1073741824.5 is "1073741825/1" and on 2147483646.6 "4294967293/2", where the engine prints 2147483649/2 and
10737418233/5. No bound on the numerator, or on the denominator times the value, fits every cell. Candidate rules to
test: a relative-error stop, nearest-integer partial quotients, a 32-bit product in Excel's loop.
Code: `last_convergent` (crates/rustsheet-engine/src/formatter/fraction.rs). Re-probe with the excel-probe skill:
denser values between 2^29 and 2^31, x.5, x.25, x.75 and x.6, and `???/???`. Probe the T-1740 overflow bounds
under scaling and a large denominator too (`?/?%`, `?/100,`, `?/40000`), which T-1740 extrapolated without evidence. Commit the probe cells together with
the fix that makes them pass.

## Acceptance criteria
- [x] A new probe case holds the number-formats.md cells above plus the denser sweep, and every cell passes
  (displayed text and TEXT).
- [x] The fitted rule is recorded in number-formats.md, and the existing fraction cases (gen-m10,
  probes/fraction-*) have no regressions.
- [x] `make check` green; reviewers approve.

## Out of scope
- The `%`/`,`-scaled and large-fixed-denominator bounds (`?/?%`, `?/100,`, `?/40000`): not probed here, still UNVERIFIED in number-formats.md.

## Outcome
Fitted Excel's rule on probes/fraction-placeholder-past-2-30 (75 cells, excel-graph 2026-10-23, all passing):
the continued fraction stops where a partial quotient times the last numerator passes 2^31 - 1, though the new numerator
itself may pass it, and a stop before any fractional term rounds the value to an integer over 1. Implemented in
`last_convergent` (formatter/fraction.rs), recorded in number-formats.md. Eval: 75 new probe cells pass, no regressions.

## Visible-parity stage
Decision 0083: a wrong TEXT or displayed-text result is visible. The acceptance criteria above are the stage.
