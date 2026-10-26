---
id: T-2728
title: "IRR of CELL's row or col is #VALUE!, IRR's #REF! guess error, XIRR at a huge guess (gen-m9, 3 cells)"
milestone: M9
lane: functions-financial
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
IRR and XIRR return Excel's value or error for three gen-m9 cells cut from T-2494.

## Context
Cut from T-2494 on 2026-10-25. Probes (excel-graph, 2026-10-25):
- CELL: `IRR(CELL("row",D2))`, `IRR(CELL("row",D2),0.5)`, `IRR(CELL("col",D2))` and `IRR(LET(x,CELL("row",D2),x))` are
  `#VALUE!` (ours `#NUM!`), while `IRR(CELL("contents",D2))`, `IRR(CELL("address",D2))`, `IRR(CELL("row",D2)+0)`,
  `IRR(N(CELL("row",D2)))`, `IRR(ROW(D2))`, `IRR(COLUMN(D2))`, `IRR(D2)`, `IRR(5)` and `IRR({7})` are `#NUM!` as ours;
  `TYPE(CELL("row",D2))` is 1 and `NPV(0.1,CELL("row",D2))` 1.8181818181818181 (probes/irr-guess-edges A20-A26,
  irr-guess-edges-2 A35-A44). So CELL's row and col results reach IRR as something other than a plain number; probe what
  (e.g. ISREF, ROWS, an array-taking function) before implementing.
- `IRR((YEAR(Data!D$11)+UNIQUE(Data!B2:E10,TRUE,TRUE)),INDEX(Data!A3:B9,2))` is `#REF!` (ours `#VALUE!`): probe the guess
  `INDEX(Data!A3:B9,2)` (a row reference) and the order between a flows error and a guess error, including a flows
  error with a guess at or above 2^53 (`IRR(1/0,1E+300)`: ours `#DIV/0!`; financial.md "IRR's guess range").
- XIRR: `XIRR({-100,10,20,30,40},{0,1,59,60,61},G)` is 3.4266142457661796E-09 for G = 123456789.123, 4.336808689942018E-09
  for 1E+10 and 3.637978807091713E-09 for 1000 (ours `#NUM!`), `#NUM!` for 1E+300 as ours, and 2.9802322387695314E-09 with
  no guess (ours -2.9802322387695314E-09, the sign) (probes/irr-guess-edges A41-A47). The root is exactly 0; read
  financial.md "XIRR is a bracketing bisection".
Code: `fn_irr`, `fn_xirr` in crates/rustsheet-engine/src/functions/financial/mod.rs; `compute_xirr` in financial_util.rs.

Owned gen-m9 cells: 3 known-failure lines in eval/baseline/gen-m9.txt.
- gen-m9/5422bdbf173fc233 Calc!A63 `=IRR(CELL("row",Data!D$2),0.5)`: Excel #VALUE!, ours #NUM!
- gen-m9/5c6a23c4f6597f1c Calc!A153 `=IRR((YEAR(Data!D$11)+UNIQUE(Data!B2:E10,TRUE,TRUE)),INDEX(Data!A3:B9,2))`: Excel #REF!, ours #VALUE!
- gen-m9/a91bad4e84f49739 Calc!A350 `=XIRR({-100,10,20,30,40},{0,1,59,60,61},MAX(Data!E4:E8))`: Excel 3.4266142457661796E-09, ours #NUM!

## Acceptance criteria
- [ ] Visible parity (gen-m9): these 3 known-failure lines disappear from eval/baseline/gen-m9.txt (`make eval-baseline`), with
      no new known-failure lines in any baseline.
- [ ] Unit tests for each rule; financial.md records them with probe evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
- IRR's no-root cells (T-2727).

## Visible-parity stage
Decision 0083: every cell this task owns has the wrong type, error or value, so the task is in the campaign.
