---
id: T-1401
title: "Paired-array statistical functions take a scalar or single cell as a one-value array (COVAR, SLOPE, INTERCEPT, STEYX, RSQ, PEARSON, FORECAST.LINEAR, CHISQ.TEST, T.TEST)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
CORREL, PEARSON, RSQ, SLOPE, INTERCEPT, STEYX, COVAR, COVARIANCE.P/.S, FORECAST/FORECAST.LINEAR, CHISQ.TEST/CHITEST and
T.TEST/TTEST accept a scalar or a single cell in an array argument as an array of one value, and then give Excel's usual
result: `#N/A` for unequal sizes, `#DIV/0!` (or COVAR 0) for a single point, the argument's own error, as in Excel.

## Context
T-1289's gen-m9 suite (excel-graph sidecars, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/1782efcecd5d4b1b Calc!A38 `=TTEST(Data!A7:C7,((Data!$D5-(-2.5))+Data!C1),1,1)` (3 cells vs a number): Excel `#N/A`, engine `#VALUE!`.
- gen-m9/1782efcecd5d4b1b Calc!A10 `=TAKE(..,(T.TEST(Data!B11,Data!A8:B9,0.1,Data!$E3)^0.1))` (B11 = FALSE, tails 0.1, type 0.3):
  Excel `#NUM!` (the tails/type check), engine `#VALUE!` (`T.TEST(Data!B11,Data!A8:B9,1,1)` alone is `#VALUE!`).
- gen-m9/3462856259a37891 Calc!A22 `=STEYX(Data!D4:D10,Data!B$6)`: Excel `#N/A`, engine `#VALUE!`.
- gen-m9/50f10c4062fb2e98 Calc!A3 `=IF(IFERROR((Data!D9/Data!E2),COVAR(0,Data!C6:D6)),..)`: Excel `#N/A`, engine `#VALUE!`.
- gen-m9/50f10c4062fb2e98 Calc!A17 `=COVAR(Data!D4:D8,DCOUNT(Data!A1:D9,3,Data!E2:E3))`: Excel `#N/A`, engine `#VALUE!`.
- gen-m9/50f10c4062fb2e98 Calc!A31 `=CHOOSE(1,(RSQ(Data!A4,Data!B11:C12)-..),..)`: Excel `#N/A`, engine `#VALUE!`.
- gen-m9/b58b6d7cd39d195f Calc!A2 `=IF(((Data!B6+(-0.5))>COVAR(0,Data!D6)),1,..)`: Excel 1 (COVAR of one pair is 0), engine `#VALUE!`.
- gen-m9/696af2c8cee2f57e Calc!A9 `=(ADDRESS(4,INTERCEPT(Data!$D$10,2.5))&EXPAND(..))`: Excel `#DIV/0!` (one point), engine `#VALUE!`.
- gen-m9/89693ab0cafc29df Calc!A30 `=(SLOPE(123456789.123,(Data!B8+1000000000000000.0))-2)`: Excel `#DIV/0!`, engine `#VALUE!`.
- gen-m9/b58b6d7cd39d195f Calc!A31 `=((INTERCEPT(Data!B9:E9,1e-15)+..)*..)`: Excel `#N/A`, engine `#VALUE!`.
- gen-m9/a2e51f48717af2a1 Calc!A34 `=(CHISQ.TEST(Data!B$4,Data!C5:D11)%*..)`: Excel `#N/A`, engine `#VALUE!`.
- gen-m9/adb3fd63c7368166 Calc!A26 `=LET(w,FORECAST.LINEAR((Data!C12-Data!E$10),POWER(Data!D1,Data!D$7),YEARFRAC(52660,2473,3)),..,w)`: Excel `#DIV/0!`, engine `#VALUE!`.
- gen-m9/adb3fd63c7368166 Calc!A33 `=(DATEVALUE("1/15/2024")/FORECAST.LINEAR(ROUND(Data!D3,(-0.5)),Data!C2:D7,MIN(..)))`: Excel `#N/A`, engine `#VALUE!`.
- gen-m9/bc0901ce6193d576 Calc!A12 `=(RADIANS(..)+(FORECAST.LINEAR(Data!$C9,0.2,1)+..))`: Excel `#DIV/0!`, engine `#VALUE!`.
- gen-m9/d244411dd90f7fbd Calc!A10 `=CONFIDENCE.NORM((FORECAST.LINEAR(Data!C$2,0.2,Data!D11:D12)-Data!E$1),..)`: Excel `#N/A`, engine `#VALUE!`.
- gen-m9/eefcc3989f31de76 Calc!A22 `=SORTBY(Data!A1:D12,PEARSON(Data!$D5,SEQUENCE(0)),..)`: Excel `#CALC!` (SEQUENCE(0)'s error), engine `#VALUE!`.
The engine already handles one-element array literals (`COVAR({0},{1,2})` is `#N/A`, `SLOPE({1},{2})` is `#DIV/0!`); only a
plain number or a single-cell reference is rejected (`COVAR(0,1)`, `SLOPE(1,2)`, `COVAR(0,Data!D6)` are `#VALUE!`). The shared
reader is `fn_get_two_matrices` (functions/statistical/, used by correl.rs, pearson.rs, covariance.rs, forecast.rs, chisq.rs);
T.TEST is `fn_t_test` (t_dist.rs). T-1292 changes the same reader for unequal sizes with text: check which lands first.
Semantics: docs/excel-semantics/statistics.md "Array arguments given a scalar, a single cell or an error".

## Acceptance criteria
- [ ] The 16 gen-m9 cells above pass in `./target/release/rustsheet-eval show gen-m9/<case>.case.json`, with no new failing gen-m9 cell and no new known failures in eval/baseline/.
- [ ] Unit tests: a number and a single cell as either argument of COVAR, SLOPE and FORECAST.LINEAR (one point and unequal sizes), and T.TEST with invalid tails and a single-cell array.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`paired_samples` (COVAR/COVARIANCE/T.TEST) and `fn_get_two_matrices` (CORREL, PEARSON, RSQ, SLOPE, INTERCEPT, STEYX,
FORECAST, CHISQ.TEST) take a number as a one-value array and an error as itself; unequal shapes are `#N/A`; T.TEST checks
tails/type first and paired samples of unequal size are `#N/A`. Probes eval: 76 cells fixed, 0 regressions.
Still failing in gen-m9: `GROWTH(GAMMA(..))` of a single value (T-1402 area).
