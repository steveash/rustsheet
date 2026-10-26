---
id: T-2472
title: "PERCENTILE, QUARTILE, PERCENTRANK, TRIMMEAN and SMALL: the scalar argument's error before the no-number result, PERCENTRANK's #N/A before a bad significance (gen-m9, 62 cells)"
milestone: M9
lane: functions-stat
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
PERCENTILE*, QUARTILE*, PERCENTRANK*, TRIMMEAN and SMALL return the k/quart/x/percent argument's error before the result
of data with no number, and PERCENTRANK* checks x against the data (`#N/A`) before significance (`#NUM!`).

## Context
Gap triage of the T-2256 gen-m9 sweep. statistics.md "Order statistics: the scalar argument's error before the no-number
result; PERCENTRANK's #N/A before significance (T-2256)" records both rules. 48 cells are a scalar argument's error beside
data with no number (the engine gives the data's `#NUM!` or `#N/A`), 14 a PERCENTRANK x outside the data or data with no
number beside a significance below 1. PERCENTILE.EXC already reports k's error first (T-1403); extend the same order to the
other readers. A lone text/boolean cell as the data is T-2344's.
Code: crates/rustsheet-engine/src/functions/statistical/percentile.rs, quartile.rs, trimmean.rs and sorted_data.rs.

Owned gen-m9 cells (T-2256 triage, excel-graph, 2026-10-23): 62 known-failure lines in eval/baseline/gen-m9.txt, 62 visibly off.
Examples:
- gen-m9/097af1ca33638670 Calc!A285 `=QUARTILE(Data!E7:E7,((Data!E4-0.3)-3.14159))`: Excel #VALUE!, ours #NUM!
- gen-m9/41248d1cb153c66e Calc!A112 `=QUARTILE(Data!C10,IFNA(1/0,ISERR(42)))`: Excel #DIV/0!, ours #NUM!
- gen-m9/9e2225ead8a6a07b Calc!A165 `=PERCENTRANK.EXC(Data!D3:D4,DAY((42/1e-15)))`: Excel #NUM!, ours #N/A
- gen-m9/a63d1c7d6f96c56d Calc!A72 `=TRIMMEAN(Data!C7:C7,((Data!A$6-Data!A$11)/0))`: Excel #DIV/0!, ours #NUM!
- gen-m9/c4635963fc926284 Calc!A103 `=PERCENTRANK.INC(Data!Z1:Z3,STDEVA(Data!A1))`: Excel #DIV/0!, ours #N/A
Cells:
- gen-m9/097af1ca33638670.case.json (3): Calc!A126, A285, A312
- gen-m9/09b47629f54b9817.case.json (2): Calc!A58, A210
- gen-m9/139684e68fb2fc3b.case.json (1): Calc!A324
- gen-m9/17c110a53924beb8.case.json (1): Calc!A33
- gen-m9/201d67f1d59436ed.case.json (2): Calc!A152, A400
- gen-m9/334f3dc9ca937f10.case.json (1): Calc!A27
- gen-m9/38e4fc309fe199ea.case.json (2): Calc!A272, A341
- gen-m9/3b7990bd4e3307ba.case.json (1): Calc!A131
- gen-m9/3fb9cecf2c23169f.case.json (1): Calc!A132
- gen-m9/41248d1cb153c66e.case.json (2): Calc!A112, A153
- gen-m9/44a8841352dc48e9.case.json (5): Calc!A10, A51, A238, A276, A390
- gen-m9/5389dd4d5b5739d2.case.json (2): Calc!A233, A351
- gen-m9/5395b8a427764a3f.case.json (1): Calc!A55
- gen-m9/583ca2baea0bb30a.case.json (1): Calc!A305
- gen-m9/5cebac1f02bf30f2.case.json (1): Calc!A118
- gen-m9/775d053de498b8c7.case.json (2): Calc!A275, A337
- gen-m9/787ba8d801105a60.case.json (1): Calc!A336
- gen-m9/7fd0a98183c6babf.case.json (2): Calc!A121, A261
- gen-m9/873f30d2bf76f8db.case.json (3): Calc!A55, A131, A204
- gen-m9/91e167cd8d7eb652.case.json (1): Calc!A186
- gen-m9/928d08e3e1eecb5f.case.json (1): Calc!A318
- gen-m9/92d4fa29e5510f1e.case.json (1): Calc!A325
- gen-m9/9735c5ab72fa5ecf.case.json (2): Calc!A175, A236
- gen-m9/9bc247733a1ca9fa.case.json (3): Calc!A145, A184, A308
- gen-m9/9e2225ead8a6a07b.case.json (2): Calc!A165, A294
- gen-m9/a5342a7e24fd803e.case.json (1): Calc!A238
- gen-m9/a63d1c7d6f96c56d.case.json (1): Calc!A72
- gen-m9/b9a32ce9a1fd0d71.case.json (1): Calc!A197
- gen-m9/c4635963fc926284.case.json (3): Calc!A103, A219, A295
- gen-m9/c7664e494e418395.case.json (3): Calc!A76, A161, A176
- gen-m9/cb61169f4c033d4b.case.json (1): Calc!A308
- gen-m9/d208bbafc3bc1997.case.json (3): Calc!A67, A105, A163
- gen-m9/d8f44d556674c2fd.case.json (2): Calc!A7, A136
- gen-m9/eccd03df42e40006.case.json (2): Calc!A252, A314
- gen-m9/ede0323c389fc703.case.json (1): Calc!A30

T-2436 probe (excel-graph, 2026-10-24; probes/t2436/<case>, each gen-m9 cell's sub-expressions on its Data sheet): `TRIMMEAN(<"abc",FALSE,"a">,SUMX2PY2(..)<#N/A>)` is `#N/A`, percent's error before the no-number `#NUM!` (ours `#NUM!`); `PERCENTRANK.INC(<blank>,<"TRUE" text>,4)` is `#VALUE!`, x's text before the empty data (ours `#N/A`).
Owned gen-m9 cells (T-2436 probe, excel-graph, 2026-10-24): 2 known-failure lines in eval/baseline/gen-m9.txt, 2 formulas.
Examples:
- gen-m9/3e98b26cf6713e77 Calc!A247 `=CRITBINOM(TRIMMEAN(Data!A9:C9,SUMX2PY2(Data!B7,Data!D9:E12)),FLOOR.PRECISE((0.3/Data!D10),Data!E9),1E-200)`: Excel #N/A, ours #NUM!; inner `TRIMMEAN(Data!A9:C9,SUMX2PY2(Data!B7,Data!D9:E12))` Excel #N/A, ours #NUM!
- gen-m9/c2974c061384fdd9 Calc!A90 `=LOGNORM.INV(Data!A5,GAMMAINV(Data!B1,PERCENTRANK.INC(Data!A7,Data!D12,4),Data!B10),99999)`: Excel #VALUE!, ours #N/A; inner `PERCENTRANK.INC(Data!A7,Data!D12,4)` Excel #VALUE!, ours #N/A
Cells:
- gen-m9/3e98b26cf6713e77.case.json (1): Calc!A247
- gen-m9/c2974c061384fdd9.case.json (1): Calc!A90

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2436): these 2 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity (gen-m9, T-2256): these 62 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Unit tests: `QUARTILE(Z1,1/0)` with Z1 empty is `#DIV/0!`, `PERCENTILE.INC(A1:A2,"x")` with text data `#VALUE!`,
      `TRIMMEAN(A1:A1,NA())` `#N/A`, `PERCENTRANK(A1:A5,1963,-2.5)` with x above the data `#N/A`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- PERCENTRANK's huge significance and truncation digit (T-2473).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2256): every cell it owns has the wrong type, error or text (none is a last-digit difference), so the task is in the campaign. Its target is that every cell it owns
has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value; the visible-parity
criterion under Acceptance criteria states it.
