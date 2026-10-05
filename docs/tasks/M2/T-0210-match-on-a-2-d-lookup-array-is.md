---
id: T-0210
title: "MATCH on a 2-D lookup_array is #N/A, not #ERROR! (217 gen-m2 known failures)"
milestone: M2
lane: functions
status: todo
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
MATCH with a 2-D lookup_array (more than one row **and** more than one column) returns `#N/A`.
Errors in lookup_value or match_type still win. Today it returns `#ERROR!` from `as_vector`.

## Context
gen-m2 (T-0203): 213 failing cells pass with this change alone. They show up under MATCH and the
functions that nest it (INDEX, IF, SUM, ...). Evidence (excel-graph):
- `gen-m2/04cfd82f54dbc2b2` Calc!A394 `=MATCH("a",Data!D6:E8,-1)`: expected `#N/A`, actual `#ERROR!`.
- `gen-m2/96a0952412cefa6c` Calc!A212 `=MATCH("1.5",Data!B7:D8,1)`: expected `#N/A`, actual `#ERROR!`.
- `gen-m2/ce315bb699e1be6f` Calc!A135 `=MATCH(Data!E2,Data!A6:C7,0)`: expected `#N/A`, actual `#ERROR!`.
- Precedence: of 78 top-level 2-D MATCH calls, 53 are `#N/A`. The others propagate an argument error
  first: `#VALUE!` (18), `#REF!` (3), `#DIV/0!` (2, e.g. `gen-m2/893302d6c5b99e2e` Calc!A284) and
  `#NUM!` (2, e.g. `gen-m2/629e6e8b76fab8b9` Calc!A313
  `=MATCH(MAX(MONTH((-0.5))),Data!A7:C9)`). A `#VALUE!` from a mismatched COUNTIFS in match_type also
  wins: `gen-m2/1b6ceed7489e0800` Calc!A91 (that cell also needs T-0209).

Code: `crates/rustsheet-engine/src/functions/lookup_and_reference/mod.rs::fn_match` (`as_vector`). Check
whether XMATCH shares the path, and leave it unchanged unless evidence says otherwise.

Cells that pass with this fix alone (simulated by rewriting the call to `NA()` and re-running the engine):
- `gen-m2/04cfd82f54dbc2b2.case.json`: A78 A89 A178 A190 A199 A343 A347 A394
- `gen-m2/1b6ceed7489e0800.case.json`: A87 A122 A204 A260 A268 A271 A399
- `gen-m2/207a5753da48c7e0.case.json`: A107 A159 A265 A292 A314
- `gen-m2/2cfe279fe77a000d.case.json`: A11 A19 A22 A62 A180 A183 A192 A194 A310 A346
- `gen-m2/60a543744fc4a8f7.case.json`: A70 A134 A327
- `gen-m2/61cacc22c4244179.case.json`: A1 A60 A148 A237 A289 A296
- `gen-m2/629e6e8b76fab8b9.case.json`: A90 A110 A163 A282 A295 A336
- `gen-m2/7469c3e920e1ade4.case.json`: A65 A106 A165 A174 A183 A222 A227 A289 A377
- `gen-m2/75840b9c71a50596.case.json`: A101 A182 A187 A253 A330
- `gen-m2/78bc88199fc96650.case.json`: A16 A54 A135 A236 A299 A317
- `gen-m2/893302d6c5b99e2e.case.json`: A54 A147 A159 A167 A189 A206 A208 A225 A304 A335 A336 A348 A387 A396
- `gen-m2/8eb18149d4245e1a.case.json`: A1 A114 A131 A223 A254 A261 A285 A331
- `gen-m2/942bdade381b7537.case.json`: A103 A226 A281 A332
- `gen-m2/96a0952412cefa6c.case.json`: A101 A103 A187 A212 A225 A238 A368
- `gen-m2/a339ea84ef64382e.case.json`: A108 A114 A184 A229 A278 A291 A316
- `gen-m2/a6d4377c7b1b92cd.case.json`: A67 A130 A139 A251 A328 A357
- `gen-m2/b4392a6514a36ade.case.json`: A44 A90 A190 A272 A295 A322 A378 A381
- `gen-m2/caa981b0753f455c.case.json`: A9 A130 A220 A311 A315 A328 A360
- `gen-m2/cbdabe32a32a4012.case.json`: A115 A227 A287 A292 A296 A322 A340 A350
- `gen-m2/ce315bb699e1be6f.case.json`: A55 A135 A154 A167 A295 A303 A340 A365 A373
- `gen-m2/d04d501154afc771.case.json`: A57 A119 A192 A205 A244 A248 A277 A279 A371 A383 A385
- `gen-m2/d0c29dd5c530e28c.case.json`: A22 A84 A119 A286 A315 A397
- `gen-m2/d3e2df0bef2f173a.case.json`: A38 A153 A187 A240
- `gen-m2/dd99be1f2454159f.case.json`: A70 A95 A118 A156 A394
- `gen-m2/e8493fafa2a5378e.case.json`: A28 A99 A118 A175
- `gen-m2/ef5c55a81783b7d5.case.json`: A18 A25 A94 A146 A214 A216 A236 A305 A344
- `gen-m2/f0d1bacec5de8146.case.json`: A16 A17 A90 A372
- `gen-m2/f7e6d351c8538ef5.case.json`: A77 A105 A110 A118 A140 A170 A223 A239 A304 A386
- `gen-m2/f9d4df4efdfbcbd5.case.json`: A111 A128 A186 A193 A230 A255 A284
- `gen-m2/fa437879253827be.case.json`: A29 A43 A65 A89 A102 A115 A145 A194 A277 A315

## Acceptance criteria
- [ ] The 213 cells above disappear from eval/baseline/gen-m2.txt with no new F lines.
- [ ] Unit tests: 2-D range gives `#N/A`; lookup_value error and match_type error take precedence.
- [ ] docs/excel-semantics/lookup.md has the rule with the evidence cells above.
- [ ] `make check` green; reviewers approve.

## Out of scope
- How MATCH searches (approximate/exact type rules): T-0212.
