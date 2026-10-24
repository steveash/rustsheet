---
id: T-2475
title: "Probe the inner sub-expressions of 83 gen-m9 statistical cells of the T-2256 sweep whose divergence is not traced"
milestone: M9
lane: research
status: in-progress
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
- [ ] Visible parity (gen-m9, T-2256): these 83 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] A probe case per source case (or one combined case) holds the sub-expressions, filled from excel-graph and committed.
- [ ] Every cell above is moved to a fix task (new or extended) whose Context lists it; each quirk found is in docs/excel-semantics/.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing the inner functions (their own tasks).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2256): every cell it owns has the wrong type, error or value (none is a last-digit difference), so the task is in the campaign. Its target is that every cell it owns
has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value; the visible-parity
criterion under Acceptance criteria states it.
