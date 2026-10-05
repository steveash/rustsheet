---
id: T-0213
title: "INDEX: one index into a 2-D range is #REF! (164 gen-m2 known failures)"
milestone: M2
lane: functions-lookup
status: in-progress
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
`INDEX(array, n)` with a single index into a range that has more than one row **and** more than one
column returns `#REF!` (also for n = 0 and fractional n). Errors in the index argument still win.
Today the engine returns a cell value or spills the row/column.

## Context
gen-m2 (T-0203): 164 failing cells, under INDEX, IF, SUM, `&` and other callers. Evidence (excel-graph):
- `gen-m2/04cfd82f54dbc2b2` Calc!A342 `=INDEX(Data!B6:D9,1)`: expected `#REF!`, actual 0.
- `gen-m2/60a543744fc4a8f7` Calc!A106 `=INDEX(Data!A1:B2,0)`: expected `#REF!`, actual `#SPILL!`.
- `gen-m2/893302d6c5b99e2e` Calc!A265 `=INDEX(Data!A1:D8,1e-15)`: expected `#REF!`, actual `#SPILL!`.
- Of 90 top-level 2-arg INDEX calls on 2-D ranges, 85 are `#REF!`. The rest are argument errors
  (`#DIV/0!` `gen-m2/d0c29dd5c530e28c` Calc!A27, `#N/A`, `#VALUE!`).
- One-row and one-column ranges keep today's behavior: the single index selects along the vector.
  0 spills (`#SPILL!` when blocked; read correctly since T-0208), and out of range is `#REF!`.

Code: `crates/rustsheet-engine/src/functions/lookup_and_reference/mod.rs` (`fn_index` and the
index-resolution helper above it).

Cells whose only gen-m2 root cause is this one (164):
- `gen-m2/04cfd82f54dbc2b2.case.json`: A342
- `gen-m2/1b6ceed7489e0800.case.json`: A39 A65 A180
- `gen-m2/207a5753da48c7e0.case.json`: A18 A40 A84 A95 A318 A389
- `gen-m2/2cfe279fe77a000d.case.json`: A18 A37 A119 A270
- `gen-m2/60a543744fc4a8f7.case.json`: A18 A106 A362
- `gen-m2/61cacc22c4244179.case.json`: A11 A71 A86 A185 A200 A317 A360 A396
- `gen-m2/629e6e8b76fab8b9.case.json`: A177 A179 A210 A211 A226
- `gen-m2/7469c3e920e1ade4.case.json`: A15 A29 A87 A283 A369
- `gen-m2/75840b9c71a50596.case.json`: A370
- `gen-m2/78bc88199fc96650.case.json`: A56 A77 A246 A325 A400
- `gen-m2/893302d6c5b99e2e.case.json`: A38 A47 A97 A198 A229 A265 A295 A334 A351 A357 A374
- `gen-m2/8eb18149d4245e1a.case.json`: A45 A71 A128 A149 A182 A274
- `gen-m2/942bdade381b7537.case.json`: A54 A102 A104 A123 A187 A192 A227 A237 A241 A247 A255 A336 A346 A367 A386
- `gen-m2/96a0952412cefa6c.case.json`: A6 A171 A223 A273 A297
- `gen-m2/a339ea84ef64382e.case.json`: A12 A61 A90 A94 A115 A210 A342 A353 A363
- `gen-m2/a6d4377c7b1b92cd.case.json`: A17 A57 A181 A354
- `gen-m2/b4392a6514a36ade.case.json`: A63 A125 A167 A206 A218 A234 A241 A297
- `gen-m2/caa981b0753f455c.case.json`: A19 A92 A194 A238
- `gen-m2/cbdabe32a32a4012.case.json`: A26 A351 A362
- `gen-m2/ce315bb699e1be6f.case.json`: A1 A92 A119 A166 A313 A336
- `gen-m2/d04d501154afc771.case.json`: A113 A239 A326 A350
- `gen-m2/d0c29dd5c530e28c.case.json`: A63 A65 A82 A226 A229 A300
- `gen-m2/d3e2df0bef2f173a.case.json`: A7 A103 A160 A166 A181 A229 A372
- `gen-m2/dd99be1f2454159f.case.json`: A206 A246 A332
- `gen-m2/e8493fafa2a5378e.case.json`: A67 A103 A135 A152 A191 A275 A289 A315
- `gen-m2/ef5c55a81783b7d5.case.json`: A225 A378 A384
- `gen-m2/f0d1bacec5de8146.case.json`: A2 A106 A164 A287 A341
- `gen-m2/f7e6d351c8538ef5.case.json`: A87 A116 A214 A299
- `gen-m2/f9d4df4efdfbcbd5.case.json`: A39 A137 A154 A172 A174 A228 A269 A352
- `gen-m2/fa437879253827be.case.json`: A8 A72 A167 A322

These cells also need another task:
- `gen-m2/207a5753da48c7e0.case.json` A211 (also: ifs_shape)
- `gen-m2/d04d501154afc771.case.json` A353 (also: andor_err)

## Acceptance criteria
- [ ] The 164 cells above disappear from eval/baseline/gen-m2.txt with no new F lines.
- [ ] Unit tests: 2-D plus one index (1, 0, 1e-15) gives `#REF!`; an error in the index propagates;
      vector sources unchanged.
- [ ] docs/excel-semantics/lookup.md has the rule with the evidence cells above.
- [ ] `make check` green; reviewers approve.

## Out of scope
- INDEX returning references in aggregates (T-0216). Spill/`#SPILL!` decoding (T-0208).
