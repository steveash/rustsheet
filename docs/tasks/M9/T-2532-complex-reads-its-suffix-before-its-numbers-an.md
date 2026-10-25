---
id: T-2532
title: "COMPLEX reads its suffix before its numbers: an invalid or error suffix wins (gen-m9, 17 cells)"
milestone: M9
lane: functions-engineering
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
COMPLEX gives Excel's error when its suffix is bad and a number argument is an error: the suffix is read first, so a suffix other than "i", "j" or empty text is `#VALUE!` and an error suffix is that error, before real_num's or i_num's error.

## Context
Gap triage of the T-2258 gen-m9 engineering sweep. docs/excel-semantics/engineering.md "Engineering functions in the gen-m9 engineering sweep (T-2258)" records the rule from these cells. In each, real_num or i_num is an error from an inner call (ODDLPRICE, CRITBINOM, BITAND past 2^48, QUARTILE.INC, COUPPCD, RANK, BINOMDIST, KURT, LINEST, TRIMMEAN, LOGINV, STEYX, INT, GETPIVOTDATA, EDATE, CHISQ.INV, SECOND) and the suffix is invalid (a number cell, TRUE, "J", "a", "ü", other text) or an error (`#N/A`, `#CALC!`, `#REF!`); Excel gives the suffix's `#VALUE!` or error whatever the number errors are, so the inner calls do not decide. Ours reports real_num's or i_num's error. engineering.md "The second argument's error wins ..." (COMPLEX reports the first argument's error before the second's) still holds between the two numbers. gen-m9/df083d85b436b49a A120 (COMPLEX over SWITCH(F.TEST(..))) also needs T-2093 and is owned by T-2552.
Code: `fn_complex` in crates/rustsheet-engine/src/functions/engineering/complex.rs (the suffix match near `Suffix::I`).

Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 17 known-failure lines in eval/baseline/gen-m9.txt, 17 formulas, 17 visibly off.
Examples:
- gen-m9/060b18586fcff6e6 Calc!A144 `=COMPLEX(RANK.AVG((3.14159/0.2),Data!A6:C9,ODDLPRICE(35804,39399,42,(-1),1.005,101,2,0.2)),Data!E4,"J")`: Excel #VALUE!, ours #NUM!
- gen-m9/09e6a8028f5acfae Calc!A171 `=COMPLEX((CRITBINOM(10,Data!E$5,Data!E$8)+AVERAGE(Data!D2:D2,Data!E2:E10,Data!D9)),ARABIC((Data!C8&"1.5")),(BIN2HEX(0,1)&VALUETOTEXT("a",0)))`: Excel #VALUE!, ours #NUM!
- gen-m9/2b0324e2de2e7b74 Calc!A62 `=COMPLEX(TEXTAFTER((Data!E5&Data!C2),Data!A7:D7,BITAND(444,281474976710656),1),Data!D5,TRUE)`: Excel #VALUE!, ours #NUM!
- gen-m9/408aa11aa923a4d9 Calc!A119 `=COMPLEX((QUARTILE.INC(5,(-1))*PV(42,12.5,-107)),(-2.5),CONCAT(IF(Data!E$5,Data!$C1),VSTACK(Data!A5:D5,Data!B4:B5,Data!B$2),CHOOSEROWS(Data!C3:E5,-1)))`: Excel #N/A, ours #NUM!
- gen-m9/4e180f54ff140abc Calc!A150 `=COMPLEX(STDEV.S({1;2;"";4},Data!B5:B9,COUPPCD(39427,Data!C6,1e-15,3)),Data!C1,CLEAN(VLOOKUP("x~*",Data!B5:E8,Data!$C7)))`: Excel #N/A, ours #NUM!
- gen-m9/62c886e21a1042af Calc!A149 `=COMPLEX(RANK((Data!C$11-Data!$A7),Data!A3:D12,1),2^53-1,WRAPROWS(Data!$E$11,1,ISERROR(Data!A10)))`: Excel #VALUE!, ours #N/A
Cells:
- gen-m9/060b18586fcff6e6.case.json (1): Calc!A144
- gen-m9/09e6a8028f5acfae.case.json (1): Calc!A171
- gen-m9/2b0324e2de2e7b74.case.json (1): Calc!A62
- gen-m9/408aa11aa923a4d9.case.json (1): Calc!A119
- gen-m9/4e180f54ff140abc.case.json (1): Calc!A150
- gen-m9/62c886e21a1042af.case.json (1): Calc!A149
- gen-m9/65ff9a692665eba9.case.json (1): Calc!A389
- gen-m9/6d16afd40600302b.case.json (1): Calc!A112
- gen-m9/7096c4cd615d43ac.case.json (1): Calc!A107
- gen-m9/74779cff2810ccf4.case.json (1): Calc!A20
- gen-m9/79dac8dde81afda9.case.json (1): Calc!A99
- gen-m9/7dc52cfcbc8b4448.case.json (1): Calc!A59
- gen-m9/b5aef3645cec1b95.case.json (1): Calc!A346
- gen-m9/c2d778463fe008ed.case.json (1): Calc!A14
- gen-m9/e85daedce5c2675e.case.json (1): Calc!A298
- gen-m9/e882060cf4613302.case.json (1): Calc!A52
- gen-m9/ee70a5515d004a66.case.json (1): Calc!A347

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2258): these 17 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Unit tests: `COMPLEX(1/0,1,"x")` is `#VALUE!`, `COMPLEX(NA(),1,1/0)` `#DIV/0!`, `COMPLEX(1/0,1,"j")` `#DIV/0!`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other T-2258 triage families: each has its own task (T-2258 Outcome).
## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2258): every cell this task owns has the wrong type, error, text or value (none is a last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria states it.

## Outcome
COMPLEX now reads the suffix before real_num and i_num. 18 gen-m9 known failures removed from the baseline (the 17 owned plus gen-m9/df083d85b436b49a Calc!A120, which also passes now); unit test `complex_suffix_error_wins_over_number_errors`; quirk recorded in engineering.md.
