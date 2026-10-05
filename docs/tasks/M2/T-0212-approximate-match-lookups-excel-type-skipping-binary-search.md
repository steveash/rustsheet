---
id: T-0212
title: "Approximate-match lookups: Excel type-skipping binary search in VLOOKUP/HLOOKUP/MATCH (484 gen-m2 known failures)"
milestone: M2
lane: functions-lookup
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-0210, T-0211]
---

## Goal
Approximate-match VLOOKUP/HLOOKUP (range_lookup TRUE/omitted/non-zero) and MATCH (match_type 1 and -1)
return the same row/column as Excel on unsorted, mixed-type data. Exact match (FALSE/0) only matches
cells of the lookup value's own type.

## Context
gen-m2 (T-0203): about 489 failing lookup cells are not explained by T-0210/T-0211 alone. A Python
emulation of the algorithm below reproduces all 124 top-level VLOOKUP/HLOOKUP/MATCH cells in gen-m2
whose arguments contain no other function (passing and failing alike; 2 more with a text
range_lookup were not emulated). Substituting it for every lookup call
(innermost first) and re-running the engine explains all but about 22 of the ~925 lookup-related failures.

Algorithm that matches every one of those 124 cells (excel-graph). Implement it from this evidence, then
probe edge cases with the excel-probe skill before calling it done:
```
lo = 0; hi = n-1; best = none            # t = type of lookup value (number/text/bool); blank key = 0
while lo <= hi:
    mid = (lo+hi) / 2 (floor)
    m = mid; while m <= hi and type(v[m]) != t: m += 1     # skip blanks and other types upwards
    if m > hi: hi = mid - 1; continue
    c = compare(v[m], key)               # same type only; text case-insensitive
    if c == 0: return m                  # first equal hit wins (no run to the last duplicate)
    if (c < 0 and match_type >= 1) or (c > 0 and match_type == -1): best = m; lo = m + 1
    else: hi = m - 1
return best (none -> #N/A)
```
Evidence cells (expected / actual today):
- Blank cells never match: `gen-m2/04cfd82f54dbc2b2` Calc!A1 `=VLOOKUP(Data!B4,Data!C5:C6,1)` (42 in {"abc", blank}): `#N/A` / 0.
- No cross-type match: `gen-m2/d0c29dd5c530e28c` Calc!A67 `=VLOOKUP("#N/A",Data!D6:D11,1)` (text in numbers): `#N/A` / 42.
  `gen-m2/2cfe279fe77a000d` Calc!A132 `=MATCH(Data!E$7,Data!E10:E10)` ("TRUE" text against 2.5): `#N/A` / 1.
- Skip upwards past other types: `gen-m2/fa437879253827be` Calc!A146 `=HLOOKUP((-2.5),Data!B3:E7,4)`
  (top row {1,"ü",blank,-2.5}): 2 / `#N/A`.
- Greater value moves hi to m-1: `gen-m2/cbdabe32a32a4012` Calc!A87 (key 3.14159 over {1,123456789.123,2,blank,"1","1.5","abc",100,"x",FALSE}) gives row 1 (1.0) / 0.
- Text search over mixed data: `gen-m2/f9d4df4efdfbcbd5` Calc!A8 `=VLOOKUP("ABC",Data!B1:C9,1)`: "ABC" / 0.2.
  `gen-m2/629e6e8b76fab8b9` Calc!A389 `=VLOOKUP(TRIM("1"),Data!C2:C12,1,IF(TRUE,123456789.123))`: "1" / 2.5.
- Equality returns at once: `gen-m2/a6d4377c7b1b92cd` Calc!A33 `=MATCH(SUM(Data!B10:B10),Data!B10:B11,-1)` (0.5 in {0.5,1E+15}): 1 / 2.
- match_type -1 on mixed data: `gen-m2/ce315bb699e1be6f` Calc!A395 `=MATCH(3.14159,Data!B4:B11,-1)`: `#N/A` / 7.
- Text ordering uses Excel collation (T-0220). The 22 residual lookup cells include
  `gen-m2/60a543744fc4a8f7` Calc!A280 (`"?b"` key) and `gen-m2/a339ea84ef64382e` Calc!A246, which the
  emulation (Python code-point order) gets wrong.

Code: `SearchVector::binary_search` in `crates/rustsheet-engine/src/functions/lookup_and_reference/lookup_table.rs` (T-0119; VLOOKUP/HLOOKUP
via `lookup_index`, MATCH 1), the inline -1 loop in `fn_match`, the exact branch of `lookup_index` (`SearchVector::position`).
Keep one search helper (CLAUDE.md directive 3). XLOOKUP/XMATCH have their own Excel semantics and stay
out unless evidence says they share this one.

Failing cells attributed to this root cause (some also need T-0210/T-0211, hence the dependency):
- `gen-m2/04cfd82f54dbc2b2.case.json`: A1 A4 A19 A52 A58 A88 A111 A142 A168 A192 A207 A267 A322 A368 A389 A392
- `gen-m2/1b6ceed7489e0800.case.json`: A9 A20 A24 A38 A52 A74 A83 A91 A108 A117 A135 A168 A192 A200 A298 A305 A314 A360 A370
- `gen-m2/207a5753da48c7e0.case.json`: A108 A173 A293 A296 A302 A335 A336 A348 A355 A361 A365 A379 A385
- `gen-m2/2cfe279fe77a000d.case.json`: A14 A28 A85 A91 A94 A132 A159 A168 A185 A196 A243 A260 A269 A277 A282 A294 A316
- `gen-m2/60a543744fc4a8f7.case.json`: A45 A94 A147 A201 A243 A263 A273 A296 A321 A336 A345 A350 A367 A377
- `gen-m2/61cacc22c4244179.case.json`: A16 A53 A101 A105 A111 A115 A173 A179 A192 A209 A226 A292 A307 A310 A323 A332 A346
- `gen-m2/629e6e8b76fab8b9.case.json`: A2 A4 A27 A35 A65 A76 A85 A194 A263 A265 A280 A329 A358 A372 A387 A389
- `gen-m2/7469c3e920e1ade4.case.json`: A20 A36 A64 A67 A135 A146 A170 A172 A211 A214 A264 A266 A306 A315 A350 A384
- `gen-m2/75840b9c71a50596.case.json`: A8 A33 A40 A41 A55 A56 A63 A72 A89 A113 A124 A135 A178 A250 A263 A270 A327 A372 A382 A388
- `gen-m2/78bc88199fc96650.case.json`: A20 A37 A96 A184 A200 A224 A269 A298 A388
- `gen-m2/893302d6c5b99e2e.case.json`: A3 A42 A57 A59 A67 A68 A205 A235 A245 A260 A290 A316 A323 A400
- `gen-m2/8eb18149d4245e1a.case.json`: A21 A60 A112 A120 A230 A238 A378
- `gen-m2/942bdade381b7537.case.json`: A68 A74 A125 A133 A169 A189 A194 A223 A244 A266 A274 A278
- `gen-m2/96a0952412cefa6c.case.json`: A33 A82 A92 A93 A145 A149 A150 A166 A206 A210 A221 A239 A241 A274 A335 A336 A363 A372 A375
- `gen-m2/a339ea84ef64382e.case.json`: A31 A37 A59 A100 A112 A122 A136 A163 A164 A169 A187 A221 A225 A259 A281 A292 A315 A336 A356 A364 A376
- `gen-m2/a6d4377c7b1b92cd.case.json`: A21 A28 A33 A48 A75 A99 A154 A206 A216 A219 A242 A256 A270 A283 A304 A307 A336 A362
- `gen-m2/b4392a6514a36ade.case.json`: A58 A166 A187 A200 A257 A261 A289 A346
- `gen-m2/caa981b0753f455c.case.json`: A1 A6 A26 A27 A33 A43 A68 A76 A78 A82 A119 A169 A182 A191 A196 A222 A254 A292 A316 A325 A345 A378 A381 A393
- `gen-m2/cbdabe32a32a4012.case.json`: A9 A55 A63 A66 A81 A87 A116 A144 A188 A298 A300 A326 A332
- `gen-m2/ce315bb699e1be6f.case.json`: A12 A81 A93 A96 A160 A161 A164 A253 A302 A339 A357 A395
- `gen-m2/d04d501154afc771.case.json`: A62 A64 A66 A69 A149 A150 A176 A219 A237 A267 A274 A292 A328 A330
- `gen-m2/d0c29dd5c530e28c.case.json`: A48 A56 A67 A116 A136 A138 A139 A156 A158 A178 A179 A227 A231 A233 A265 A285 A288 A294 A305 A318 A322 A353
- `gen-m2/d3e2df0bef2f173a.case.json`: A23 A33 A59 A91 A135 A136 A164 A200 A220 A257 A258 A263 A272 A287 A327 A331 A366 A398
- `gen-m2/dd99be1f2454159f.case.json`: A3 A6 A27 A41 A55 A151 A174 A195 A200 A205 A211 A214 A231 A241 A245 A272 A311 A316 A337 A342 A348 A360 A399
- `gen-m2/e8493fafa2a5378e.case.json`: A11 A16 A33 A42 A46 A61 A102 A111 A171 A174 A190 A234 A246 A247 A257 A298 A335 A341 A381
- `gen-m2/ef5c55a81783b7d5.case.json`: A89 A93 A101 A150 A170 A207 A210 A213 A223 A297 A309 A317 A391
- `gen-m2/f0d1bacec5de8146.case.json`: A18 A29 A40 A45 A52 A55 A110 A113 A143 A199 A233 A244 A252 A291 A328 A329 A330 A362 A387 A393
- `gen-m2/f7e6d351c8538ef5.case.json`: A6 A21 A66 A72 A100 A123 A159 A160 A173 A191 A208 A218 A219 A232 A235 A245 A247 A275 A278 A287 A290 A291 A292 A312 A319 A390
- `gen-m2/f9d4df4efdfbcbd5.case.json`: A8 A69 A90 A118 A133 A275 A276 A279 A286 A288 A289 A322 A336 A339 A365 A377 A389
- `gen-m2/fa437879253827be.case.json`: A38 A90 A117 A123 A146 A166 A291 A305 A335 A370 A379 A386

## Acceptance criteria
- [ ] With T-0210 and T-0211 merged, at least 460 of the cells above disappear from eval/baseline/gen-m2.txt, with no new
      F lines. List any that remain in the PR with a reason (collation: T-0220).
- [ ] Unit tests for each evidence bullet above.
- [ ] docs/excel-semantics/lookup.md documents the algorithm with the evidence cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Collation of text comparison (T-0220). XLOOKUP/XMATCH.
