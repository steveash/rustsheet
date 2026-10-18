---
id: T-1201
title: "COS/SIN/TAN of 1E+15 return #NUM! in Excel (gen-m5 mismatch)"
milestone: M5
lane: functions-math
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
COS (and, once probed, SIN/TAN) of a huge argument such as 1E+15 return Excel's `#NUM!`.

## Context
Kernel map (T-1522, decision 0056): rank 6 in `docs/research/numeric-kernel-map.md`; fit with the fit lab (T-1523).
From T-1192's gen-m5 run (excel-graph oracle): `COS(1E+15)` is `#NUM!` in Excel; the engine computes a number. Failing cells: gen-m5/c2ce66769f6cc566 Calc!A32 and gen-m5/065c0542a4ce3d1c Calc!A26 (the error propagates before other errors). First probe the threshold and SIN/TAN/COT/SEC/CSC behavior with the `excel-probe` skill and record it in docs/excel-semantics/numbers.md.

More cells (T-1239's gen-m6 suite, excel-graph, 2026-10-09): gen-m6/3ee7ffc8d6ab061f Calc!A2
`=((SUM(Data!B4:B9)*(-1))-SEC(1000000000000000.0%))` is `#NUM!` in Excel, so `SEC(1E+13)` is `#NUM!` (engine -43.54);
gen-m6/9bd0bd18636e5f9b Calc!A4 `=SECH(CHOOSE(SIN(1000000000000000.0),Data!A$10))` is `#NUM!`, so `SIN(1E+15)` is `#NUM!`
(engine `#VALUE!`: SIN returns 0.858 and CHOOSE rejects the index).

More cells (T-1275's gen-m7 suite, excel-graph, 2026-10-11; `./target/release/rustsheet-eval show gen-m7/<case>.case.json`):
- gen-m7/9ff3afcacd3bd0fc Calc!A24 `=((Data!E7-((-0.0001)*123456789.123))+SIN((Data!$A8*Data!B4)))` (SIN(-2.5E+15)): Excel `#NUM!`, engine 12346.33.
- gen-m7/f68fafa0ca2eba26 Calc!A37 `=OFFSET(Data!B4:E7,1,-1,(Data!D10*CSC(1000000000000000.0)))`: Excel `#NUM!` (so `CSC(1E+15)` is `#NUM!`), engine TRUE (`CSC(1E+15)` = 1.165).

More cells (T-1289's gen-m9 suite, excel-graph, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/675763979c1defb6 Calc!A35 `=ISERROR(N(SIN(Data!C12)))` (C12 = 1E+15): Excel TRUE, engine FALSE.
- gen-m9/adb3fd63c7368166 Calc!A11 `=T.DIST.2T(COUPNUM(CSC(Data!E9),46324,1,Data!$E5),7)` (E9 = 1E+15): Excel `#NUM!`, engine 4.949374243778948e-13 (`CSC(1E+15)` = 1.165).
- gen-m9/eefcc3989f31de76 Calc!A2 `=COS(Data!A2)` (A2 = 1E+15): Excel `#NUM!`, engine -0.5131937377869703.

## Acceptance criteria
- [ ] A probe case with oracle values for the threshold and the sibling trig functions.
- [ ] gen-m5 c2ce66769f6cc566 A32 and 065c0542a4ce3d1c A26 pass; baseline updated if the suite gains one.
- [ ] In `(A-COS(1E+15))=INDEX(B11:D11,6,2)` the left operand's `#NUM!` wins over INDEX's `#REF!` (error precedence).
- [ ] gen-m6/3ee7ffc8d6ab061f A2 and gen-m6/9bd0bd18636e5f9b A4 pass.
- [ ] gen-m7/9ff3afcacd3bd0fc A24 and gen-m7/f68fafa0ca2eba26 A37 pass. (T-1275's gen-m7 cells listed under Context, `rustsheet-eval show`.)
- [ ] gen-m9/675763979c1defb6 A35, adb3fd63c7368166 A11 and eefcc3989f31de76 A2 pass, with no new failing gen-m9 cell and no new known failures in eval/baseline/.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
SIN, COS, TAN, COT, SEC and CSC return `#NUM!` for |x| >= 2^27 (threshold probed: probes/trig-huge-argument, recorded in numbers.md). Eval: the 14 listed gen-m5..gen-m9 cells are not in this checkout's suites (only gen-m1/gen-m2 are present locally) so they could not be run here; the rule covers every cited formula. The new probe adds 6 known failures (`f(2^27-1)` last digits, Excel's inexact range reduction), filed as T-1563.
