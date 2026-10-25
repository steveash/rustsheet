---
id: T-2692
title: "F.TEST and FTEST: an error element inside an array argument is that error, not #VALUE! (gen-m9, 21 cells)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
F.TEST and FTEST return the error of an error element inside an array argument (an array constant such as
`{1,#N/A,3}` or a computed array such as `VSTACK(..)`, `TRANSPOSE(..)` or a GROUPBY result), as Excel does, instead of
`#VALUE!`.

## Context
Gap triage 2026-10-25 (`make eval`, excel-graph gen-m9 evidence). The rule is already recorded and VERIFIED:
docs/excel-semantics/statistics.md "An error element of an array argument is that error (T-2255)": `F.TEST({-1,0,1},{1,#N/A,3})`
is `#N/A` (165db58d1540f014 A337), and `FTEST({1,#N/A,3},AVERAGE(0.2))` is `#N/A`, not the one-point `#DIV/0!`
(9cca17438a7e6f8a A43). T-2430 implemented it for LARGE, SMALL, the percentile family and the paired functions and left
F.TEST/FTEST to T-2093; T-2093 (done) fixed the scalar and error-argument cases but not this one, so these cells have had no
open owner since.

Code: `f_test_sample` in crates/rustsheet-engine/src/functions/statistical/fisher.rs. Its `CalcResult::Array` arm maps every
`values_from_array` error (crates/rustsheet-engine/src/functions/statistical/chisq.rs) to `#VALUE!`. `values_from_array` already
returns the element's own `Error` for an `ArrayNode::Error`, and `Error::VALUE` for a LAMBDA or nested element. Keep that
error kind (and keep `#VALUE!` for the LAMBDA and nested elements), so the error comes before the `n1 < 2 || n2 < 2`
one-point check. The first sample is read before the second, so the first argument's error wins (statistics.md "F.TEST: an
error argument is that error").

The four cells below that are not literal `{1,#N/A,3}` take the same path: the GROUPBY total and the `DAYS360(..)/TRANSPOSE(..)`
quotient hold `#DIV/0!` elements, `INDIRECT(..,WRAPROWS(..))` lifts to `#REF!` elements, `UNIQUE(..)-RANK.EQ(..)` to `#N/A`
elements, and `VSTACK` pads with `#N/A`. If one of them still differs after the fix, file it as a follow-up and do not
widen this task.

Examples (`make eval-show CASE=gen-m9/<id>.case.json`), Excel vs ours:
- gen-m9/165db58d1540f014 Calc!A337 `=F.TEST({-1,0,1},{1,#N/A,3})`: Excel #N/A, ours #VALUE!
- gen-m9/9cca17438a7e6f8a Calc!A43 `=FTEST({1,#N/A,3},AVERAGE(0.2))`: Excel #N/A, ours #VALUE!
- gen-m9/7674c89c824aca43 Calc!A9 `=FTEST(Data!A1:D3,_xlfn.GROUPBY(Data!A1:A12,Data!A1:B12,_xleta.AVERAGE,1))`: Excel #DIV/0!, ours #VALUE!
- gen-m9/85f48d5cdfb24555 Calc!A32 `=FTEST(Data!A7:D10,(DAYS360(50233,828)/TRANSPOSE(Data!C1:C5)))`: Excel #DIV/0!, ours #VALUE!
- gen-m9/7c20351562a2d71d Calc!A304 `=F.TEST(Data!B1:D1,INDIRECT((Data!C2&Data!D$1),WRAPROWS({1;2;3},3)))`: Excel #REF!, ours #VALUE!
- gen-m9/2138b909d8f810ea Calc!A105 `=PERCENTRANK.INC(Data!C3:C11,(FTEST(Data!B7:E8,{1,#N/A,3})^GAMMADIST(..)))`: Excel #N/A, ours #VALUE! (nested)

Owned gen-m9 cells (gap triage 2026-10-25, excel-graph): 21 known-failure lines in eval/baseline/gen-m9.txt, all 21 visibly off
(a wrong error):
- gen-m9/06b4e8f5de409d5e.case.json (1): Calc!A383
- gen-m9/0a614cfe974b59c3.case.json (1): Calc!A206
- gen-m9/165db58d1540f014.case.json (1): Calc!A337
- gen-m9/2138b909d8f810ea.case.json (1): Calc!A105
- gen-m9/2a18b2f5a4dc43ac.case.json (2): Calc!A62, A245
- gen-m9/320d51765af9b6b7.case.json (1): Calc!A269
- gen-m9/3a25a03eb94fcfbd.case.json (1): Calc!A374
- gen-m9/4250f880ab640c49.case.json (1): Calc!A328
- gen-m9/4bad303d32cc3f46.case.json (1): Calc!A22
- gen-m9/4e55500bb8ad3cd8.case.json (1): Calc!A105
- gen-m9/530c636749a85209.case.json (1): Calc!A272
- gen-m9/60b0b6e69ad35a45.case.json (1): Calc!A239
- gen-m9/623663cacea53bbc.case.json (1): Calc!A34
- gen-m9/7674c89c824aca43.case.json (1): Calc!A9
- gen-m9/7c20351562a2d71d.case.json (1): Calc!A304
- gen-m9/85f48d5cdfb24555.case.json (1): Calc!A32
- gen-m9/9cca17438a7e6f8a.case.json (1): Calc!A43
- gen-m9/a5828d3925575713.case.json (1): Calc!A10
- gen-m9/bcb24385baafb2f0.case.json (1): Calc!A316
- gen-m9/d8266bc15533702a.case.json (1): Calc!A105

## Acceptance criteria
- [x] Visible parity: the 21 owned cells listed under Context have Excel's error, so these 21 known-failure lines leave
      eval/baseline/gen-m9.txt (`make eval-baseline`), with no new line in any baseline.
- [x] A unit test covers `F.TEST({-1,0,1},{1,#N/A,3})` (#N/A), `FTEST({1,#N/A,3},AVERAGE(0.2))` (#N/A, before the one-point #DIV/0!)
      and the existing `#DIV/0!` and `#VALUE!` (blank one-cell reference) cases in fisher.rs.
- [x] statistics.md "An error element of an array argument is that error (T-2255)": its "Ours" line says F.TEST and FTEST
      implement it (T-2692).
- [x] `make check` green; reviewers approve.

## Out of scope
- PROB and FREQUENCY (T-2636), the ERROR.TYPE/CELL `#NUM!` cells (T-2576) and INDIRECT's 1x1 array a1 argument (T-2577).
- The probes/t2436 ROWS/COLUMNS shadow cells of these formulas; they follow the gen-m9 cells.

## Visible-parity stage
Decision 0083: every owned cell is a wrong error, visible to anyone, so this task is in the `semantics` stage
(decision 0114). Its first acceptance criterion is the visible-parity target, and it is also the bit-level one.

## Outcome
- `f_test_sample` (fisher.rs) keeps the error kind `values_from_array` returns for an error element (a LAMBDA or nested
  element stays `#VALUE!`), so F.TEST and FTEST return the element's error before the one-point check.
- Eval: all 21 owned gen-m9 cells pass, including the GROUPBY, DAYS360/TRANSPOSE, INDIRECT/WRAPROWS, UNIQUE-RANK.EQ and
  VSTACK ones. So does one more cell with the same cause that the triage did not list, gen-m9/8f2ac25945af87da Calc!A9
  `=FTEST({1,#N/A,3},Data!B6:D9)`. `make eval-baseline` removed all 22 lines, and there are no regressions. Unit test `f_test_an_error_element_is_that_error`.
  statistics.md names T-2692 for F.TEST and FTEST.
