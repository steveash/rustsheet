---
id: T-0209
title: "SUMIFS/COUNTIFS: a criteria range shaped unlike the first range is #VALUE! (442 gen-m2 known failures)"
milestone: M2
lane: functions
status: todo
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
SUMIFS and COUNTIFS return `#VALUE!` when any criteria_range (or SUMIFS's sum_range) has a different
**shape** (rows x columns) from the first range. Today the engine walks the ranges by offset and
returns a number.

## Context
Largest single root cause in gen-m2 (T-0203): 442 failing cells, all verdict TypeMismatch/ErrorMismatch,
reported under SUMIFS, COUNTIFS and every function that nests them (SUM, IF, AVERAGE, ...).
Evidence (oracle excel-graph, gen-m2 sidecars):
- `gen-m2/04cfd82f54dbc2b2` Calc!A150 `=SUMIFS(Data!B7:C9,Data!A10:D12,"<5")` (3x2 vs 3x4): expected `#VALUE!`, actual 1.2049.
- `gen-m2/1b6ceed7489e0800` Calc!A372 `=SUMIFS(Data!B4:B11,Data!C2:D2,"<>")`: expected `#VALUE!`, actual 83.5.
- `gen-m2/207a5753da48c7e0` Calc!A327 `=COUNTIFS(Data!B5:D9,"<5",Data!B10:E11,"<5")`: expected `#VALUE!`, actual 2.
- **Same size, different shape is still `#VALUE!`**: `gen-m2/7469c3e920e1ade4` Calc!A51
  `=COUNTIFS(Data!A11:D12,1,Data!B9:C12,"abc")` (2x4 vs 4x2): expected `#VALUE!`, actual 0.
- Across all 171 top-level SUMIFS/COUNTIFS calls in gen-m2: every mismatched-shape call (88 SUMIFS,
  57 COUNTIFS) is `#VALUE!`. Every same-shape call (26) gives a number.
- SUMIF is different: it never errors on a shape mismatch (T-0214).

Code: `crates/rustsheet-engine/src/functions/statistical/if_ifs.rs` (`fn_countifs`, `apply_ifs`; both
carry `// TODO test ranges are of the same size`). `apply_ifs` also serves AVERAGEIFS/MINIFS/MAXIFS.
Probe them with the excel-probe skill before extending the rule to them. Semantics entry: see the
criteria quirk the triager reported for docs/excel-semantics/criteria.md (add it in this PR if it is
not there yet).

Cells whose only gen-m2 root cause is this one (30 cases, 437 cells):
- `gen-m2/04cfd82f54dbc2b2.case.json`: A5 A28 A104 A150 A176 A193 A274 A280 A282 A304 A308 A321 A345 A352 A370 A372 A380
- `gen-m2/1b6ceed7489e0800.case.json`: A7 A55 A89 A110 A124 A130 A139 A145 A148 A199 A293 A316 A335 A351 A353 A358 A372
- `gen-m2/207a5753da48c7e0.case.json`: A10 A87 A96 A102 A104 A115 A131 A168 A184 A191 A204 A214 A221 A223 A255 A281 A301 A304 A327 A362
- `gen-m2/2cfe279fe77a000d.case.json`: A25 A99 A107 A113 A117 A156 A170 A189 A195 A210 A248 A251 A275 A278 A315 A329 A332 A393 A400
- `gen-m2/60a543744fc4a8f7.case.json`: A47 A72 A83 A120 A122 A128 A130 A150 A196 A217 A267 A356
- `gen-m2/61cacc22c4244179.case.json`: A12 A20 A54 A94 A95 A98 A109 A128 A140 A170 A172 A178 A208 A218 A263 A298 A309 A320 A326 A350
- `gen-m2/629e6e8b76fab8b9.case.json`: A58 A63 A92 A130 A137 A288 A305 A344 A374 A392
- `gen-m2/7469c3e920e1ade4.case.json`: A51 A122 A124 A132 A149 A178 A182 A217 A224 A296 A307 A323 A342 A376
- `gen-m2/75840b9c71a50596.case.json`: A20 A46 A115 A136 A174 A255 A278 A333 A341 A343 A355 A359 A395 A400
- `gen-m2/78bc88199fc96650.case.json`: A23 A70 A71 A161 A164 A172 A186 A234 A237 A242 A244 A273 A278 A308 A335
- `gen-m2/893302d6c5b99e2e.case.json`: A21 A32 A116 A121 A134 A192 A261 A309 A332 A337 A373
- `gen-m2/8eb18149d4245e1a.case.json`: A37 A92 A94 A139 A140 A154 A164 A198 A237 A243 A305 A318 A338 A347 A358 A389 A391 A394
- `gen-m2/942bdade381b7537.case.json`: A12 A18 A50 A58 A97 A154 A162 A181 A201 A231 A234 A305 A338 A356 A373 A387 A396
- `gen-m2/96a0952412cefa6c.case.json`: A1 A84 A85 A108 A158 A211 A281 A318 A331 A379 A398
- `gen-m2/a339ea84ef64382e.case.json`: A1 A45 A78 A161 A216 A219 A220 A223 A254 A283 A304 A306 A344 A377 A396
- `gen-m2/a6d4377c7b1b92cd.case.json`: A6 A156 A195 A202 A244 A254 A274 A289 A295 A315 A335 A349
- `gen-m2/b4392a6514a36ade.case.json`: A41 A70 A183 A185 A193 A258 A294 A312 A315 A320 A361 A387
- `gen-m2/caa981b0753f455c.case.json`: A61 A87 A121 A128 A187 A212 A239 A308 A309 A385
- `gen-m2/cbdabe32a32a4012.case.json`: A3 A34 A36 A84 A104 A112 A120 A198 A234 A241 A342 A386 A387
- `gen-m2/ce315bb699e1be6f.case.json`: A14 A29 A109 A143 A193 A246 A256 A260 A268 A301 A335 A354 A367 A368
- `gen-m2/d04d501154afc771.case.json`: A24 A38 A42 A76 A83 A86 A91 A102 A318 A320 A332 A348 A361
- `gen-m2/d0c29dd5c530e28c.case.json`: A7 A11 A17 A71 A117 A142 A145 A148 A153 A160 A163 A281 A295 A314 A327 A369 A371 A374 A391
- `gen-m2/d3e2df0bef2f173a.case.json`: A9 A29 A31 A50 A55 A60 A77 A81 A89 A127 A145 A193 A225 A234 A264 A274 A303 A336 A360 A380 A387
- `gen-m2/dd99be1f2454159f.case.json`: A17 A77 A144 A175 A177 A222 A230 A257 A372 A378
- `gen-m2/e8493fafa2a5378e.case.json`: A6 A18 A21 A53 A54 A64 A85 A123 A230 A253 A273 A300 A336
- `gen-m2/ef5c55a81783b7d5.case.json`: A1 A14 A29 A35 A76 A111 A122 A145 A233 A264 A270 A276 A292 A302 A336 A345 A392
- `gen-m2/f0d1bacec5de8146.case.json`: A4 A54 A56 A63 A91 A140 A189 A202 A273 A294 A301 A305 A340 A348 A380
- `gen-m2/f7e6d351c8538ef5.case.json`: A10 A50 A61 A137 A202 A277 A288 A305 A340 A351 A361 A384 A399
- `gen-m2/f9d4df4efdfbcbd5.case.json`: A32 A74 A91 A123 A149 A181 A236 A239 A329 A330 A356 A364 A373 A398
- `gen-m2/fa437879253827be.case.json`: A1 A19 A122 A153 A163 A202 A281 A287 A308 A350 A362

These cells also need another task:
- `gen-m2/207a5753da48c7e0.case.json` A211 (also: index2d)
- `gen-m2/207a5753da48c7e0.case.json` A333 (also: datepart)
- `gen-m2/78bc88199fc96650.case.json` A217 (also: andor_err)
- `gen-m2/893302d6c5b99e2e.case.json` A371 (also: sumif_resize)
- `gen-m2/dd99be1f2454159f.case.json` A114 (also: datepart)
- `gen-m2/f7e6d351c8538ef5.case.json` A329 (also: datepart)

## Acceptance criteria
- [ ] The 437 cells above disappear from eval/baseline/gen-m2.txt with no new F lines. The 6 shared cells leave once
      their other task lands too.
- [ ] Unit tests: mismatched shape (including the same-size transposed case) gives `#VALUE!` for SUMIFS
      and COUNTIFS. Same shape still sums or counts.
- [ ] docs/excel-semantics/criteria.md has the rule with the evidence cells above.
- [ ] `make check` green; reviewers approve.

## Out of scope
- SUMIF/AVERAGEIF resizing (T-0214).
- AVERAGEIFS/MINIFS/MAXIFS unless an oracle probe confirms the same rule (file a follow-up).
