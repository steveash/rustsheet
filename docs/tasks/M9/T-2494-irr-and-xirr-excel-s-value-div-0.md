---
id: T-2494
title: "IRR: a guess at or above 2^53 is #VALUE! (gen-m9, 5 cells; the rest cut to T-2727 and T-2728)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: M
tier: opus
depends_on: []
campaign: semantics
---

## Goal
An IRR guess at or above 2^53 is Excel's `#VALUE!`. (Narrowed on 2026-10-25: the task first covered 22 IRR/XIRR cells; the
no-root and `#DIV/0!` cells moved to T-2727, the CELL, `#REF!` and XIRR cells to T-2728, and 3 cells, 3aa168259335be54 A1,
5ded7594e597e31e A282 and d912671baa1364aa A158, already passed on main.)

## Context
Gap triage of the T-2257 gen-m9 financial sweep: a guess of 1E+300 (or 2.5E+29) is `#VALUE!` in Excel, where ours ran the
solver. Probes/irr-guess-edges and irr-guess-edges-2 (excel-graph, 2026-10-25) put the bound at 2^53: 2^53-1 and 9E+15
converge, 2^53 and 1E+16 are `#VALUE!`. docs/excel-semantics/financial.md "IRR's guess range".
Code: `compute_irr` in crates/rustsheet-engine/src/functions/financial/financial_util.rs.

Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 5 known-failure lines in eval/baseline/gen-m9.txt.
- gen-m9/1b60bf99d71424e7 Calc!A215 `=IRR(Data!E4:E8,1E+300)`: Excel #VALUE!, ours #NUM!
- gen-m9/6376e27be7a05871 Calc!A9 `=IRR(COUNTBLANK(Data!C9:C12),1E+300)`: Excel #VALUE!, ours #NUM!
- gen-m9/79d87fc527927626 Calc!A26 `=IRR(Data!C5:D6,1E+300)`: Excel #VALUE!, ours 30.126729201490246
- gen-m9/b6a02893a8d6b151 Calc!A236 `=IRR(Data!B8:C8,VARPA(Data!A9:A10))`: Excel #VALUE!
- gen-m9/e168be7f6d3f84c2 Calc!A395 `=IRR({0,0,-100,50,60},1E+300)`: Excel #VALUE!, ours 0.0639

## Acceptance criteria
- [x] Visible parity (gen-m9): the 5 guess cells (1b60bf99d71424e7 A215, 6376e27be7a05871 A9, 79d87fc527927626 A26,
      b6a02893a8d6b151 A236, e168be7f6d3f84c2 A395) disappear from eval/baseline/gen-m9.txt, with no new known-failure lines.
      (The T-2436 cell and the other T-2257 cells are T-2727's and T-2728's.)
- [x] Unit tests for a huge guess; scalar flows stay `#NUM!`. (A63's and A56's causes: T-2728, T-2727.)
- [x] `make check` green; reviewers approve.

## Out of scope
- IRR's returned digits (T-2131) and IRR over an array with an error element (T-2390).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2257): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.

## Outcome
Probed (excel-graph, 2026-10-25) in probes/irr-guess-edges (47 cells) and irr-guess-edges-2 (44 cells): an IRR guess at or
above 2^53 is `#VALUE!` (2^53-1 and 9E+15 converge), before the flows are read. `compute_irr` checks it with the existing
guess <= -1 check; financial.md "IRR's guess range". The probes' other findings (no root for far roots, `#DIV/0!` at a root
near 1E+15, CELL's row and col, XIRR at huge guesses) are recorded in the new tasks T-2727 and T-2728, which own the
remaining 14 cells. Eval: gen-m9 known failures 4345 -> 4340 (5 fixed); probes +30 known-failure lines, all in the
two new probe cases (`make eval-baseline`), owned by T-2727 and T-2728; 0 regressions.
