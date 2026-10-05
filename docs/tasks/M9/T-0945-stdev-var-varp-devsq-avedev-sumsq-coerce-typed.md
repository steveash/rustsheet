---
id: T-0945
title: "STDEV/VAR/VARP/DEVSQ/AVEDEV/SUMSQ: coerce typed text and boolean arguments; AVEDEV/DEVSQ with no numbers is #NUM! (13 poi known failures)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
STDEV, VAR, VARP, DEVSQ, AVEDEV and SUMSQ count a number typed as text (`"1"`) and a boolean typed
directly as an argument, as AVERAGE and SUM do; text and booleans inside ranges stay ignored.
AVEDEV and DEVSQ with no number at all return `#NUM!` (STDEV/VAR keep `#DIV/0!`).

## Context
Evidence: poi/FormulaEvalTestData_Copy.xlsx (Excel-cached, Excel 15), sheet EverythingTests
(`make eval-show CASE=poi/FormulaEvalTestData_Copy.xlsx`). Column D of the data block is blank,
column E is text.
- G1288 `STDEV("1","2")` 0.7071067811865476, K1288 `STDEV(TRUE, FALSE, TRUE)` 0.5773502691896258; actual `#DIV/0!`.
- G1448 `VAR("1","2")` 0.5, K1448 `VAR(TRUE, FALSE, TRUE)` 0.33333333333333337; actual `#DIV/0!`.
- G1456 `VARP("1","2")` 0.25, K1456 `VARP(TRUE, FALSE, TRUE)` 0.2222222222222222; actual `#DIV/0!`.
- I392 `DEVSQ("1","2",2)` 0.6666666666666666; actual 0.
- L176 `AVEDEV(TRUE, FALSE)` 0.5; actual `#DIV/0!`.
- M1332 `SUMSQ(TRUE, TRUE, FALSE)` 2, actual 0; N1332 `SUMSQ("2", "3", 4)` 29, actual 16.
- F176 `AVEDEV(D7:E9)`, G176 `AVEDEV(E13:E17)`, H392 `DEVSQ(E15:E17)` (no numbers): `#NUM!`; actual `#DIV/0!`.
Passing today and must stay so: F1288/F1448/F1456 `STDEV/VAR/VARP(E15:E17)` = `#DIV/0!`,
H1288 `STDEV(D14:D15,E13)` = `#DIV/0!`, G392 `DEVSQ(D7:D9,C9)` = 0.

Root cause: `fn_stdev_s`/`fn_stdev_p` (standard_dev.rs), `fn_var_s`/`fn_var_p` (variance.rs),
`fn_devsq` (devsq.rs), `fn_avedev` (count_and_average.rs) and `fn_sumsq`
(math_and_trigonometry/mathematical.rs) each hand-roll their argument loop and drop `String` and
`Boolean` results. `Model::for_each_value` (statistical/count_and_average.rs) already coerces typed
text/booleans and skips them in references (the T-0200 single aggregate iterator): route these
functions through it instead of their own loops (CLAUDE.md directive 3), keeping each function's
empty-result error (check each against the poi cells above: `for_each_value` returns `#VALUE!` for
non-numeric typed text).

Constraint: `for_each_value` counts booleans inside array literals as 1/0, while these six functions
ignore them today, and Excel ignores them for MIN/MAX (probes/min-max-value-args C11). Do not change
array-literal handling here: keep booleans and text in arrays ignored (until T-0236 settles it with
a probe), and add a unit test pinning `STDEV({TRUE,1,3})` to today's result. DEVSQ E392, F392 and
J392 are known last-digit failures (accumulation order): keep the summation order, and watch those
lines. If the diff grows past S, SUMSQ (math_and_trigonometry) may go in its own PR.

Rule recorded in docs/excel-semantics/coercion.md "STDEV / VAR / DEVSQ / AVEDEV / SUMSQ typed
arguments; AVEDEV and DEVSQ of nothing is #NUM!".

## Acceptance criteria
- [ ] These 13 lines disappear from eval/baseline/poi.txt with no new F lines in any baseline:
      `poi/FormulaEvalTestData_Copy.xlsx` EverythingTests!G1288, K1288, G1448, K1448, G1456, K1456,
      I392, L176, M1332, N1332, F176, G176, H392.
- [ ] Unit tests for each function with `"1"`, TRUE, and a range holding text/booleans (ignored).
- [ ] The hand-written argument loops are gone from the six functions (duplication does not grow).
- [ ] coercion.md entry gets an `Implemented in:` line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Booleans/text inside array literals and empty arguments (`STDEV({TRUE,1,3})`, `AVERAGE(5,)`):
  T-0236 probes those for every `for_each_value` user.
- STDEV.P/VAR.P/STDEVA/VARA variants not cited above: switch them to the same iterator only if no
  F line changes; no evidence here.

## Outcome
STDEV.S/.P, VAR.S/.P, DEVSQ, AVEDEV and SUMSQ now iterate with `Model::for_each_value` (hand-written
loops removed, about 250 lines fewer); AVEDEV/DEVSQ of no numbers return `#NUM!`. Unit tests added
for each. Eval delta: poi known failures 932 -> 921 (11 of the 13 listed fixed). I392 and K1456 stay
known failures: they are last-digit accumulation differences (DEVSQ/VARP summation order), which the
task said to keep; see the follow-up task.
