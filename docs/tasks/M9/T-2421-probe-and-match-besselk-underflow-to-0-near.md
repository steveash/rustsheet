---
id: T-2421
title: "Probe and match BESSELK underflow to 0 near x 713 and BITLSHIFT shifts of 53 and -53 giving 0"
milestone: M9
lane: functions-engineering
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
BESSELK and BITLSHIFT/BITRSHIFT give Excel's values where the smoke check found them differ.

## Context
Found by the T-2249 smoke check: a scratch suite of 790 cells, one formula per entry of the M9 engineering and database pools, filled through excel-graph on 2026-10-22 and deleted afterwards (docs/excel-semantics/engineering.md and criteria.md, "edge-pool smoke check (T-2249)"). Commit a probe case with these cells first (excel-probe skill).
- `BESSELK(713,1)` and `BESSELK(714,1)` are 0. Ours gives the subnormals 1.0465929954E-311 and 3.84750046922E-312. Find where Excel starts to flush: whether its result is cut at the normal range (2.2250738585072E-308) or its kernel underflows earlier. `BESSELI(713,0)` is `#NUM!`, which ours matches.
- `BITLSHIFT(1,53)` and `BITLSHIFT(1,-53)` are 0, while `BITLSHIFT(1,48)` and `BITLSHIFT(1,54)` are `#NUM!` and `BITLSHIFT(1,47)` is 140737488355328. Ours `#NUM!` at ±53. Probe shifts 48 to 54 and -48 to -54 with numbers 0, 1 and 2^47, and BITRSHIFT the same.
Code: functions/engineering (bessel_k.rs, the BIT* functions).

gen-m9 engineering sweep (gap triage 2026-10-23, T-2258; engineering.md "Engineering functions in the gen-m9 engineering sweep (T-2258)"): 17 more cells. `BITRSHIFT(n,-53)` is 0 for n = 41, 434, 633, 699, 944 and `BITLSHIFT(218,53)` 0; a number 0 shifts to 0 by 54 either way (`BITRSHIFT(0,54)`, `BITLSHIFT(<empty>,-54)`) although `BITLSHIFT(1,-54)` is `#NUM!` (8 cells, ours `#NUM!`). `BESSELK(713,4)`, `BESSELK(714,0)` (2), `(714,1)`, `(714,4)` are 0 (5, ours subnormals). `BESSELI(1E+15,n)` for n = 2, 3, 5 and `BESSELI(-1000000000000002,3)` are `#NUM!` (4, ours 0): the overflow, as `BESSELI(713,0)`, not an underflow; check where ours returns 0 for a huge |x|. BESSELK(700,3)'s 15th-digit miss is T-2549 (sweep T-2550, which can share the probe).
Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 17 known-failure lines in eval/baseline/gen-m9.txt, 16 formulas, 17 visibly off.
Examples:
- gen-m9/01810fc89c5ec2c0 Calc!A372 `=BITRSHIFT(633,(-53))`: Excel 0.0, ours #NUM!
- gen-m9/12e4aa13a20cc1f8 Calc!A207 `=BESSELK(714,0)`: Excel 0.0, ours 3.84480912157e-312
- gen-m9/2f0d384b8ed0d392 Calc!A259 `=BESSELI((((-1)-1000000000000000.0)-Data!A$10),3)`: Excel #NUM!, ours 0.0
- gen-m9/3cf48801617a72ad Calc!A41 `=BESSELK(713,4)`: Excel 0.0, ours 1.057652364502e-311
- gen-m9/59a16b5ab566ee5c Calc!A393 `=BESSELI(1000000000000000.0,2)`: Excel #NUM!, ours 0.0
- gen-m9/65dc4e4da669f98a Calc!A251 `=BESSELK(714,4)`: Excel 0.0, ours 3.88809988249e-312
Cells:
- gen-m9/01810fc89c5ec2c0.case.json (1): Calc!A372
- gen-m9/12e4aa13a20cc1f8.case.json (1): Calc!A207
- gen-m9/2f0d384b8ed0d392.case.json (1): Calc!A259
- gen-m9/3cf48801617a72ad.case.json (1): Calc!A41
- gen-m9/4fffa5addab1f8af.case.json (1): Calc!A317
- gen-m9/59a16b5ab566ee5c.case.json (1): Calc!A393
- gen-m9/65dc4e4da669f98a.case.json (2): Calc!A251, A351
- gen-m9/756a9f7bc3a06bcc.case.json (1): Calc!A143
- gen-m9/7c7e5bff12d93365.case.json (1): Calc!A6
- gen-m9/a7fda34502ed3030.case.json (1): Calc!A312
- gen-m9/b5aef3645cec1b95.case.json (1): Calc!A66
- gen-m9/bf979a17ab5aafcb.case.json (1): Calc!A187
- gen-m9/ca182d96dedee546.case.json (1): Calc!A107
- gen-m9/e882060cf4613302.case.json (1): Calc!A28
- gen-m9/ee70a5515d004a66.case.json (1): Calc!A45
- gen-m9/ef33be4fd2389f3c.case.json (1): Calc!A150

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2258): these 17 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] A probe case holds the cells above and the shift grid, plus the BESSEL order-cap evidence the smoke check saw
      (`BESSELI(2.5,100001)` and `BESSELI(2.5,1E+6)` are 0) and `BESSELK(2.5,100001)`, so the DEVIATION entry
      "BESSEL order cap" cites a committed probe.
- [ ] Ours matches every probe cell; docs/excel-semantics/engineering.md is updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The BESSEL order cap (a DEVIATION, decision 0014).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2258): every cell this task owns has the wrong type, error, text or value (none is a last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria states it.
