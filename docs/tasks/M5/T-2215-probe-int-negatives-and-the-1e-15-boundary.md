---
id: T-2215
title: "Probe INT negatives and the 1E+15 boundary"
milestone: M5
lane: functions-math
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's INT is established for negative values and for values at or above 1E+15, with the `fn_int` choices of T-2171 confirmed or corrected.

## Context
T-2171 cuts INT to 15 significant digits (`round_decimal`) below 1E+15 and floors a larger value as it is. Unprobed (docs/excel-semantics/numbers.md "INT cuts to 15 significant digits first"): `INT(-(1-2^-53))` (engine: -1), a negative 16th-digit tie, a negative value with a fraction in (-1E15,-1E14], the exact 1E+15 boundary, and a fraction between 1E+15 and 2^53 such as 1000000000000005.5. Use the `excel-probe` skill.

## Acceptance criteria
- [ ] Probe case(s) with Excel-computed values for the cases above are in eval/suites/probes, and `fn_int` matches them (or the difference is fixed).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
