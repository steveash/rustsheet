---
id: T-2226
title: "Scientific formats: engineering mantissa rounding after the exponent shift (##0.0E+0 on 1/3 is 333.3E-3)"
milestone: M5
lane: functions-text
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Engineering scientific formats round the mantissa to the displayed digits after the exponent shift.

## Context
Follow-up of T-1730. `format.rs` rounds the value to `precision + len(floor(value))` digits before the exponent is chosen, so a mantissa with several integer digits loses digits: `##0.0E+0` on 1/3 Excel "333.3E-3", ours "330.0E-3"; "666.7E-3"; "-142.9E-3"; `##0.00E+00` "333.33E-03", "-142.86E-03", "11.57E-06"; `00.0E+0` "12.5E-2", "11.6E-6" (gen-m10 05c1c646c95a2d60 F!T12, 659cd5776d423ce3 F!J7, ~30 cells).
A first attempt (round the mantissa after the shift: skip the early rounding for scientific codes) regressed ~15 gen-m10 text cells (e.g. 4c6a49fe4a6fa404 F!R15, 5255389143dbcef6 F!D16, 8e073c5b2145567a F!H14), so the exponent choice and the rounding interact; reproduce with `make eval` before changing. `digit_count` as the group size for codes with `,` grouping is unprobed.

## Acceptance criteria
- [ ] The listed gen-m10 text cells pass, no regressions; number-formats.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
