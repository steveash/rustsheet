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
defer: phase-1b
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
- T-2676 sweep (gen-m9 round 2, gap triage 2026-10-26, excel-graph), not visibly off: gen-m9/932b33b1ef8b7608 Calc!A14
  `=CONVERT((0--(-0.0001)),"in","mi")` (the number is -0.0001): Excel -1.5782828282828283E-9, ours -1.578282828282828E-9
  (one ulp). A non-base to non-base pair (in to mi), so the same factor question as m to ft; add in and mi to the
  T-2046 sweep's units if they are not there.
- Gap triage 2026-10-30, same m to ft value, no open task named them: probes/eng-convert-boolean-unit Sheet1!A7
  `=CONVERT(1,{"m",TRUE},"ft")`, A29 (spilled from A28 `=CONVERT(1,{TRUE;"m"},"ft")`) and B8 (spilled from A8
  `=CONVERT(1,"m",{TRUE,"ft"})`): Excel 3.2808398950131235, ours 3.280839895013123.
- Engine: `fn_convert` and the unit table in crates/rustsheet-engine/src/functions/engineering/convert.rs.

## Acceptance criteria
- [ ] The three known-failure lines (probes/scalar-lift-open-a Sheet1!Z8, Z9, Z10) leave eval/baseline/probes.txt, and
      gen-m9/932b33b1ef8b7608 Calc!A14 leaves eval/baseline/gen-m9.txt,
      with no new lines in any suite.
- [ ] The T-2046 sweep matches >= 99.9% bit for bit (or the misses are listed in engineering.md and
      filed), and CONVERT.xlsx and probes/convert-targets still pass in full.
- [ ] engineering.md's CONVERT entry states the fitted rule with the sweep cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Temperature conversions; CONVERT error order (T-1385).

## Deferred (decision 0114)
`defer: phase-1b`: the cells this task owns are already within 15 significant digits of Excel, so it waits until Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells visibly off (then give it a `campaign:` stage instead).
