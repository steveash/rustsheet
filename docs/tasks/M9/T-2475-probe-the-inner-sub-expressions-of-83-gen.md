---
id: T-2475
title: "Probe the inner sub-expressions of 83 gen-m9 statistical cells of the T-2256 sweep whose divergence is not traced"
milestone: M9
lane: research
status: done
priority: P2
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
Each of these 83 gen-m9 cells of the T-2256 sweep has its innermost diverging sub-expression identified from Excel values
(a probe case that evaluates the sub-expressions on the same Data sheet) and is then owned by a fix task, new or existing,
with any quirk recorded in docs/excel-semantics/.

## Context
Gap triage of the T-2256 gen-m9 sweep (the part-2 sibling of T-2436). The triage evaluated every sub-expression of the
1,953 new failures in the engine and traced 1,870 to a family; these 83 are left: the Excel value needs an inner function
(not a statistical M-to-Z one) to differ from ours, or an error order the cells do not settle. Inner functions involved:
NPV (9 cells; the hypothesis is that NPV with a rate of -1 is `#DIV/0!` and with a rate near 1E+300 `#DIV/0!` too, e.g.
`QUARTILE.EXC(LOG(NPV((-1),Data!B7:B11)),3)` is `#DIV/0!`, ours `#NUM!`), IRR (7: Excel `#NUM!` where ours returns a huge
rate, e.g. `STANDARDIZE(1,IRR(Data!C8:E12,1E-10),..)`; see T-1686), CELL("contents"/"row"/"col") (4), GETPIVOTDATA (2),
TOROW/TAKE/INDEX/SEQUENCE edge shapes, PRICE/YIELD/PV/DB domain checks, GROUPBY/PIVOTBY results, LAMBDA/REDUCE, and the
paired functions with two direct numbers (`SLOPE(CELL("row",Data!E3),-0.12..)` and `STEYX(3,1)` are `#NUM!` where
"Array arguments given a scalar, a single cell or an error" has `#DIV/0!` for one point). Method: copy each case's Data
sheet into a probe case (excel-probe skill), one formula per sub-expression of each cell, fill it through excel-graph,
compare with the engine, then file or extend the owning task for each family found (decision 0083: `campaign: visible`).
The triage's scripts evaluated the sub-expressions with the engine through a scratch suite (one probe cell per
sub-expression, `ROWS`/`COLUMNS`/`ISERROR` counts beside it); the same layout works for the probe.

Owned gen-m9 cells (T-2256 triage, excel-graph, 2026-10-23): 83 known-failure lines in eval/baseline/gen-m9.txt, 83 visibly off.
Examples:
- gen-m9/6e18550e9fc9c1b9 Calc!A69 `=STEYX(LINEST(Data!B7:D7),Data!A5:D9)`: Excel #VALUE!, ours #N/A
- gen-m9/928d08e3e1eecb5f Calc!A333 `=T.INV(TOROW(Data!E1,1),9)`: Excel #CALC!, ours #NUM!
- gen-m9/92d4fa29e5510f1e Calc!A98 `=MODE.SNGL(CELL("contents",Data!C$5))`: Excel #VALUE!, ours #N/A
- gen-m9/c4635963fc926284 Calc!A92 `=PERCENTILE.INC(Data!D4:D5,Data!D10)`: Excel #VALUE!, ours #NUM!
- gen-m9/d208bbafc3bc1997 Calc!A262 `=SKEW(Data!D11:D12,Data!D10)`: Excel 2.4980018054066022e-15, ours 2.9976021664879227e-15
Cells:
- gen-m9/0620505280d2ea6e.case.json (2): Calc!A4, A77
- gen-m9/097af1ca33638670.case.json (3): Calc!A14, A89, A358
- gen-m9/11eb3ab4f4f8104c.case.json (4): Calc!A90, A109, A160, A169
- gen-m9/17c110a53924beb8.case.json (1): Calc!A300
- gen-m9/1b31092a0d79e5f1.case.json (5): Calc!A143, A171, A179, A260, A343
- gen-m9/1db18890916ee643.case.json (2): Calc!A89, A320
- gen-m9/2138b909d8f810ea.case.json (2): Calc!A344, A358
- gen-m9/31f09e6c48ca245c.case.json (1): Calc!A232
- gen-m9/329b177b5d5165ad.case.json (1): Calc!A157
- gen-m9/334f3dc9ca937f10.case.json (2): Calc!A125, A226
- gen-m9/38e4fc309fe199ea.case.json (1): Calc!A89
- gen-m9/3b7990bd4e3307ba.case.json (2): Calc!A98, A261
- gen-m9/41248d1cb153c66e.case.json (1): Calc!A394
- gen-m9/44a8841352dc48e9.case.json (3): Calc!A7, A291, A318
- gen-m9/5919660b7bb4d195.case.json (1): Calc!A88
- gen-m9/5cebac1f02bf30f2.case.json (1): Calc!A311
- gen-m9/607e85cfee83a689.case.json (1): Calc!A28
- gen-m9/6e18550e9fc9c1b9.case.json (2): Calc!A69, A347
- gen-m9/73f053591a54bc43.case.json (1): Calc!A226
- gen-m9/775d053de498b8c7.case.json (3): Calc!A98, A112, A235
- gen-m9/7980ffb6095a340d.case.json (1): Calc!A146
- gen-m9/7fd0a98183c6babf.case.json (1): Calc!A240
- gen-m9/80586b859b73c87d.case.json (1): Calc!A67
- gen-m9/873f30d2bf76f8db.case.json (2): Calc!A96, A308
- gen-m9/928d08e3e1eecb5f.case.json (1): Calc!A333
- gen-m9/92d4fa29e5510f1e.case.json (2): Calc!A98, A236
- gen-m9/941c409532603d32.case.json (1): Calc!A148
- gen-m9/9bc247733a1ca9fa.case.json (3): Calc!A5, A207, A360
- gen-m9/9e2225ead8a6a07b.case.json (2): Calc!A100, A292
- gen-m9/a2262ea466fcc142.case.json (1): Calc!A316
- gen-m9/a4090567758520df.case.json (1): Calc!A101
- gen-m9/a5342a7e24fd803e.case.json (2): Calc!A166, A322
- gen-m9/a63d1c7d6f96c56d.case.json (3): Calc!A92, A111, A216
- gen-m9/acac71a65a5a1d1b.case.json (1): Calc!A27
- gen-m9/b9a32ce9a1fd0d71.case.json (3): Calc!A14, A142, A212
- gen-m9/bbab6562e88b70e8.case.json (1): Calc!A290
- gen-m9/bcb1105c61a2649d.case.json (2): Calc!A97, A230
- gen-m9/c2684066d8df263e.case.json (2): Calc!A65, A201
- gen-m9/c4635963fc926284.case.json (3): Calc!A12, A92, A153
- gen-m9/cb61169f4c033d4b.case.json (1): Calc!A272
- gen-m9/d208bbafc3bc1997.case.json (1): Calc!A262
- gen-m9/d37313406b9f6e55.case.json (2): Calc!A137, A182
- gen-m9/d8f44d556674c2fd.case.json (1): Calc!A398
- gen-m9/d910f000b60e2eac.case.json (1): Calc!A20
- gen-m9/eccd03df42e40006.case.json (1): Calc!A21
- gen-m9/ede0323c389fc703.case.json (1): Calc!A7
- gen-m9/eedd629f3d69fdda.case.json (1): Calc!A297
- gen-m9/f82473275380882e.case.json (2): Calc!A131, A143

## Acceptance criteria
- [x] Visible parity (gen-m9, T-2256): each of these 83 known-failure lines is owned by a `campaign: visible` follow-up
      (T-2618 to T-2623), which removes it from eval/baseline/gen-m9.txt; a probe task cannot fix them (Out of scope). The probe's
      121 new known-failure lines (written by `make eval-baseline`) are the divergences those follow-ups own.
- [x] A probe case per source case (or one combined case) holds the sub-expressions, filled from excel-graph and committed.
- [x] Every cell above is moved to a fix task (new or extended) whose Context lists it; each quirk found is in docs/excel-semantics/.
- [x] `make check` green; reviewers approve.

## Out of scope
- Fixing the inner functions (their own tasks).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2256): every cell it owns has the wrong type, error or value (none is a last-digit difference), so the task is in the campaign. Its target is that every cell it owns
has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value; the visible-parity
criterion under Acceptance criteria states it.

## Probe results (2026-10-24)
Probe: probes/t2475-stat-subexpressions (excel-graph, 2026-10-24): 574 cells, one per function call and compound argument of 81 of
the 83 owned cells (A4 and A216 of 0620505280d2ea6e and a63d1c7d6f96c56d read Pivot! through GETPIVOTDATA and are not copied), each on
a copy of its case's Data sheet (D01..D48) through a TYPE/shape summary: an error as `#<ERROR.TYPE>`, an array as
`64|<rows>x<cols>|<elements>`, a number as itself, anything else `<TYPE>|<value>` (column B names the cell and sub-expression).
Sub-expressions inside LAMBDA and LET bodies are not split out (they read the bodies' names). 453 of 574 match Excel.
The innermost diverging sub-expression of each cell (a mismatch none of whose own probed sub-expressions mismatches; a bare
`LAMBDA(..)` sub-expression is skipped: as a LET value it is `#VALUE!` in Excel and `#N/IMPL!` in ours, an artefact of the wrapper
that is itself a quirk to record):

| cell | probe | innermost diverging sub-expression | Excel | ours |
|---|---|---|---|---|
| 0620505280d2ea6e A77 | P!A2 | `IRR(D01!C4:E7,-D01!C8)` | "#6" | 31622775.851604085 |
| 097af1ca33638670 A358 | P!A28 | `PERMUTATIONA(D02!D$12,D02!A12)` | 1.0 | "#6" |
| 11eb3ab4f4f8104c A90 | P!A35 | `COMBINA((-0.5),D03!$A3)` | 1.0 | "#6" |
| 11eb3ab4f4f8104c A109 | P!A39 | `PV(-D03!B$1,0,-713)` | "#6" | "#2" |
| 11eb3ab4f4f8104c A160 | P!A45 | `PRICEDISC(D03!$A4,D03!D2,0.3,103,(-2.5))` | "#6" | "#3" |
| 11eb3ab4f4f8104c A169 | P!A58 | `T.TEST((D03!A11+(-0.5)),(42/D03!$D6),1,(-1))` | "#3" | "#6" |
| 1b31092a0d79e5f1 A143 | P!A70 | `TAKE(LET(v,D05!A$11,acc,v,y,"?b",acc),-1,-1)` | 0.0 | "64|1x1|0" |
| 1b31092a0d79e5f1 A171 | P!A75 | `PERCENTRANK.INC(D05!A7:D8,2,-1e-15)` | "#7" | "#6" |
| 1b31092a0d79e5f1 A179 | P!A78 | `WEIBULL.DIST(IRR({-100,100}),1E+8,NORMDIST(40,(D05!A11*1),MAX(D05!C1:C8,D05!C8:C10),(D05!$` | 0.0 | "#6" |
| 1b31092a0d79e5f1 A260 | P!A92 | `NPER(1E-300,-113,3037,368,1)` | "#2" | "#6" |
| 1b31092a0d79e5f1 A343 | P!A96 | `MODE.MULT(D05!A8:A8,D05!C7:C10,D05!D3:D11)` | "#3" | "64|2x1|2.5;1.005" |
| 1db18890916ee643 A320 | P!A116 | `NPV((-1),(-0.5),D06!A$9,D06!B8:B11)` | "#2" | #NUM! |
| 2138b909d8f810ea A344 | P!A124 | `T.INV.2T(D07!$A4,1)` | "#2" | "#6" |
| 2138b909d8f810ea A358 | P!A127 | `BETA.INV(0.025,1e-15,2.5)` | "#6" | 2.518020676354722e-31 |
| 31f09e6c48ca245c A232 | P!A136 | `PV((-1+2^-52),165,-978,224,0)` | "#2" | "#6" |
| 329b177b5d5165ad A157 | P!A148 | `NPV((-1),1.005,D09!$E1,D09!C11)` | "#2" | #NUM! |
| 334f3dc9ca937f10 A226 | P!A158 | `STDEV.P(D10!A9:A12,IFERROR(1/0,D10!$E6),{1,2,3,4,5,6,7,8,9,10})` | 25.559768683380703 | 26.354817185647274 |
| 38e4fc309fe199ea A89 | P!A165 | `DB(15779,385,37,D11!$B7,D11!C11)` | "#6" | "#3" |
| 3b7990bd4e3307ba A261 | P!A177 | `NPV(((-1)-D12!$C4),D12!B1:B2,D12!B7:B12,(0.5+D12!B5))` | "#2" | #NUM! |
| 44a8841352dc48e9 A7 | P!A198 | `ACCRINT(34443,36178,38986,0.05/12,3587,1,5,"true")` | "#3" | "#6" |
| 44a8841352dc48e9 A291 | P!A207 | `YIELDDISC(36531,45775,97,RANK.EQ(D14!A9,D14!B9:E11,0),(-1))` | "#6" | "#7" |
| 44a8841352dc48e9 A318 | P!A214 | `YIELD(39681,45662,D14!B$8,3.14159,102,4)` | "#6" | 19.040243161919385 |
| 5919660b7bb4d195 A88 | P!A216 | `POISSON.DIST(((D15!E12+0.2)*(1000000000000000.0*1000000000000000.0)),((D15!D5-D15!D10)+HOU` | 1.0 | "#6" |
| 5cebac1f02bf30f2 A311 | P!A222 | `TTEST(((D16!C6+D16!E$6)/FINV(1.5,3,0.3)),IRR(INDEX(D16!D3:E9,D16!$C7),ROUNDDOWN(1,42)),2,(` | "#3" | "#4" |
| 607e85cfee83a689 A28 | P!A236 | `NPV((-1),D17!B7:B11)` | "#2" | #NUM! |
| 6e18550e9fc9c1b9 A347 | P!A245 | `SKEW({1E+200,1E+200,-1E+200},2.5,D18!D5)` | "#2" | 0.0 |
| 775d053de498b8c7 A98 | P!A266 | `STEYX(ERROR.TYPE(SUMPRODUCT(D20!C1:E5,7)),MATCH((D20!$E$8>=D20!$E7),D20!A11:C11))` | "#6" | "#2" |
| 775d053de498b8c7 A112 | P!A273 | `SLOPE(CELL("row",D20!E3),(0.1-TIME(5,16,59)))` | "#6" | "#2" |
| 775d053de498b8c7 A235 | P!A279 | `(TAKE(D20!C8:E12,D20!B12,D20!$A7)-(D20!A$3/D20!A2))` | "#2" | "64|1x1|#2" |
| 7fd0a98183c6babf A240 | P!A297 | `NPV(D22!$B4,D22!D8,D22!A5:A5,D22!C6:C9)` | "#2" | #NUM! |
| 873f30d2bf76f8db A308 | P!A317 | `F.INV.RT(1-1E-10,8,7)` | 0.0012599666259138292 | 0.0012599666259138298 |
| 92d4fa29e5510f1e A98 | P!A326 | `MODE.SNGL(CELL("contents",D26!C$5))` | "#3" | "#7" |
| 92d4fa29e5510f1e A236 | P!A330 | `IRR(D26!C8:E12,1E-10)` | "#6" | 399999999999998.7 |
| 9bc247733a1ca9fa A5 | P!A341 | `BINOM.DIST(3,1E+15,1E-20,FALSE)` | "#6" | 1.6666500000833278e-16 |
| 9bc247733a1ca9fa A360 | P!A353 | `TRIMMEAN(D28!A6,D28!C2)` | "#6" | "#3" |
| a4090567758520df A101 | P!A396 | `PERCENTRANK.EXC(D31!A9:C9,D31!B12,D31!B$3)` | "#7" | "#6" |
| a5342a7e24fd803e A166 | P!A407 | `WEIBULL(D32!E2,D32!E2,D32!$E11,D32!$A$8)` | "#6" | 1.0 |
| a63d1c7d6f96c56d A92 | P!A417 | `NEGBINOMDIST(7,9,1E-10)` | 6.434999995495541e-87 | 6.434999995495502e-87 |
| a63d1c7d6f96c56d A111 | P!A424 | `IRR(D33!C4:E8,((-0.5)*D33!D4))` | "#6" | -0.9710878190154174 |
| acac71a65a5a1d1b A27 | P!A429 | `NPV(1E+300,(D34!E3+0.5),{-100,-200,-300})` | "#2" | 8e-301 |
| bbab6562e88b70e8 A290 | P!A455 | `PERCENTRANK(D36!C11:E12,(-0.0001),1000000000000000.0)` | "#2" | #NUM! |
| bcb1105c61a2649d A97 | P!A462 | `PMT(D37!C9,346,96301,42,1)` | -278.44797687866065 | -278.4479768786607 |
| c2684066d8df263e A201 | P!A491 | `COUNT(D38!B2:B10,CELL("contents",D38!C8))` | 3.0 | 2.0 |
| cb61169f4c033d4b A272 | P!A517 | `DB(79665,(-100),17,9)` | "#6" | #NUM! |
| d208bbafc3bc1997 A262 | P!A520 | `SKEW(D41!D11:D12,D41!D10)` | 2.4980018054066022e-15 | 2.9976021664879227e-15 |
| d910f000b60e2eac A20 | P!A543 | `PPMT(100,5,217,4143,669,1)` | 0.0 | 4.768374219565885e-5 |
| eccd03df42e40006 A21 | P!A549 | `XMATCH(COUNTA(D45!B8),{3;1;4;1;5},(D45!$C1+D45!E8))` | 2.0 | "#7" |
| eedd629f3d69fdda A297 | P!A563 | `IRR(D47!A6:C8)` | "#6" | -0.8583707295763425 |
| f82473275380882e A131 | P!A567 | `ACCRINTM(32899,36547,D48!C9,2035,3)` | "#3" | "#6" |
| f82473275380882e A143 | P!A572 | `NORMDIST(1E+300,MAX(D48!C11:C12),2.5,0)` | "#6" | 0.0 |

No diverging sub-expression in the probe (31): 097af1ca33638670 A14, 097af1ca33638670 A89, 17c110a53924beb8 A300, 1db18890916ee643 A89, 334f3dc9ca937f10 A125, 3b7990bd4e3307ba A98, 41248d1cb153c66e A394, 6e18550e9fc9c1b9 A69, 73f053591a54bc43 A226, 7980ffb6095a340d A146, 80586b859b73c87d A67, 873f30d2bf76f8db A96, 928d08e3e1eecb5f A333, 941c409532603d32 A148, 9bc247733a1ca9fa A207, 9e2225ead8a6a07b A100, 9e2225ead8a6a07b A292, a2262ea466fcc142 A316, a5342a7e24fd803e A322, b9a32ce9a1fd0d71 A14, b9a32ce9a1fd0d71 A142, b9a32ce9a1fd0d71 A212, bcb1105c61a2649d A230, c2684066d8df263e A65, c4635963fc926284 A12, c4635963fc926284 A92, c4635963fc926284 A153, d37313406b9f6e55 A137, d37313406b9f6e55 A182, d8f44d556674c2fd A398, ede0323c389fc703 A7

Families seen: NPV with a rate of -1 or 1E+300 is `#DIV/0!` (ours `#NUM!` or a value; 6 cells); IRR is `#NUM!` where ours returns a
rate (4; T-1686); argument-domain error order of PV, NPER, PRICEDISC, DB, ACCRINT, ACCRINTM, YIELDDISC, T.TEST, T.INV.2T,
PERCENTRANK(.INC/.EXC), TRIMMEAN, XMATCH (one or two cells each); PERMUTATIONA and COMBINA with a negative argument are 1; CELL("contents")
in COUNT and MODE.SNGL; MODE.MULT, STDEV.P, SKEW, F.INV.RT, NEGBINOMDIST, PMT, PPMT, BINOM.DIST, NORMDIST, WEIBULL(.DIST), BETA.INV
and YIELD values; TAKE of a LET scalar is a scalar (`TAKE(LET(..),-1,-1)`), and TAKE minus a scalar a scalar error.
Owners: T-2618 (NPV, 6 cells), T-2619 (IRR, 4), T-2620 (financial domain errors and digits, 12), T-2621 (statistical domain
errors and extreme values, 23), T-2622 (TAKE, CELL("contents"), XMATCH, a LAMBDA joined as text, 5), T-2623 (33: the 31 cells whose
probed sub-expressions all match Excel and the 2 GETPIVOTDATA cells). Quirks recorded: financial.md "NPV with a rate of -1 or
1E+300 is #DIV/0!", arrays.md "An uncalled LAMBDA is one value" (joined as text), and every other family's per-cell Excel
values in statistics.md "Inner sub-expressions of the T-2256 gen-m9 statistical cells (T-2475)"; the rules behind them are the
follow-ups' to state.

## Outcome
probes/t2475-stat-subexpressions (574 cells, excel-graph) gives 50 of the 83 cells an innermost diverging sub-expression; every cell
now sits in one of the six follow-ups above, which carry `campaign: visible`, so the visible-parity criterion moves with them (the
83 gen-m9 lines stay until those land). Eval delta: the probe adds 121 known-failure lines in eval/baseline/probes.txt, the
divergences the follow-ups own; no other baseline changes.
