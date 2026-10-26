---
id: T-2793
title: "CONVERT is #VALUE! for a boolean unit read from a cell (3 gen-m9 cells)"
milestone: M9
lane: functions-engineering
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
`CONVERT(number, from_unit, to_unit)` is `#VALUE!`, as in Excel, when from_unit or to_unit is a reference to a TRUE or
FALSE cell, instead of our unknown-unit `#N/A`. The 3 gen-m9 cells below pass.

## Context
Gap triage of the T-2676 sweep (gen-m9 round 2, excel-graph, 2026-10-26). In each cell the number is a valid number and
the unit argument is a one-cell reference to a boolean (checked with the engine on the case's Data sheet); Excel gives
`#VALUE!`, ours `#N/A`:
- gen-m9/290180acf017709c Calc!A148 `=CONVERT(2^31-1,Data!C6,"HP")` (C6 = FALSE, from_unit)
- gen-m9/290180acf017709c Calc!A158 `=CONVERT(1-2^-53,Data!E10,"mm")` (E10 = FALSE, from_unit)
- gen-m9/f81bd3b6dc373e5c Calc!A291 `=CONVERT(1E+21,"C",Data!A7)` (A7 = TRUE, to_unit)
Rule: docs/excel-semantics/engineering.md, "CONVERT: a boolean unit is #VALUE! (T-2676)". It extends the T-1385 rule
that a boolean number is `#VALUE!` before the units are read. Code: `fn_convert` in
crates/rustsheet-engine/src/functions/engineering/convert.rs (how the unit arguments are read into text).

## Acceptance criteria
- [ ] These 3 known-failure lines disappear from eval/baseline/gen-m9.txt with no new ones.
- [ ] A unit test in crates/rustsheet-engine/src/test/engineering/test_convert.rs covers a TRUE and a FALSE cell as
      from_unit and as to_unit.
- [ ] engineering.md's entry gets "Ours: since T-2793".
- [ ] `make check` green; reviewers approve.

## Out of scope
- A typed boolean unit (`CONVERT(1,TRUE,"m")`) and a boolean array element as a unit: no Excel cell yet, so leave ours
  as it is (the entry marks them unprobed).

## Visible-parity stage
`semantics`: the 3 cells have the wrong error; the task is done when each is `#VALUE!`.
