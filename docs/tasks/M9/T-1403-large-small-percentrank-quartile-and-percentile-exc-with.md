---
id: T-1403
title: "LARGE, SMALL, PERCENTRANK, QUARTILE and PERCENTILE.EXC with a scalar, error or single-cell data argument give Excel's error"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
LARGE, SMALL, PERCENTRANK, PERCENTRANK.EXC, QUARTILE and PERCENTILE.EXC give Excel's result when their data argument is a
scalar error, a single cell or holds no number, and report the k argument's error before an empty-data `#NUM!`.

## Context
T-1289's gen-m9 suite (excel-graph sidecars, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/1782efcecd5d4b1b Calc!A25 `=LARGE(XLOOKUP(Data!E3,Data!A6:A10,Data!A3:A12),((100*Data!$E1)-0))`: Excel `#VALUE!`
  (XLOOKUP's size mismatch), engine `#N/IMPL!` (`LARGE(#N/A,1)` and `SMALL(#VALUE!,1)` are `#N/IMPL!`).
- gen-m9/95efb37a9874f82e Calc!A2 `=LARGE(PERCENTILE(Data!B8:D8,HLOOKUP(Data!B2,Data!C6:C6,Data!$E9)),0.1)`: Excel `#REF!`
  (the HLOOKUP index error through PERCENTILE), engine `#N/IMPL!`.
- gen-m9/b5352ebdb9c5f1a1 Calc!A24 `=PERCENTRANK((VLOOKUP(..)+NETWORKDAYS(..)),100)` (the data argument is `#N/A`): Excel
  `#N/A`, engine `#VALUE!`.
- gen-m9/288b99797f58f149 Calc!A3 `=(BETAINV(Data!$B$7,QUARTILE(Data!$D6,Data!D5),Data!B5)+..)` (D6 = " x ", quart 7):
  Excel `#NUM!`, engine `#VALUE!` (`QUARTILE(Data!$D6,1)` is `#VALUE!` too: a single text cell is rejected). This cell does
  not pin QUARTILE down: BETAINV's probability B7 = 123456789.123 is invalid and can give `#NUM!` by itself. Probe
  `QUARTILE(<cell " x ">,7)` and `QUARTILE(<cell " x ">,1)` with the excel-probe skill before changing QUARTILE.
- gen-m9/d5bd4272655c5211 Calc!A31 `=PERCENTILE.EXC(Data!A8:B8,((Data!C4-Data!B4)^123456789.123%))` (A8:B8 = FALSE, "ü";
  k is `#VALUE!`): Excel `#VALUE!`, engine `#NUM!` (empty data checked before k).
- gen-m9/65faaeb48419087d Calc!A17 `=IMSIN(VLOOKUP(PERCENTRANK.EXC(1.005,Data!B$11,1),Data!A4:B9,Data!B4,(Data!D3=Data!B7)))`
  (B11 = 1.005): Excel `#N/A`, engine `#VALUE!`. The engine's PERCENTRANK.EXC is 0.5, which VLOOKUP finds (A4 = 0.5) and
  then rejects the blank index B4 with `#VALUE!`; Excel's `#N/A` means PERCENTRANK.EXC of a single value is not 0.5,
  inferred `#N/A`. Probe `PERCENTRANK.EXC({1.005},1.005)`, `PERCENTRANK.EXC(B11,1.005)` and `PERCENTRANK.INC` of the same
  with the excel-probe skill before changing it.
Code: crates/rustsheet-engine/src/functions/statistical/count_and_average.rs (`fn_large`, `fn_small`) and percentile.rs.

Semantics: docs/excel-semantics/statistics.md "Array arguments given a scalar, a single cell or an error".

## Acceptance criteria
- [ ] The 6 gen-m9 cells above pass in `./target/release/rustsheet-eval show gen-m9/<case>.case.json`, with no new failing gen-m9 cell and no new known failures in eval/baseline/.
- [ ] The PERCENTRANK.EXC single-value probe cells pass; the rule is in docs/excel-semantics/statistics.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
PERCENTILE.EXC now reports k's error (and a non-numeric k's `#VALUE!`) before the empty-data `#NUM!`
(gen-m9/d5bd4272655c5211 A31 passes). The other gen-m9 cells listed (A25, A2, A24, A17) already passed on main.
Not done: gen-m9/288b99797f58f149 A3 (`QUARTILE(<cell " x ">,7)`) is undecided by the evidence (BETAINV's invalid
probability can give `#NUM!` alone) and needs an oracle probe, which this run did not have; follow-up filed below.
