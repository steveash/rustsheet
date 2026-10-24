---
id: T-2484
title: "DISC, INTRATE, PRICEDISC, PRICEMAT, RECEIVED, YIELDDISC, YIELDMAT and ACCRINTM: argument check order (gen-m9, 87 cells)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
DISC, INTRATE, PRICEDISC, PRICEMAT, RECEIVED, YIELDDISC, YIELDMAT and ACCRINTM give Excel's result when one argument is an error, text or a boolean and another is out of its domain.

## Context
Gap triage of the T-2257 gen-m9 financial sweep. Ours returns a text or error argument's `#VALUE!`/error where Excel returns
`#NUM!` for a settlement after the maturity or a date out of range given earlier (`PRICEDISC(39668,-1,<text>%,106)`, and
`INTRATE(123456789.123," x ",97446,109042)` and `RECEIVED(1E15,44370,123456789.123,TRUE,4)` with those values in Data cells, a
settlement past 9999-12-31 first: `#NUM!`, ours `#VALUE!`), and `#VALUE!` for a text argument where ours checks a later domain first (`INTRATE(38384,45711,95280,
LARGE(..)<#NUM!>,"abc")` is `#VALUE!`). financial.md "Financial function argument modes" says ACCRINTM reads every argument
before it checks a value (`ACCRINTM(39083,39448,-0.1,1000,NA())` is `#N/A`, probes/fin-later-argument-ranges J17); these cells
show the discount-security functions mixing the two orders, so fit one rule over the probe cells and these before changing
the reader, and confirm doubtful positions with a probe (excel-probe skill). docs/excel-semantics/financial.md "Financial functions in the gen-m9 financial sweep (T-2257)" records what the cells show.
About a quarter of the cells nest a statistical or lookup call (RSQ, CORREL, FORECAST.LINEAR, CHITEST, REGEXEXTRACT,
FTEST) in an argument; if a probe shows that call is what differs, move the cell to T-2496 (or the call's own task) and say
so in the Outcome.
Code: crates/rustsheet-engine/src/functions/financial/discount.rs (the shared discount-security reader).

Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 87 known-failure lines in eval/baseline/gen-m9.txt, 87 visibly off.
Examples:
- gen-m9/31474c20fc1dafd6 Calc!A103 `=INTRATE(38384,45711,95280,LARGE(Data!C10:D11,4),Data!A4)`: Excel #VALUE!, ours #NUM!
- gen-m9/3c06f9298f65a3ba Calc!A215 `=DISC(38246,46571,98,RSQ(Data!$C5,(-2.5)))`: Excel #VALUE!, ours #DIV/0!
- gen-m9/79d87fc527927626 Calc!A207 `=INTRATE(Data!B$1,Data!E5,97446,109042)`: Excel #NUM!, ours #VALUE!
- gen-m9/81bcdc0bec0c8e6b Calc!A192 `=RECEIVED(Data!B$2,44370,Data!C10,Data!C3,4)`: Excel #NUM!, ours #VALUE!
- gen-m9/a91bad4e84f49739 Calc!A205 `=YIELDDISC(36584,44569,Data!D$6,109,Data!E8)`: Excel #NUM!, ours #VALUE!
- gen-m9/abb7d565af89b849 Calc!A294 `=PRICEDISC(39668,(-1),(Data!C7*0.3)%,106)`: Excel #NUM!, ours #VALUE!
Cells:
- gen-m9/01523d247594952e.case.json (5): Calc!A9, A54, A208, A220, A362
- gen-m9/0a957119475760c5.case.json (1): Calc!A370
- gen-m9/1b60bf99d71424e7.case.json (4): Calc!A35, A103, A138, A173
- gen-m9/1dce4774f5c0e736.case.json (2): Calc!A178, A332
- gen-m9/23d82b62e9a85bdd.case.json (4): Calc!A31, A139, A208, A330
- gen-m9/31474c20fc1dafd6.case.json (2): Calc!A44, A103
- gen-m9/3c06f9298f65a3ba.case.json (5): Calc!A106, A166, A215, A329, A365
- gen-m9/5422bdbf173fc233.case.json (3): Calc!A55, A162, A399
- gen-m9/5c6a23c4f6597f1c.case.json (6): Calc!A43, A69, A102, A278, A291, A340
- gen-m9/5ded7594e597e31e.case.json (1): Calc!A328
- gen-m9/6376e27be7a05871.case.json (1): Calc!A249
- gen-m9/762cd0e4ec9a7aa9.case.json (2): Calc!A260, A344
- gen-m9/79d87fc527927626.case.json (2): Calc!A207, A230
- gen-m9/7d2979fba2c95d64.case.json (1): Calc!A336
- gen-m9/81bcdc0bec0c8e6b.case.json (3): Calc!A118, A192, A356
- gen-m9/89f0aee953a55a45.case.json (4): Calc!A100, A114, A150, A333
- gen-m9/9a1fa5366bc50918.case.json (2): Calc!A264, A298
- gen-m9/9d9ceba257f8001b.case.json (4): Calc!A48, A123, A270, A332
- gen-m9/a91bad4e84f49739.case.json (6): Calc!A99, A103, A197, A205, A246, A395
- gen-m9/abb7d565af89b849.case.json (3): Calc!A71, A198, A294
- gen-m9/b6a02893a8d6b151.case.json (1): Calc!A394
- gen-m9/b73c6cc96e093f03.case.json (1): Calc!A374
- gen-m9/ba4ddb817a8e700e.case.json (7): Calc!A1, A59, A142, A197, A267, A268, A393
- gen-m9/bc04c1f478c9896c.case.json (2): Calc!A167, A319
- gen-m9/c0681a8b150ce301.case.json (2): Calc!A43, A143
- gen-m9/d0854366830dcdee.case.json (5): Calc!A11, A70, A81, A117, A207
- gen-m9/e168be7f6d3f84c2.case.json (2): Calc!A216, A372
- gen-m9/e8d55ec393460bb7.case.json (6): Calc!A11, A239, A277, A317, A347, A365

T-2436 probe (excel-graph, 2026-10-24; probes/t2436/<case>, each gen-m9 cell's sub-expressions on its Data sheet): `YIELDDISC(9.33,1E+15,N("1.5")<0>,INDEX(Data!B9:C12,-0.0001)<#VALUE!>,5)` is `#NUM!`, the zero price or basis 5 before the redemption's error (ours `#VALUE!`).
Owned gen-m9 cells (T-2436 probe, excel-graph, 2026-10-24): 1 known-failure line in eval/baseline/gen-m9.txt, 1 formula.
Examples:
- gen-m9/2a18b2f5a4dc43ac Calc!A236 `=HYPGEOM.DIST(YIELDDISC(LOG10(2^31-1),MAX(Data!C11,Data!E5:E6,Data!C9:C10),N(Data!B10),INDEX(Data!B9:C12,(-0.0001)),5),AVERAGE(Data!B6:B8),4,15,((Data!D$12>Data!C1)=IMCSC(" x ")))`: Excel #NUM!, ours #VALUE!; inner `YIELDDISC(LOG10(2^31-1),MAX(Data!C11,Data!E5:E6,Data!C9:C10),N(Data!B10),INDEX(Data!B9:C12,(-0.0001)),5)` Excel #NUM!, ours #VALUE!
Cells:
- gen-m9/2a18b2f5a4dc43ac.case.json (1): Calc!A236

From T-1743 (2026-10-24): gen-m9 05d44f6dd6cfc567 Calc!A34 `=GROWTH(DISC(39060,Data!A$4,Data!C6,(Data!D$7/1e-15)),..)` is
`#NUM!` in Excel, `#VALUE!` in ours. GROWTH reports its known_y's error first, so the inner DISC decides: maturity 1E+15
(past 9999-12-31), price "a*" (text) and redemption `"#N/A"/1E-15` (`#VALUE!`); Excel's DISC is `#NUM!` there (the
date first), ours `#VALUE!`. This cell's line goes with this task.
Cells:
- gen-m9/05d44f6dd6cfc567.case.json (1): Calc!A34

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2436): this known-failure line disappears from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity (gen-m9, T-2257): these 87 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Unit tests for the order between a date error, a domain error and a text argument in each function.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The T-bill functions (T-2487).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2257): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.
