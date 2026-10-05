---
id: T-0211
title: "VLOOKUP/HLOOKUP: search before validating the column/row index (201 gen-m2 known failures)"
milestone: M2
lane: functions-lookup
status: todo
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
VLOOKUP and HLOOKUP look the value up **before** validating col_index_num/row_index_num. When the
value is not found the result is `#N/A`, whatever the index is. Only a found value goes on to index
validation, and there a non-numeric text index is `#REF!`.

## Context
gen-m2 (T-0203): 205 failing cells pass with this ordering alone (simulated as
`IF(ISNA(VLOOKUP(k,t,1,m)),NA(),VLOOKUP(k,t,c,m))` re-run through the engine). Evidence (excel-graph):
- Not found, index > width: `gen-m2/1b6ceed7489e0800` Calc!A24 `=VLOOKUP("a",Data!E2:E6,3)` (no text in E2:E6): expected `#N/A`, actual `#REF!`.
- Not found, index < 1: `gen-m2/2cfe279fe77a000d` Calc!A85 `=VLOOKUP(Data!E11,Data!B7:D11,(-2.5))`: expected `#N/A`, actual `#REF!`.
- Not found, exact match: `gen-m2/8eb18149d4245e1a` Calc!A17 `=VLOOKUP(Data!A$9,Data!A11:C11,4,FALSE)`: expected `#N/A`, actual `#REF!`.
- Found, index is non-numeric text, gives `#REF!` (not `#VALUE!`): `gen-m2/61cacc22c4244179` Calc!A310
  `=(""&(Data!E$7&VLOOKUP("x~*",Data!D2:E12,Data!$E1)))` with Data!E1 = "a": expected `#REF!`, actual
  `#VALUE!`. The same for HLOOKUP: `gen-m2/caa981b0753f455c` Calc!A182
  `=HLOOKUP(Data!$E3,Data!A1:C3,Data!B$4)` with B4 = "Hello World": expected `#REF!`, actual `#VALUE!`.
- Found, numeric-text index coerces: `gen-m2/cbdabe32a32a4012` Calc!A87
  `=VLOOKUP(Data!$E1,Data!C3:D12,Data!C8)` with C8 = "1.5" acts as index 1 (expected 1). That cell
  also needs T-0212.
- Found, index -1E+15 gives `#REF!`: `gen-m2/629e6e8b76fab8b9` Calc!A53 (`VLOOKUP(YEAR(0.2),Data!B2:D2,-1000000000000000.0)`).
  This is a single cell. Probe whether a large-magnitude negative index is `#REF!` while -1 is
  `#VALUE!` before encoding it.

Code: `crates/rustsheet-engine/src/functions/lookup_and_reference/mod.rs` (`fn_vlookup`, `fn_hlookup`).

Cells that pass with this fix alone:
- `gen-m2/04cfd82f54dbc2b2.case.json`: A15 A22 A29 A36 A65 A66 A145 A306 A338
- `gen-m2/1b6ceed7489e0800.case.json`: A23 A115 A189 A196 A220 A224 A309 A328 A400
- `gen-m2/207a5753da48c7e0.case.json`: A112 A134 A217 A240 A263
- `gen-m2/2cfe279fe77a000d.case.json`: A35 A205 A232 A323 A384
- `gen-m2/60a543744fc4a8f7.case.json`: A135 A156 A164 A227 A241 A256 A275 A292 A328
- `gen-m2/61cacc22c4244179.case.json`: A37 A56 A155 A205 A327 A354 A392
- `gen-m2/629e6e8b76fab8b9.case.json`: A23 A47 A82 A150 A197 A212 A232 A257 A279 A306 A331 A371 A382
- `gen-m2/7469c3e920e1ade4.case.json`: A69 A144 A304 A305 A338 A355 A364
- `gen-m2/75840b9c71a50596.case.json`: A97 A141 A204 A241 A254 A356
- `gen-m2/78bc88199fc96650.case.json`: A27 A138 A230 A238
- `gen-m2/893302d6c5b99e2e.case.json`: A25 A40 A58 A90 A117 A163 A167 A237 A267 A328 A350
- `gen-m2/8eb18149d4245e1a.case.json`: A5 A17 A191 A215 A284 A303 A320 A335
- `gen-m2/942bdade381b7537.case.json`: A31 A141 A161 A257
- `gen-m2/96a0952412cefa6c.case.json`: A53 A157 A169 A329 A348 A358 A393
- `gen-m2/a339ea84ef64382e.case.json`: A33 A137 A240 A331
- `gen-m2/a6d4377c7b1b92cd.case.json`: A44 A60 A72 A98 A123 A132 A220 A238 A294 A366
- `gen-m2/b4392a6514a36ade.case.json`: A13 A30 A95 A109 A170 A224 A235 A247 A251 A252 A259
- `gen-m2/caa981b0753f455c.case.json`: A24 A110 A247
- `gen-m2/cbdabe32a32a4012.case.json`: A74 A129 A161 A216 A259 A271 A280 A391
- `gen-m2/ce315bb699e1be6f.case.json`: A6 A30 A75 A85 A222 A239
- `gen-m2/d04d501154afc771.case.json`: A127 A186 A265 A345 A381
- `gen-m2/d0c29dd5c530e28c.case.json`: A5 A98 A240 A303 A375
- `gen-m2/d3e2df0bef2f173a.case.json`: A85 A124 A151 A176 A256 A286 A306 A338 A355 A365
- `gen-m2/dd99be1f2454159f.case.json`: A62 A157 A187 A255 A328 A366 A397
- `gen-m2/e8493fafa2a5378e.case.json`: A98 A245 A338 A399
- `gen-m2/ef5c55a81783b7d5.case.json`: A32 A59 A85 A114 A121 A205 A259 A268
- `gen-m2/f0d1bacec5de8146.case.json`: A74 A100 A109 A238 A366
- `gen-m2/f7e6d351c8538ef5.case.json`: A193 A373
- `gen-m2/f9d4df4efdfbcbd5.case.json`: A10 A25 A37 A89 A301 A333
- `gen-m2/fa437879253827be.case.json`: A57 A114 A124 A227 A273 A279 A303

## Acceptance criteria
- [ ] The 205 cells above disappear from eval/baseline/gen-m2.txt with no new F lines.
- [ ] Unit tests for each bullet above (not found beats any index; found plus text index is `#REF!`;
      found plus numeric text index coerces).
- [ ] docs/excel-semantics/lookup.md has the rule with the evidence cells above.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The search algorithm itself (T-0212). MATCH (T-0210).
