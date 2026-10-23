---
id: T-2436
title: "Probe the inner sub-expressions of 116 gen-m9 statistical cells whose divergence is not traced"
milestone: M9
lane: research
status: todo
priority: P2
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
Each of these 116 gen-m9 cells has its innermost diverging sub-expression identified from Excel values (a probe case that
evaluates the sub-expressions with the same Data sheet), and is then owned by a fix task (new or existing), with the quirks
recorded in docs/excel-semantics/.

## Context
Gap triage of the T-2255 gen-m9 sweep. The triage traced 4,175 of the 4,291 visibly-off cells to a family by evaluating
every sub-expression in the engine; these 116 are left: their Excel value is an error none of the call's arguments has in
ours, or a value that needs an inner function to differ, and the engine alone cannot tell which inner call Excel computes
differently. Most involve an inner function outside the statistical lane (DATEDIF, INDEX, TEXTSPLIT, MAKEARRAY, PROB,
ZTEST/Z.TEST, QUARTILE.INC, STANDARDIZE, WEIBULL, BESSELI/BESSELJ, PHI, GROUPBY/PIVOTBY, TRIMRANGE, CONVERT, HEX2OCT,
SUMX2MY2 and others) or an argument error order in the outer statistical function (`GAMMA.INV(1.5,FORECAST.LINEAR(..)<#N/A>,
NETWORKDAYS(..)<#VALUE!>)` is `#VALUE!`; `LOGNORM.DIST(..,MAP(..)<#NUM!>,Data!C2,CHOOSECOLS(..)<#VALUE!>)` `#VALUE!`), and
three F.TEST cells statistics.md marks UNVERIFIED (a number against a range `#NUM!`). Method: copy each case's Data sheet
into a probe case (excel-probe skill), one formula per sub-expression of each cell, fill it through excel-graph, compare
with the engine, then file or extend the owning task for each family found (decision 0083: `campaign: visible`).

Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 116 known-failure lines in eval/baseline/gen-m9.txt, 116 formulas.
Examples:
- gen-m9/05d44f6dd6cfc567 Calc!A196 `=COVAR(RANK(HSTACK(Data!A5),Data!A11:D12),5)`: Excel #N/A, ours #VALUE!
- gen-m9/05d44f6dd6cfc567 Calc!A392 `=GAMMA.DIST(DEGREES(FINV(1E-20,1,6)),-VLOOKUP(Data!E11,Data!B11:E12,Data!C6,Data!C6),3,OR(Data!D4:D11))`: Excel #VALUE!, ours #NUM!
- gen-m9/06b4e8f5de409d5e Calc!A261 `=F.TEST(INDIRECT("Data!B3",TOCOL(2,0,Data!A5)),{-1,0,1})`: Excel #NUM!, ours #DIV/0!
- gen-m9/0a9226016e8f5f33 Calc!A23 `=GAMMADIST(QUARTILE.EXC(Data!A8,DATE(0,1,21)),3,0.5,(Data!$E12<>(Data!$A8+Data!A2)))`: Excel #NUM!, ours #VALUE!
- gen-m9/0a9226016e8f5f33 Calc!A26 `=CHIDIST(TIME(11,HYPGEOMDIST(2,5,9,11),1),9)`: Excel 0.9999790730352063, ours #NUM!
- gen-m9/165db58d1540f014 Calc!A105 `=CONFIDENCE((POISSON(0,(-0.5),Data!E$6)*CHITEST(Data!E1,(-2.5))),VAR(Data!D5:D11,Data!B8:B9),6)`: Excel #N/A, ours #NUM!
Cells:
- gen-m9/05d44f6dd6cfc567.case.json (2): Calc!A196, A392
- gen-m9/06b4e8f5de409d5e.case.json (2): Calc!A261, A366
- gen-m9/0a614cfe974b59c3.case.json (1): Calc!A356
- gen-m9/0a9226016e8f5f33.case.json (2): Calc!A23, A26
- gen-m9/165db58d1540f014.case.json (2): Calc!A45, A105
- gen-m9/2801c1c11c21e134.case.json (2): Calc!A24, A132
- gen-m9/285d87cea1aaa73c.case.json (1): Calc!A154
- gen-m9/2a18b2f5a4dc43ac.case.json (2): Calc!A79, A236
- gen-m9/320d51765af9b6b7.case.json (1): Calc!A329
- gen-m9/35e8168aae2b00f8.case.json (5): Calc!A48, A108, A162, A271, A385
- gen-m9/3a25a03eb94fcfbd.case.json (1): Calc!A179
- gen-m9/3e98b26cf6713e77.case.json (2): Calc!A247, A383
- gen-m9/40425341c5034eed.case.json (1): Calc!A206
- gen-m9/4250f880ab640c49.case.json (3): Calc!A237, A295, A325
- gen-m9/4ab2ba98d26c8a5e.case.json (4): Calc!A23, A116, A341, A395
- gen-m9/530c636749a85209.case.json (2): Calc!A371, A379
- gen-m9/589f42e67f4b4b8d.case.json (2): Calc!A164, A185
- gen-m9/58e3253bbd3a7a82.case.json (3): Calc!A46, A143, A350
- gen-m9/60b0b6e69ad35a45.case.json (1): Calc!A69
- gen-m9/623663cacea53bbc.case.json (3): Calc!A256, A353, A400
- gen-m9/68d8ffb69ff739d8.case.json (2): Calc!A285, A290
- gen-m9/7674c89c824aca43.case.json (1): Calc!A305
- gen-m9/7c20351562a2d71d.case.json (4): Calc!A194, A276, A277, A381
- gen-m9/899f9555e67da2d6.case.json (3): Calc!A113, A311, A369
- gen-m9/8d3588d8df0812df.case.json (3): Calc!A236, A240, A313
- gen-m9/8dd2312c96effb1b.case.json (4): Calc!A131, A228, A381, A399
- gen-m9/984cba67bbe6972b.case.json (2): Calc!A160, A277
- gen-m9/9cca17438a7e6f8a.case.json (1): Calc!A171
- gen-m9/a54c738c697617c3.case.json (2): Calc!A244, A245
- gen-m9/a612e89321a6df9c.case.json (5): Calc!A22, A23, A241, A375, A390
- gen-m9/a790a4d4a6a82756.case.json (2): Calc!A308, A380
- gen-m9/a7bf0d8d81d33ba6.case.json (1): Calc!A99
- gen-m9/abc2939ebd1e2365.case.json (5): Calc!A11, A44, A246, A250, A280
- gen-m9/b4173fe25109a238.case.json (2): Calc!A78, A313
- gen-m9/b8ee55934c5b824c.case.json (1): Calc!A394
- gen-m9/bbdfc06cae0e0d71.case.json (2): Calc!A253, A322
- gen-m9/bc40d37cb4d8301d.case.json (1): Calc!A115
- gen-m9/bcb24385baafb2f0.case.json (1): Calc!A252
- gen-m9/c18dbf21f11093f3.case.json (1): Calc!A174
- gen-m9/c2974c061384fdd9.case.json (4): Calc!A90, A96, A149, A157
- gen-m9/c468f5046aadfbbc.case.json (2): Calc!A162, A378
- gen-m9/c6cfda6bd10cae93.case.json (1): Calc!A375
- gen-m9/caa1eed48b1883fe.case.json (2): Calc!A307, A399
- gen-m9/d8266bc15533702a.case.json (3): Calc!A14, A99, A316
- gen-m9/e6cf3b20a014e6a3.case.json (1): Calc!A229
- gen-m9/ecc87f80077959e0.case.json (2): Calc!A187, A234
- gen-m9/eced4bb6a0234c9e.case.json (2): Calc!A34, A116
- gen-m9/ee2497a5e683c5d9.case.json (2): Calc!A73, A128
- gen-m9/f2bd5f380ec002bf.case.json (3): Calc!A254, A383, A398
- gen-m9/f34f66fb7f81e673.case.json (4): Calc!A106, A127, A140, A283
- gen-m9/fd9fa22d212080d6.case.json (1): Calc!A131
- gen-m9/fec39efaba9284c3.case.json (4): Calc!A37, A168, A242, A253

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2255): these 116 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Every cell above is listed in a fix task's Context (new or extended), or explained in statistics.md as a quirk
      with its owning task; the probe case's cells are in eval/ and gated or baselined.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing the functions (the tasks this one files).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2255): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.
