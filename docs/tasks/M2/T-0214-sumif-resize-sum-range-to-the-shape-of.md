---
id: T-0214
title: "SUMIF: resize sum_range to the shape of range from its top-left cell (59 gen-m2 known failures)"
milestone: M2
lane: functions-criteria
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
SUMIF(range, criteria, sum_range) sums over a sum_range **resized to the shape of `range`**, anchored at
sum_range's top-left cell. Today the engine walks sum_range's own shape.

## Context
gen-m2 (T-0203): 59 failing cells (NumericMismatch). Evidence (excel-graph):
- `gen-m2/04cfd82f54dbc2b2` Calc!A344 `=SUMIF(Data!C3:C5,"?b?",Data!C10:C10)`: expected 0.5 (C10:C12 is used), actual 0.
- `gen-m2/629e6e8b76fab8b9` Calc!A217 `=SUMIF(Data!C2:E2,"<>1",Data!B4:B10)`: expected 123456831.123 (B4:D4), actual 81.5.
- `gen-m2/f0d1bacec5de8146` Calc!A127 `=SUMIF(Data!C9:D10,">=2",Data!D4:D7)`: expected 42.5, actual 0.
- Of 72 top-level 3-argument SUMIF calls in gen-m2, all 71 with mismatched shapes return a number
  (never `#VALUE!`), unlike SUMIFS (T-0209).

Code: `crates/rustsheet-engine/src/functions/statistical/if_ifs.rs::sumif` (delegates to `fn_sumifs`
with sum_range first, so the walk follows sum_range). AVERAGEIF is documented by Microsoft to resize the
same way. Probe it with the excel-probe skill and include it only if the evidence agrees.
Probed by T-0209 (excel-graph, 2026-10-05): AVERAGEIF resizes the same way and never errors:
probes/ifs-shape-mismatch F9 `=AVERAGEIF(A1:B2,"<99",A1:D1)` = 3.5, F10 `=AVERAGEIF(A1:D1,"<99",A1)` = 2.5,
F11 `=AVERAGEIF(A1:B2,"<99",C1:C4)` = 5.5 (known failures in eval/baseline/probes.txt today).

Cells whose only gen-m2 root cause is this one (59):
- `gen-m2/04cfd82f54dbc2b2.case.json`: A106 A344
- `gen-m2/1b6ceed7489e0800.case.json`: A53
- `gen-m2/207a5753da48c7e0.case.json`: A237 A267 A349
- `gen-m2/2cfe279fe77a000d.case.json`: A57
- `gen-m2/60a543744fc4a8f7.case.json`: A29
- `gen-m2/61cacc22c4244179.case.json`: A24 A97 A382
- `gen-m2/629e6e8b76fab8b9.case.json`: A49 A214 A217 A325
- `gen-m2/7469c3e920e1ade4.case.json`: A163
- `gen-m2/8eb18149d4245e1a.case.json`: A118 A169 A177 A236
- `gen-m2/96a0952412cefa6c.case.json`: A152 A288
- `gen-m2/a339ea84ef64382e.case.json`: A260 A386
- `gen-m2/a6d4377c7b1b92cd.case.json`: A105 A204
- `gen-m2/b4392a6514a36ade.case.json`: A117 A164
- `gen-m2/caa981b0753f455c.case.json`: A216 A256
- `gen-m2/cbdabe32a32a4012.case.json`: A16
- `gen-m2/ce315bb699e1be6f.case.json`: A334
- `gen-m2/d04d501154afc771.case.json`: A94 A147 A380
- `gen-m2/d3e2df0bef2f173a.case.json`: A352 A357
- `gen-m2/dd99be1f2454159f.case.json`: A15 A48 A160 A250 A341
- `gen-m2/e8493fafa2a5378e.case.json`: A48 A84 A211 A279
- `gen-m2/ef5c55a81783b7d5.case.json`: A20 A106 A163
- `gen-m2/f0d1bacec5de8146.case.json`: A85 A127 A253
- `gen-m2/f7e6d351c8538ef5.case.json`: A22 A283
- `gen-m2/f9d4df4efdfbcbd5.case.json`: A215 A378
- `gen-m2/fa437879253827be.case.json`: A239 A317 A333

Also needs T-0209:
- `gen-m2/893302d6c5b99e2e.case.json` A371 (also: ifs_shape)

## Acceptance criteria
- [ ] If AVERAGEIF is resized too: `test_averageif_shape_mismatch_is_not_an_error` (test_fn_sumifs.rs) asserts 3.5, and probes/ifs-shape-mismatch F9–F11 leave eval/baseline/probes.txt.
- [ ] The 59 cells above disappear from eval/baseline/gen-m2.txt with no new F lines.
- [ ] Unit tests: taller, wider and smaller sum_range all resize from the top-left; the array-criteria
      path in `sumif` resizes too.
- [ ] docs/excel-semantics/criteria.md has the rule with the evidence cells above.
- [ ] `make check` green; reviewers approve.

## Out of scope
- SUMIFS/COUNTIFS shape errors (T-0209).
