---
id: T-2344
title: "Probe a single cell, name or INDEX result holding a boolean or text as the data of QUARTILE/PERCENTILE/PERCENTRANK"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
The data argument of QUARTILE*, PERCENTILE* and PERCENTRANK* has Excel-evidenced readings when it is a single cell, a defined name or an INDEX/OFFSET result holding a boolean or text.

## Context
T-1748 made a typed boolean or text a one-element array in `percentile_numbers` (percentile.rs) only when the node is not a `ReferenceKind`. Unprobed: `QUARTILE(B1,1)` with B1 = TRUE, "1", "abc"; a name pointing at such a cell; `QUARTILE(INDEX(C1:C3,2),1)`; `QUARTILE("abc",1)` (unit test only). SMALL/LARGE treat a cell holding a scalar as #NUM! (chisq.rs `numbers_from_ranked_arg`).

T-2256 sweep (gen-m9 part 2, statistical M to Z; gap triage 2026-10-23): gen-m9 evidence answers the single-cell question: statistics.md "A lone text, boolean or empty cell as the data of QUARTILE, PERCENTILE, PERCENTRANK and TRIMMEAN (T-2256)": a single-cell reference holding text or a boolean is data with no number, `#NUM!` for QUARTILE*, PERCENTILE* and TRIMMEAN (also for an empty cell) and `#N/A` for PERCENTRANK*; ours `#VALUE!`. The name and INDEX forms stay to probe. This also settles T-2066's QUARTILE cell.

Owned gen-m9 cells (T-2256 triage, excel-graph, 2026-10-23): 72 known-failure lines in eval/baseline/gen-m9.txt, 72 visibly off.
Examples:
- gen-m9/139684e68fb2fc3b Calc!A360 `=QUARTILE.INC(Data!C5,2)`: Excel #NUM!, ours #VALUE!
- gen-m9/1db18890916ee643 Calc!A347 `=QUARTILE.EXC(Data!E8,2)`: Excel #NUM!, ours #VALUE!
- gen-m9/46ac3ef5e61583ca Calc!A134 `=PERCENTILE(Data!C5,0.5)`: Excel #NUM!, ours #VALUE!
- gen-m9/5cebac1f02bf30f2 Calc!A247 `=PERCENTILE(Data!D5,0)`: Excel #NUM!, ours #VALUE!
- gen-m9/7980ffb6095a340d Calc!A132 `=QUARTILE(Data!E2,3)`: Excel #NUM!, ours #VALUE!
Cells:
- gen-m9/0620505280d2ea6e.case.json (1): Calc!A396
- gen-m9/11eb3ab4f4f8104c.case.json (2): Calc!A16, A172
- gen-m9/139684e68fb2fc3b.case.json (3): Calc!A33, A71, A360
- gen-m9/17c110a53924beb8.case.json (2): Calc!A42, A234
- gen-m9/1b31092a0d79e5f1.case.json (1): Calc!A287
- gen-m9/1db18890916ee643.case.json (1): Calc!A347
- gen-m9/201d67f1d59436ed.case.json (2): Calc!A11, A329
- gen-m9/2138b909d8f810ea.case.json (4): Calc!A6, A67, A228, A270
- gen-m9/329b177b5d5165ad.case.json (1): Calc!A281
- gen-m9/38e4fc309fe199ea.case.json (1): Calc!A7
- gen-m9/3fb9cecf2c23169f.case.json (2): Calc!A92, A202
- gen-m9/43c6227e0ff368c7.case.json (1): Calc!A261
- gen-m9/44a8841352dc48e9.case.json (2): Calc!A52, A360
- gen-m9/46ac3ef5e61583ca.case.json (3): Calc!A134, A159, A362
- gen-m9/583ca2baea0bb30a.case.json (1): Calc!A289
- gen-m9/5919660b7bb4d195.case.json (1): Calc!A264
- gen-m9/5be29fa316931f05.case.json (1): Calc!A356
- gen-m9/5cebac1f02bf30f2.case.json (2): Calc!A91, A247
- gen-m9/607e85cfee83a689.case.json (1): Calc!A303
- gen-m9/6e18550e9fc9c1b9.case.json (1): Calc!A318
- gen-m9/775d053de498b8c7.case.json (4): Calc!A49, A148, A227, A334
- gen-m9/787ba8d801105a60.case.json (1): Calc!A355
- gen-m9/7980ffb6095a340d.case.json (2): Calc!A132, A389
- gen-m9/7f7ca726c3f68554.case.json (3): Calc!A54, A168, A337
- gen-m9/80586b859b73c87d.case.json (1): Calc!A312
- gen-m9/873f30d2bf76f8db.case.json (1): Calc!A253
- gen-m9/91e167cd8d7eb652.case.json (1): Calc!A346
- gen-m9/928d08e3e1eecb5f.case.json (1): Calc!A16
- gen-m9/92d4fa29e5510f1e.case.json (3): Calc!A170, A207, A293
- gen-m9/941c409532603d32.case.json (1): Calc!A82
- gen-m9/9735c5ab72fa5ecf.case.json (2): Calc!A188, A264
- gen-m9/9bc247733a1ca9fa.case.json (2): Calc!A218, A369
- gen-m9/9e2225ead8a6a07b.case.json (1): Calc!A233
- gen-m9/a2262ea466fcc142.case.json (1): Calc!A220
- gen-m9/b9a32ce9a1fd0d71.case.json (1): Calc!A323
- gen-m9/bcb1105c61a2649d.case.json (1): Calc!A46
- gen-m9/c4635963fc926284.case.json (2): Calc!A303, A369
- gen-m9/cb61169f4c033d4b.case.json (1): Calc!A104
- gen-m9/ce312817dc9c7f0b.case.json (1): Calc!A25
- gen-m9/d208bbafc3bc1997.case.json (1): Calc!A371
- gen-m9/d37313406b9f6e55.case.json (1): Calc!A372
- gen-m9/d8f44d556674c2fd.case.json (3): Calc!A48, A86, A381
- gen-m9/d910f000b60e2eac.case.json (1): Calc!A143
- gen-m9/eccd03df42e40006.case.json (2): Calc!A123, A201
- gen-m9/f82473275380882e.case.json (1): Calc!A278

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2256): these 72 known-failure lines (the T-2256 cells in Context) disappear from
      eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] A probes-numeric case with an oracle sidecar covers those forms; the engine matches, and the hand-written expectations in test_sorted_data_lift.rs (`typed_scalar_data_...`) cite it.
- [ ] statistics.md drops UNVERIFIED for the single-cell reading.
- [ ] `make check` green; reviewers approve.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2256): it now owns 72 visibly-off gen-m9 cells (Context), so the task is in the
campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th
significant digit of Excel's value; the visible-parity criterion under Acceptance criteria states it. Bit-level criteria
still unmet then move to a follow-up task without the key.
