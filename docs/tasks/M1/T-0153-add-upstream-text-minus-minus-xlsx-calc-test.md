---
id: T-0153
title: "Add upstream TEXT/MINUS_MINUS.xlsx calc_test and the 10:11.5:12 clock oddity"
milestone: M1
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The upstream IronCalc `TEXT/MINUS_MINUS.xlsx` workbook passes whole as a calc_test, and `"10:11.5:12"` (timevalue-component-limits B26/C26) matches Excel.

## Context
Left over from T-0146. The workbook is `git show 9b1ae3fd:xlsx/tests/calc_tests/TEXT/MINUS_MINUS.xlsx` in the IronCalc upstream clone (`scripts/upstream-log.sh`); it needs time-text coercion (done in T-0146). The oddity: `TIMEVALUE("10:11.5:12")` = 0.424369212962963 (36665.5 s), see docs/excel-semantics/dates.md "Time text".

## Acceptance criteria
- [x] Probe and settle the unverified edges listed under dates.md "Time text" (hour-only AM/PM forms, VALUE of time text, direct arguments of AVERAGE/SUM, `+10:00` typed into a cell).
- [x] MINUS_MINUS.xlsx added under crates/rustsheet-xlsx/tests/calc_tests/TEXT/ and passes; B26/C26 leave eval/baseline/probes.txt.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Handoff (resumed)
Done on this branch: `TEXT/MINUS_MINUS.xlsx` (from IronCalc 9b1ae3fd) added under crates/rustsheet-xlsx/tests/calc_tests/TEXT/; it already passes whole (`make eval-show CASE=ironcalc-calc/TEXT/MINUS_MINUS.xlsx`: 73/73), so that acceptance line needs no engine change.
Not done: the `"10:11.5:12"` oddity (B26/C26 stay in eval/baseline/probes.txt) and probing the unverified edges listed in the acceptance criteria. Why: the only datum is `TIMEVALUE("10:11.5:12")` = 36665.5 s = 10h 11m 5.5s, and nothing says how a `H:M.f:S` string maps to that (the trailing `:12` seems to vanish); a rule needs several new oracle probes (vary the pieces: `10:11.5:13`, `10:12.5:12`, `10:11.25:12`, `1:11.5:12`) before implementing, which is beyond a sonnet-sized change. Next: write the probe case with those variants (excel-probe skill, oracle excel-graph is OK), fit the rule, implement in `time_text::parse_clock_text`/`parse_time_serial`, record in dates.md, drop B26/C26 via `make eval-baseline`.

## Outcome
- Three oracle probes (probes/clock-text-oddities, -2, -3; excel-graph) settle the `"10:11.5:12"`
  oddity: in `H:M.f:S` the fraction digits are read as whole seconds and as the fraction and `S`
  is checked then ignored; minutes plus the seconds' whole minutes must stay within 9999; one
  trailing `:` is ignored. Implemented in `time_text::parse_clock_text` (TIMEVALUE, VALUE,
  coercion, date-clock text). Recorded in docs/excel-semantics/dates.md "Time text".
- Direct arguments of SUM/AVERAGE/MAX/MIN/PRODUCT/LARGE/SMALL/HOUR/MINUTE/SECOND/DAY/array math
  probed: the engine already matched; COUNT does not count clock text (follow-up T-0252).
  `+10:00` typed into a cell is out of the oracle's reach (it uploads xlsx), noted in dates.md.
- MINUS_MINUS.xlsx (73/73) added by the sonnet run.
- Eval delta: probes known failures 105 -> 104 (fixed 2: timevalue-component-limits B26/C26;
  new coverage 197 cells, 1 failure: clock-text-oddities F37 = T-0252); regressions 0.
