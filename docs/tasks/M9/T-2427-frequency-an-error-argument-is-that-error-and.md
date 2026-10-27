---
id: T-2427
title: "FREQUENCY: an error argument is that error, and the result has one row per bin cell plus one even for a single or text bin (gen-m9, 140 cells)"
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
FREQUENCY returns its bins-plus-one column for a single bin (a number, a one-cell reference, a text or boolean bin cell)
and for a single text or boolean data value, as Excel does, and an error argument is that error (data before bins), instead
of a single number or `#VALUE!`.

## Context
Gap triage of the T-2255 gen-m9 sweep. statistics.md "FREQUENCY: one row per bin plus one, even for one bin or a text
bin; an error argument is that error (T-2255)" records it. 139 cells: 85 where an argument is an error (ours `#VALUE!`;
Excel the error: 37 `#NUM!`, 24 `#N/A`, 10 `#DIV/0!`, 8 `#REF!`, 3 `#CALC!`, and 3 `#SPILL!` where the error is lifted),
54 where Excel's result has at least two rows and so is `#SPILL!` in the generated layout (the next formula one row
below; ours a single number, or `#VALUE!` for a text/boolean argument), 8 of them nested in another call that then lifts
(2 of those end as `#N/A` and `#DIV/0!` through the enclosing call). Three FREQUENCY cells whose bins come from NPER, DB or
ODDLYIELD are T-2428's. The exact row count for a text
or boolean bin is not visible in these cells (at least two rows); if the implementation must choose, probe
`ROWS(FREQUENCY({1;2},"a"))` and `ROWS(FREQUENCY({1;2},TRUE))` first (excel-probe skill). T-2000 lists
`FREQUENCY(NA(),{1;2})` as unprobed; these cells answer the error part.
Code: `fn_frequency` in crates/rustsheet-engine/src/functions/statistical/frequency.rs.

Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 139 known-failure lines in eval/baseline/gen-m9.txt, 139 formulas.
Examples:
- gen-m9/05d44f6dd6cfc567 Calc!A126 `=FREQUENCY(CHISQ.DIST.RT(INDEX(Data!B9:D9,Data!E8),5),Data!$C2)`: Excel #NUM!, ours #VALUE!
- gen-m9/05d44f6dd6cfc567 Calc!A65 `=FREQUENCY((0*CHITEST(Data!D1,Data!A4:D9)),(-2.5))`: Excel #N/A, ours #VALUE!
- gen-m9/06b4e8f5de409d5e Calc!A239 `=FREQUENCY((SLN(30984,2.5,28)-COLUMN(Data!E10)),TAN(TRANSPOSE({"a","b";"c","d"})))`: Excel #SPILL!, ours #VALUE!
- gen-m9/0a614cfe974b59c3 Calc!A64 `=FREQUENCY(Data!B11:C12,Data!B3:B3)`: Excel #SPILL!, ours 1.0
- gen-m9/0a9226016e8f5f33 Calc!A127 `=FREQUENCY((-Data!$E$10*AVERAGEIF(Data!D10:E12,"a*",Data!D9:D10)),Data!B5:B10)`: Excel #DIV/0!, ours #VALUE!
- gen-m9/27601819397dd56e Calc!A89 `=FREQUENCY((Data!A8+INDEX(Data!B11:D12,0,4)),Data!A10:A11)`: Excel #REF!, ours #VALUE!
Cells:
- gen-m9/05d44f6dd6cfc567.case.json (4): Calc!A4, A65, A126, A370
- gen-m9/06b4e8f5de409d5e.case.json (3): Calc!A56, A117, A239
- gen-m9/0a614cfe974b59c3.case.json (2): Calc!A64, A184
- gen-m9/0a9226016e8f5f33.case.json (4): Calc!A81, A127, A173, A311
- gen-m9/146b46e74f2dac3b.case.json (2): Calc!A145, A328
- gen-m9/165db58d1540f014.case.json (3): Calc!A138, A184, A258
- gen-m9/27601819397dd56e.case.json (3): Calc!A89, A227, A365
- gen-m9/2801c1c11c21e134.case.json (2): Calc!A189, A372
- gen-m9/285d87cea1aaa73c.case.json (2): Calc!A320, A381
- gen-m9/2a18b2f5a4dc43ac.case.json (3): Calc!A104, A226, A287
- gen-m9/320d51765af9b6b7.case.json (2): Calc!A25, A255
- gen-m9/35e8168aae2b00f8.case.json (1): Calc!A251
- gen-m9/3a25a03eb94fcfbd.case.json (1): Calc!A70
- gen-m9/3e98b26cf6713e77.case.json (1): Calc!A249
- gen-m9/40425341c5034eed.case.json (2): Calc!A12, A242
- gen-m9/4250f880ab640c49.case.json (4): Calc!A59, A120, A181, A303
- gen-m9/4e55500bb8ad3cd8.case.json (1): Calc!A210
- gen-m9/589f42e67f4b4b8d.case.json (3): Calc!A62, A154, A384
- gen-m9/58e3253bbd3a7a82.case.json (2): Calc!A41, A179
- gen-m9/60b0b6e69ad35a45.case.json (4): Calc!A4, A32, A187, A370
- gen-m9/623663cacea53bbc.case.json (2): Calc!A187, A309
- gen-m9/68d8ffb69ff739d8.case.json (3): Calc!A16, A77, A382
- gen-m9/7674c89c824aca43.case.json (1): Calc!A238
- gen-m9/7bd219ec0c0c5e22.case.json (1): Calc!A45
- gen-m9/7c20351562a2d71d.case.json (2): Calc!A23, A201
- gen-m9/85f48d5cdfb24555.case.json (1): Calc!A377
- gen-m9/899f9555e67da2d6.case.json (3): Calc!A48, A232, A278
- gen-m9/8d3588d8df0812df.case.json (3): Calc!A51, A173, A234
- gen-m9/8dd2312c96effb1b.case.json (2): Calc!A149, A393
- gen-m9/8f2ac25945af87da.case.json (3): Calc!A43, A104, A348
- gen-m9/8fec7b207f4dd753.case.json (2): Calc!A62, A245
- gen-m9/984cba67bbe6972b.case.json (1): Calc!A293
- gen-m9/9cca17438a7e6f8a.case.json (2): Calc!A106, A350
- gen-m9/a013a45a8fbcab4f.case.json (3): Calc!A40, A223, A345
- gen-m9/a54c738c697617c3.case.json (2): Calc!A254, A335
- gen-m9/a5828d3925575713.case.json (4): Calc!A66, A158, A204, A388
- gen-m9/a612e89321a6df9c.case.json (2): Calc!A109, A155
- gen-m9/a61e547a1cab994c.case.json (1): Calc!A352
- gen-m9/a790a4d4a6a82756.case.json (2): Calc!A109, A170
- gen-m9/a7bf0d8d81d33ba6.case.json (3): Calc!A83, A327, A388
- gen-m9/abc2939ebd1e2365.case.json (1): Calc!A300
- gen-m9/b4173fe25109a238.case.json (4): Calc!A62, A123, A184, A245
- gen-m9/b6593d7dd13f2dac.case.json (3): Calc!A83, A205, A388
- gen-m9/b8ee55934c5b824c.case.json (3): Calc!A21, A113, A343
- gen-m9/bbdfc06cae0e0d71.case.json (2): Calc!A165, A349
- gen-m9/bc40d37cb4d8301d.case.json (3): Calc!A51, A112, A295
- gen-m9/bcb24385baafb2f0.case.json (2): Calc!A74, A257
- gen-m9/c18dbf21f11093f3.case.json (3): Calc!A3, A141, A233
- gen-m9/c468f5046aadfbbc.case.json (1): Calc!A184
- gen-m9/c6cfda6bd10cae93.case.json (2): Calc!A184, A245
- gen-m9/caa1eed48b1883fe.case.json (3): Calc!A130, A191, A313
- gen-m9/d8266bc15533702a.case.json (1): Calc!A157
- gen-m9/e6cf3b20a014e6a3.case.json (2): Calc!A171, A354
- gen-m9/eced4bb6a0234c9e.case.json (2): Calc!A96, A142
- gen-m9/ee2497a5e683c5d9.case.json (1): Calc!A311
- gen-m9/f2bd5f380ec002bf.case.json (3): Calc!A43, A135, A319
- gen-m9/f34f66fb7f81e673.case.json (4): Calc!A73, A211, A257, A349
- gen-m9/fd9fa22d212080d6.case.json (4): Calc!A32, A93, A100, A154
- gen-m9/fec39efaba9284c3.case.json (3): Calc!A13, A59, A197

T-2256 sweep (gen-m9 part 2, statistical M to Z; gap triage 2026-10-23): FREQUENCY nested in the statistical calls (a single bin spills, so the outer argument is an array or `#SPILL!`).

Owned gen-m9 cells (T-2256 triage, excel-graph, 2026-10-23): 12 known-failure lines in eval/baseline/gen-m9.txt, 12 visibly off.
Examples:
- gen-m9/17c110a53924beb8 Calc!A348 `=STANDARDIZE(0,FREQUENCY(0.2,Data!C9),10000000001)`: Excel #SPILL!, ours -9.999999999e-11
- gen-m9/5395b8a427764a3f Calc!A10 `=NORM.INV((F.INV(0.3,9,Data!$D$1)/Data!$C12),Data!E$8,(FREQUENCY(2.5,Data!E8)+Data!E9))`: Excel #SPILL!, ours #VALUE!
- gen-m9/a2262ea466fcc142 Calc!A300 `=STANDARDIZE((FREQUENCY(Data!E9,Data!B1:E5)+1.005),YEAR(17910),(-Data!C11-VALUE("?b")))`: Excel #SPILL!, ours #VALUE!
- gen-m9/cb61169f4c033d4b Calc!A137 `=VAR(Data!D8:D9,Data!B5,(FREQUENCY(Data!D6,Data!E8:E10)/(2.5*Data!E4)))`: Excel 12.25, ours #VALUE!
- gen-m9/eedd629f3d69fdda Calc!A266 `=T.TEST(FREQUENCY(Data!C4:C6,T.DIST.2T((-1),1E-10)),{3,3,3,3},2,3)`: Excel #NUM!, ours #VALUE!
Cells:
- gen-m9/17c110a53924beb8.case.json (1): Calc!A348
- gen-m9/44a8841352dc48e9.case.json (1): Calc!A312
- gen-m9/5395b8a427764a3f.case.json (1): Calc!A10
- gen-m9/787ba8d801105a60.case.json (1): Calc!A259
- gen-m9/9e2225ead8a6a07b.case.json (1): Calc!A169
- gen-m9/a2262ea466fcc142.case.json (1): Calc!A300
- gen-m9/a5342a7e24fd803e.case.json (1): Calc!A229
- gen-m9/bbab6562e88b70e8.case.json (1): Calc!A154
- gen-m9/cb61169f4c033d4b.case.json (1): Calc!A137
- gen-m9/d5fd9e05e9e6f466.case.json (2): Calc!A175, A312
- gen-m9/eedd629f3d69fdda.case.json (1): Calc!A266

T-2257 sweep (gen-m9 financial; gap triage 2026-10-23): FREQUENCY nested in the financial functions (`AMORDEGRC(..,FREQUENCY(..),1)` `#NUM!`, `MDURATION(..,FREQUENCY(..)-SUM(..),1)` `#SPILL!`).

Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 3 known-failure lines in eval/baseline/gen-m9.txt, 3 visibly off.
Examples:
- gen-m9/23d82b62e9a85bdd Calc!A206 `=DOLLARDE(TTEST((Data!$C4/Data!E1),NORMSDIST(Data!D3),IF(TRUE,Data!$D10,Data!A12),3),((Data!C$7*2)+FREQUENCY(Data!C$5,Data!C10)))`: Excel #SPILL!, ours #DIV/0!
- gen-m9/23d82b62e9a85bdd Calc!A258 `=AMORDEGRC(0.5,42,39112,571,4,FREQUENCY((Data!E9+Data!A8),(123456789.123^123456789.123)),1)`: Excel #NUM!, ours #VALUE!
- gen-m9/d0854366830dcdee Calc!A349 `=MDURATION(36651,44354,INDEX(Data!D3:D11,5),(FREQUENCY(Data!A3:D11,Data!A$5)-SUM(Data!C8:C10,Data!$C8)),1)`: Excel #SPILL!, ours #VALUE!
Cells:
- gen-m9/23d82b62e9a85bdd.case.json (2): Calc!A206, A258
- gen-m9/d0854366830dcdee.case.json (1): Calc!A349

gen-m9 engineering sweep (gap triage 2026-10-23, T-2258): `BESSELI(-FREQUENCY(Data!A9:D9,<"abc" cell>),..)` is `#SPILL!` (a text bin, two rows; ours `#VALUE!`; gen-m9/b5aef3645cec1b95 A72).
Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 1 known-failure lines in eval/baseline/gen-m9.txt, 1 formulas, 1 visibly off.
Examples:
- gen-m9/b5aef3645cec1b95 Calc!A72 `=BESSELI(-FREQUENCY(Data!A9:D9,Data!C$12),((Data!D11*0.1)^(Data!A3+1000000000000000.0)))`: Excel #SPILL!, ours #VALUE!
Cells:
- gen-m9/b5aef3645cec1b95.case.json (1): Calc!A72

T-2496 probe (probes/t2496-fin-subexpressions, each case's Data sheet copied as Dnn, every call and compound argument read through a TYPE/shape summary, excel-graph, 2026-10-25; probes/tbill-error-order-inner-calls for the T-bill cells): the innermost diverging sub-expression of 2 gen-m9 financial cells moved here from T-2496.

| gen-m9 cell | innermost sub-expression | Excel | ours |
|---|---|---|---|
| gen-m9/b6a02893a8d6b151 Calc!A245 | `FREQUENCY(2.5,Data!E10:E11)` (bins TRUE, FALSE: no number); ODDFYIELD lifts over it | 2x1 {0;1} (cell #SPILL!) | 1x1 {1} (cell #NUM!) |
| gen-m9/d0854366830dcdee Calc!A183 | `FREQUENCY(0.1,Data!A11:A11)` (bin "Hello World"); CUMIPMT lifts over it | 2x1 {0;1} (cell #SPILL!) | 1x1 {1} (cell #VALUE!) |


Owned gen-m9 cells (T-2524 trace, 2026-10-26; arguments evaluated by the engine, which agree with Excel): 1 known-failure
line in eval/baseline/gen-m9.txt, 1 visibly off:
- gen-m9/530c636749a85209 Calc!A174 `FREQUENCY((DSUM(..)/SUMIF(..)),COUPDAYS(..))`: data_array #DIV/0!, bins 180; Excel #DIV/0!, ours #VALUE!

## Acceptance criteria
- [x] Visible parity (gen-m9, T-2258): these 1 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] Visible parity (gen-m9, T-2257): the 3 visibly-off known-failure lines listed under "T-2257 sweep" in Context disappear
      from eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [~] (partly; rest in T-2624) Visible parity (gen-m9, T-2255 and T-2256): these 151 known-failure lines (139 from T-2255, 12 from T-2256) disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] Unit tests: `ROWS(FREQUENCY({1;2;3},2))` is 2, `FREQUENCY(NA(),{1;2})` is `#N/A`, an error bins is its error.
- [x] statistics.md's entry gets an "Ours:" line naming this task.
- [x] Visible parity (gen-m9, T-2496): the 2 known-failure lines of the T-2496 probe block above disappear from eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] `make check` green; reviewers approve.

## Out of scope
- FREQUENCY over a union (references.md, unchanged).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2255): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.


## Outcome
FREQUENCY skips text/booleans/blanks, returns two rows when no bin is numeric, and an error argument is the result
(scalar errors before array errors, data before bins); T.TEST lets an error `type` beat a bad `tails` when the samples are
good. Probe: probes/frequency-bins. Eval: gen-m9 known failures 5755 -> 5657 (98 net; 162 baseline lines removed, including cells of other functions that now pass; none new), probes baseline
unchanged apart from its new case. 19 FREQUENCY cells stay off because their other arguments are other functions' gaps
(ODDFYIELD/ODDLYIELD, FORECAST.ETS*, EXPAND, DB, NPER, WRAPCOLS, FILTER, INDIRECT/MAP); the acceptance lines for those are
not met; they move to T-2624 and the remaining gen-m9 lines stay in the baseline.

Review fixes (2026-10-27, after reviewer-excel-fidelity's REQUEST_CHANGES of 2026-10-26):
- An error element now propagates only from the data array (`numeric_flat_with_element_errors`, replacing the
  hand-rolled loop). An error element of the bins array keeps `main`'s `#VALUE!`, because no oracle cell covers it
  and corpus-bench/53062 points the other way (T-2636).
- An error cell in a data range keeps `main`'s data-first order against a scalar bins error, UNVERIFIED (T-2636).
  A scalar bins error still beats an error element of the data array (probes/frequency-bins B1-B3).
- The T.TEST change went to `main` separately (`t_dist.rs` takes `main`'s side in the merge), and its test cells
  still pass.
- The T-2636 notes are refreshed (statistics.md, the prob.rs TODO, the T-2636 task). The `"2"` text-bin rule stays
  UNVERIFIED.
- Eval on main 7f6b373a merged in: gen-m9 3,753 -> 3,598 (155 fixed), probes 2,891 -> 2,886 (5 fixed;
  frequency-bins 38/38), 0 regressions.
- The steward's blocker of 2026-10-27 (the same two gaps) is resolved by the routes it offered, without new probes:
  confine element errors to the data array, and keep `main`'s data-first order. Its probe list moves to T-2636.
