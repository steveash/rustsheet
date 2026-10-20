---
id: T-2047
title: "CONVERT to a non-base unit: fit Excel's last digit (m to ft is 3.2808398950131235)"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-2046]
---

## Goal
CONVERT matches Excel bit for bit when converting from a base unit to a non-base unit, starting with
`CONVERT(1,"m","ft")` = 3.2808398950131235, from a factor table or order fitted on the T-2046 sweep.

## Context
Needs an oracle sweep: T-2046 (only two m→ft points exist).
- Known failures (gap triage 2026-10-19): probes/scalar-lift-open-a Sheet1!Z8, Z9, Z10
  `CONVERT(1,"m","ft")`: Excel 3.2808398950131235, ours 3.280839895013123.
- The current rule (docs/excel-semantics/engineering.md "CONVERT constants and evaluation order":
  `value * F_from / F_to`, prefixes after) fits all 509 CONVERT.xlsx cells, which only go unit → base.
  A reciprocal factor for ft (K = 3.2808398950131235; m→ft `v * K`, ft→m `v / K`) fits the four
  known ft cells (m→ft at 1 and 2.5, ft→m at 1 and 4); x / 0.0254 / 12 fits the two m→ft cells but
  not ft→m. Decide per unit or per class with the sweep, and keep every currently passing
  CONVERT cell passing.
- Engine: `fn_convert` and the unit table in crates/rustsheet-engine/src/functions/engineering/convert.rs.

## Acceptance criteria
- [ ] The three known-failure lines (probes/scalar-lift-open-a Sheet1!Z8, Z9, Z10) leave
      eval/baseline/probes.txt with no new lines in any suite.
- [ ] The T-2046 sweep matches >= 99.9% bit for bit (or the misses are listed in engineering.md and
      filed), and CONVERT.xlsx and probes/convert-targets still pass in full.
- [ ] engineering.md's CONVERT entry states the fitted rule with the sweep cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Temperature conversions; CONVERT error order (T-1385).
