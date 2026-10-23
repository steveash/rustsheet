---
id: T-2428
title: "XIRR and XNPV size and type checks, and financial domain-before-text order, under the gen-m9 statistical cells (79 cells)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
XIRR and XNPV check their arguments as Excel does: XIRR with values and dates of different counts, or a single value, is
`#NUM!` (not `#VALUE!`), an error guess is that error; XNPV with a boolean or text one-cell rate or dates is `#VALUE!` and a
number against a range of a different count `#NUM!`. The other financial functions under these cells (ODDFYIELD, ODDLYIELD,
ODDLPRICE, DURATION, PRICEMAT, PRICEDISC, RECEIVED, TBILLYIELD, TBILLPRICE, AMORLINC, AMORDEGRC, ACCRINT, DB, VDB, PV,
EFFECT, FVSCHEDULE, NPV, IRR, RATE) give Excel's error where the enclosing statistical call shows a different one.

## Context
Gap triage of the T-2255 gen-m9 sweep: in these 79 cells the innermost diverging call is a financial function inside a
statistical one (traced by evaluating every sub-expression in the engine; the enclosing function only passes the error on,
so the cell's value is the financial call's). financial.md "XIRR and XNPV under the gen-m9 statistical cells: size and
type checks (T-2255)" records the XIRR/XNPV rule (51 cells: XIRR 40, XNPV 11). The other 28 are single cells whose
Excel error is inferred the same way and mostly show a domain `#NUM!` beating a text or boolean `#VALUE!`
(`ODDFYIELD(36349,-0.5,..,FALSE,4)` `#NUM!`, `PRICEMAT(..,TRUE,3)` `#NUM!`, `DB(-1E-6,286,11,1,FALSE)` `#NUM!`) or
the reverse (`EFFECT(0,9)`, `ACCRINT(..,"TRUE")`, `AMORDEGRC(..,TRUE,1)`, `ODDLYIELD(..,BINOM.DIST(..))` `#VALUE!`), plus
`PV(-1+2^-52,30,-750)` `#DIV/0!`, `TBILLPRICE(0.5,100,1E-15)` `#DIV/0!`, `FVSCHEDULE(3669,<empty cell>)` 3669 and
`IRR({-100,#N/A,60})` `#VALUE!`; confirm each with a probe of the bare call before changing its order (excel-probe skill).
T-2257 (the financial sweep) will add direct evidence; coordinate with it.
Code: `fn_xirr`, `fn_xnpv` in crates/rustsheet-engine/src/functions/financial/mod.rs and the other functions' readers.

Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 79 known-failure lines in eval/baseline/gen-m9.txt, 79 formulas.
Examples:
- gen-m9/0a614cfe974b59c3 Calc!A390 `=F.INV(XIRR(Data!D11:D12,Data!C6:C9,(-1)),1,1)`: Excel #NUM!, ours #VALUE!
- gen-m9/0a9226016e8f5f33 Calc!A373 `=F.INV(XIRR({0,0,-100,50,60},Data!A8:D11,((-0.0001)/Data!B12)),10,3)`: Excel #DIV/0!, ours #VALUE!
- gen-m9/165db58d1540f014 Calc!A361 `=LOGINV(ERROR.TYPE(VALUE("x")),(123456789.123-AVERAGE(Data!E2:E11)),XNPV(1E-300,Data!D4:E12,Data!E1))`: Excel #VALUE!, ours #NUM!
- gen-m9/285d87cea1aaa73c Calc!A28 `=GROWTH((UNIQUE(Data!D3:E9,FALSE,Data!B$12)+DURATION(45412,44998,Data!$D$8,(-0.5),"2")),Data!B5:E6,Data!C10:C12,LOOKUP(HYPERLINK("#Data!A1",Data!C2),Data!A5:A12,Data!C8:C11))`: Excel #REF!, ours #NUM!
- gen-m9/40425341c5034eed Calc!A94 `=CRITBINOM(9,IRR({-100,#N/A,60}),Data!D8)`: Excel #VALUE!, ours #N/A
- gen-m9/530c636749a85209 Calc!A199 `=FISHERINV((IFS(Data!$B9,"1.5",FALSE,Data!E2,Data!D$4,Data!B11)-XIRR((-2.5),Data!C1,0.05/12)))`: Excel #N/A, ours #VALUE!
Cells:
- gen-m9/0a614cfe974b59c3.case.json (1): Calc!A390
- gen-m9/0a9226016e8f5f33.case.json (3): Calc!A139, A231, A373
- gen-m9/146b46e74f2dac3b.case.json (1): Calc!A215
- gen-m9/165db58d1540f014.case.json (1): Calc!A361
- gen-m9/2801c1c11c21e134.case.json (2): Calc!A71, A110
- gen-m9/285d87cea1aaa73c.case.json (1): Calc!A28
- gen-m9/320d51765af9b6b7.case.json (3): Calc!A85, A246, A346
- gen-m9/3a25a03eb94fcfbd.case.json (1): Calc!A305
- gen-m9/3e98b26cf6713e77.case.json (1): Calc!A295
- gen-m9/40425341c5034eed.case.json (5): Calc!A18, A94, A216, A240, A322
- gen-m9/4250f880ab640c49.case.json (1): Calc!A91
- gen-m9/4ab2ba98d26c8a5e.case.json (1): Calc!A155
- gen-m9/4bad303d32cc3f46.case.json (1): Calc!A364
- gen-m9/530c636749a85209.case.json (2): Calc!A61, A199
- gen-m9/589f42e67f4b4b8d.case.json (1): Calc!A32
- gen-m9/58e3253bbd3a7a82.case.json (3): Calc!A97, A189, A326
- gen-m9/60b0b6e69ad35a45.case.json (2): Calc!A94, A142
- gen-m9/7674c89c824aca43.case.json (1): Calc!A281
- gen-m9/7c20351562a2d71d.case.json (1): Calc!A24
- gen-m9/85f48d5cdfb24555.case.json (1): Calc!A41
- gen-m9/899f9555e67da2d6.case.json (2): Calc!A313, A390
- gen-m9/8f2ac25945af87da.case.json (1): Calc!A137
- gen-m9/8fec7b207f4dd753.case.json (2): Calc!A8, A356
- gen-m9/984cba67bbe6972b.case.json (2): Calc!A141, A323
- gen-m9/9cca17438a7e6f8a.case.json (2): Calc!A236, A262
- gen-m9/a013a45a8fbcab4f.case.json (2): Calc!A100, A186
- gen-m9/a54c738c697617c3.case.json (1): Calc!A317
- gen-m9/a5828d3925575713.case.json (1): Calc!A210
- gen-m9/a612e89321a6df9c.case.json (1): Calc!A219
- gen-m9/a61e547a1cab994c.case.json (3): Calc!A113, A246, A353
- gen-m9/a790a4d4a6a82756.case.json (2): Calc!A23, A379
- gen-m9/abc2939ebd1e2365.case.json (2): Calc!A133, A327
- gen-m9/b4173fe25109a238.case.json (1): Calc!A337
- gen-m9/b8ee55934c5b824c.case.json (1): Calc!A122
- gen-m9/bbdfc06cae0e0d71.case.json (1): Calc!A44
- gen-m9/bc40d37cb4d8301d.case.json (4): Calc!A63, A79, A213, A392
- gen-m9/bcb24385baafb2f0.case.json (2): Calc!A224, A373
- gen-m9/c18dbf21f11093f3.case.json (1): Calc!A188
- gen-m9/c2974c061384fdd9.case.json (2): Calc!A28, A283
- gen-m9/caa1eed48b1883fe.case.json (1): Calc!A206
- gen-m9/cde551ea787f713a.case.json (1): Calc!A37
- gen-m9/d8266bc15533702a.case.json (1): Calc!A83
- gen-m9/e6cf3b20a014e6a3.case.json (1): Calc!A122
- gen-m9/ecc87f80077959e0.case.json (2): Calc!A29, A91
- gen-m9/ee2497a5e683c5d9.case.json (2): Calc!A102, A107
- gen-m9/f2bd5f380ec002bf.case.json (1): Calc!A54
- gen-m9/f34f66fb7f81e673.case.json (1): Calc!A88
- gen-m9/fd9fa22d212080d6.case.json (1): Calc!A78
- gen-m9/fec39efaba9284c3.case.json (2): Calc!A216, A285

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2255): these 79 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Unit tests for the XIRR/XNPV shapes above (different counts, one value, an error guess, a boolean dates cell).
- [ ] financial.md's entry gets an "Ours:" line naming this task; each of the 28 other cells is fixed or listed in the
      Outcome with its probe result.
- [ ] `make check` green; reviewers approve.

## Out of scope
- XIRR/XNPV values (their solver tasks).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2255): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.
