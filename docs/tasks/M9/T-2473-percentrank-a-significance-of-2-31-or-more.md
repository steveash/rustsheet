---
id: T-2473
title: "PERCENTRANK: a significance of 2^31 or more is #DIV/0!, and its truncation keeps Excel's digit at a cut (gen-m9, 11 cells)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
PERCENTRANK, PERCENTRANK.INC and PERCENTRANK.EXC give `#DIV/0!` for a significance of 2^31 or more and Excel's truncated
digit where the double rank sits just below a cut (0.5 not 0.499, 0.36 not 0.35).

## Context
Gap triage of the T-2256 gen-m9 sweep. statistics.md "PERCENTRANK: a huge significance is #DIV/0!; truncation does not lose
a digit at a cut (T-2256)". Four cells have a huge significance (2^31, 5.6E+12, 1E+15, 9.2E+36); find the cut from these and
existing probes, or file a probe for it. Five cells (all visibly off: 0.5 against 0.499) show Excel's truncation snapping the
rank before the floor; candidates are a round to 15 significant digits before the floor, or the floor of rank*10^d plus a
tolerance. Two more cells are last-digit differences of the untruncated rank (`PERCENTRANK.INC(..,-0.025,250.2)`, 250
digits) and fall out of the same fit. A candidate must keep every passing PERCENTRANK cell in eval/ passing.
Code: the PERCENTRANK kernel in crates/rustsheet-engine/src/functions/statistical/percentile.rs.

Owned gen-m9 cells (T-2256 triage, excel-graph, 2026-10-23): 11 known-failure lines in eval/baseline/gen-m9.txt, 9 visibly off.
Examples:
- gen-m9/7fd0a98183c6babf Calc!A202 `=PERCENTRANK.INC(Data!B10:D11,1,(ROUNDUP(2^31,4)-Data!A4))`: Excel #DIV/0!, ours #NUM!
- gen-m9/91e167cd8d7eb652 Calc!A9 `=PERCENTRANK(Data!D2:E5,TRUNC(ERF.PRECISE(7)),Data!B2)`: Excel #DIV/0!, ours #NUM!
- gen-m9/c7664e494e418395 Calc!A252 `=PERCENTRANK.EXC({1E+15,1,-1E+15},Data!$A$5)`: Excel 0.5, ours 0.499
- gen-m9/d37313406b9f6e55 Calc!A303 `=PERCENTRANK.INC(Data!D4:E12,ROUNDDOWN(2.2250738585072E-308,10))`: Excel 0.5, ours 0.499
- gen-m9/d8f44d556674c2fd Calc!A328 `=PERCENTRANK.EXC({1E+15,1,-1E+15},IFNA(#N/A,(0.2-Data!C9)))`: Excel 0.5, ours 0.499
Cells:
- gen-m9/0620505280d2ea6e.case.json (1): Calc!A21
- gen-m9/2138b909d8f810ea.case.json (1): Calc!A333
- gen-m9/7fd0a98183c6babf.case.json (1): Calc!A202
- gen-m9/91e167cd8d7eb652.case.json (1): Calc!A9
- gen-m9/941c409532603d32.case.json (1): Calc!A168
- gen-m9/c7664e494e418395.case.json (1): Calc!A252
- gen-m9/d37313406b9f6e55.case.json (1): Calc!A303
- gen-m9/d5fd9e05e9e6f466.case.json (1): Calc!A185
- gen-m9/d8f44d556674c2fd.case.json (1): Calc!A328
- gen-m9/ede0323c389fc703.case.json (2): Calc!A43, A157

## Acceptance criteria
- [x] Visible parity (gen-m9, T-2256): these 11 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] Unit tests: `PERCENTRANK.EXC({1E+15,1,-1E+15},0.3)` is 0.5, `PERCENTRANK.INC(A1:A3,1,2147483648)` `#DIV/0!`.
- [x] `make check` green; reviewers approve.

## Out of scope
- The argument order (T-2472).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2256): nine of its eleven cells are visibly off (0.5 against 0.499, `#DIV/0!` against `#NUM!`), so the task is in the campaign. Its target is that every cell it owns
has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value; the visible-parity
criterion under Acceptance criteria states it.

## Outcome
- Probed (probes/t2473-percentrank-significance, excel-graph 2026-10-25): a significance of 309 or more (10^sig overflows)
  is `#DIV/0!`, 308 computes; the truncation is floor(cut15(rank * 10^sig)) / 10^sig with 10^sig correctly rounded.
- `percentrank_of` does that, with FLOOR's 15-digit cut moved into `decimal_round::cut_to_15_digits` (shared with
  ceiling_floor.rs `snap_ratio`).
- Eval delta: gen-m9 known failures -12 (the 11 owned cells and bbab6562e88b70e8 A290), probes -3 (t2475-stat-subexpressions
  P!A449, A452, A455); the new probe case passes 69/69; no regressions.
