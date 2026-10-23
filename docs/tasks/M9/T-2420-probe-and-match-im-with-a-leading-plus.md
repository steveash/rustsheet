---
id: T-2420
title: "Probe and match IM* with a leading plus, OCT2DEC of 11 digits and OCT/HEX conversions of TRUE"
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
IM* parsing, OCT2DEC and the OCT/HEX conversions give Excel's values on the three inputs where they differ.

## Context
Found by the T-2249 smoke check: a scratch suite of 790 cells, one formula per entry of the M9 engineering and database pools, filled through excel-graph on 2026-10-22 and deleted afterwards (docs/excel-semantics/engineering.md and criteria.md, "edge-pool smoke check (T-2249)"). Commit a probe case with these cells first (excel-probe skill).
- `IMABS("+i")` is 1 and `IMSUB("+i","1")` is "-1+i": a leading `+` before a bare `i` is read. Ours `#NUM!`. Probe `"+3+4i"`, `"+4i"`, `"+3"` and the other IM* functions too.
- `OCT2DEC(77777777777)` (11 octal digits, a number) is `#NUM!`. Ours 7516192767.
- `OCT2DEC(TRUE)`, `OCT2BIN(TRUE)`, `HEX2DEC(TRUE)`, `HEX2BIN(TRUE)` and `HEX2OCT(TRUE)` are `#VALUE!`, as `BIN2DEC(TRUE)` already is (engineering.md "Engineering function argument modes"). Ours `#NUM!`.
Code: functions/engineering (complex parsing, base conversions).

gen-m9 engineering sweep (gap triage 2026-10-23, T-2258; engineering.md "Engineering functions in the gen-m9 engineering sweep (T-2258)"): 63 more cells. A leading `+` before a bare `i` (33): IMDIV first or second argument (`IMDIV("+i","4")` "0.25i", `IMDIV("2i","+i")` "2"), IMSUB either argument, IMLN 1.5707963267949i, IMLOG10 0.682188176920921i, IMLOG2 2.2661800709136i, IMSQRT, IMSIN, IMCOS, IMCOSH, IMSEC, IMSECH; once "+i" is read the other argument's error decides (`IMPOWER("+i",<#VALUE!>)` `#VALUE!`, `IMSUM("+i",<#N/A>)` and `IMSUB("+i",<#N/A>)` `#N/A`). A boolean in OCT2*/HEX2* (29): also OCT2HEX(TRUE) with or without places, a TRUE or FALSE cell, an `IF(..)` FALSE and an `INDEX(..)` TRUE are `#VALUE!` (ours `#NUM!`). And `OCT2DEC(77777777777)` once more (1).
Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 63 known-failure lines in eval/baseline/gen-m9.txt, 48 formulas, 63 visibly off.
Examples:
- gen-m9/01810fc89c5ec2c0 Calc!A66 `=IMDIV("+i","4")`: Excel "0.25i", ours #NUM!
- gen-m9/060b18586fcff6e6 Calc!A380 `=OCT2DEC(TRUE)`: Excel #VALUE!, ours #NUM!
- gen-m9/09e6a8028f5acfae Calc!A150 `=OCT2HEX(TRUE,8)`: Excel #VALUE!, ours #NUM!
- gen-m9/12e4aa13a20cc1f8 Calc!A329 `=HEX2BIN(TRUE,COUNT(Data!D9:D9))`: Excel #VALUE!, ours #NUM!
- gen-m9/2f0d384b8ed0d392 Calc!A68 `=HEX2BIN(TRUE,9)`: Excel #VALUE!, ours #NUM!
- gen-m9/3cf48801617a72ad Calc!A165 `=HEX2DEC(TRUE)`: Excel #VALUE!, ours #NUM!
Cells:
- gen-m9/01810fc89c5ec2c0.case.json (1): Calc!A66
- gen-m9/060b18586fcff6e6.case.json (1): Calc!A380
- gen-m9/09e6a8028f5acfae.case.json (2): Calc!A150, A352
- gen-m9/12e4aa13a20cc1f8.case.json (1): Calc!A329
- gen-m9/2f0d384b8ed0d392.case.json (1): Calc!A68
- gen-m9/3cf48801617a72ad.case.json (1): Calc!A165
- gen-m9/408aa11aa923a4d9.case.json (1): Calc!A360
- gen-m9/493bb194c8e0b577.case.json (4): Calc!A23, A148, A238, A289
- gen-m9/4fffa5addab1f8af.case.json (1): Calc!A385
- gen-m9/516488da78a6b08c.case.json (2): Calc!A124, A129
- gen-m9/59a16b5ab566ee5c.case.json (2): Calc!A119, A305
- gen-m9/5b6cc12a94f7bf14.case.json (2): Calc!A67, A125
- gen-m9/5bdef392d16a5efb.case.json (1): Calc!A36
- gen-m9/62c886e21a1042af.case.json (2): Calc!A114, A293
- gen-m9/65ff9a692665eba9.case.json (1): Calc!A251
- gen-m9/74779cff2810ccf4.case.json (3): Calc!A156, A257, A296
- gen-m9/756a9f7bc3a06bcc.case.json (1): Calc!A258
- gen-m9/76551d6fbafc7007.case.json (1): Calc!A298
- gen-m9/7a819ffbbeff8fcc.case.json (1): Calc!A269
- gen-m9/831f987e63e2412c.case.json (3): Calc!A144, A285, A340
- gen-m9/895a33aa8352864a.case.json (1): Calc!A259
- gen-m9/8cbfd5e99d2e82da.case.json (1): Calc!A265
- gen-m9/8fa8b572ebc1f001.case.json (5): Calc!A15, A63, A80, A190, A379
- gen-m9/a1b4181c04804efc.case.json (2): Calc!A234, A332
- gen-m9/a333e1febf9e93cd.case.json (2): Calc!A242, A389
- gen-m9/a7fda34502ed3030.case.json (1): Calc!A173
- gen-m9/b5aef3645cec1b95.case.json (3): Calc!A82, A273, A302
- gen-m9/bf979a17ab5aafcb.case.json (1): Calc!A120
- gen-m9/ca182d96dedee546.case.json (2): Calc!A150, A310
- gen-m9/cb80ca09be0e12bc.case.json (5): Calc!A57, A104, A161, A179, A268
- gen-m9/df083d85b436b49a.case.json (1): Calc!A161
- gen-m9/e85daedce5c2675e.case.json (1): Calc!A167
- gen-m9/e882060cf4613302.case.json (1): Calc!A123
- gen-m9/ee70a5515d004a66.case.json (1): Calc!A77
- gen-m9/f280df8ba95f96b9.case.json (1): Calc!A294
- gen-m9/f8bc18a0e45a0e1a.case.json (1): Calc!A8
- gen-m9/fb73d8163ad87fc6.case.json (2): Calc!A266, A356

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2258): these 63 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] A probe case holds the cells above, plus neighbours (a leading + on other IM* forms, OCT2HEX/OCT2BIN of 11 digits, a TRUE cell).
- [ ] Ours matches every probe cell; docs/excel-semantics/engineering.md is updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other base-conversion or IM* parsing changes.

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2258): every cell this task owns has the wrong type, error, text or value (none is a last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria states it.
