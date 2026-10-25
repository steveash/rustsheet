---
id: T-2429
title: "HYPGEOM.DIST and HYPGEOMDIST: a huge count is #NUM!, and the remaining T-2255/T-2256 cells (gen-m9, 26 cells; impossible sample and negative counts cut to T-2699, T-2700)"
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
HYPGEOM.DIST and HYPGEOMDIST return `#NUM!` for a count near 1E15, and Excel's results for the remaining owned cells, as
Excel does. The impossible sample (0) is T-2699's. The negative fractional counts (`#NUM!`) are T-2700's.

## Context
Split 2026-10-25 (gap triage, sonnet refill). The impossible-sample cells (Excel 0 where ours is #NUM!: 11 T-2255 cells and the 2 T-2436 cells) moved to T-2699. The negative fractional counts (7674c89c824aca43 A386, 589f42e67f4b4b8d A2, cde551ea787f713a A72) moved to T-2700. This task keeps the huge counts and their unprobed threshold, and the remaining T-2255 and T-2256 cells. Those include a text or negative cumulative and the lifted or nested calls. The counts below exclude the moved cells.

Gap triage of the T-2255 gen-m9 sweep. statistics.md "HYPGEOMDIST: a sample below the possible range is 0; a negative or
huge count is #NUM! (T-2255)" records it: 29 cells, 11 where Excel is 0 and ours `#NUM!` (sample_s below
number_sample - (number_pop - population_s)), 18 where a negative (down to -5E-20) or huge (about 1E15) count, or a
negative or text cumulative beside one, is `#NUM!` and ours 0 or 1. The lifted HYPGEOM.DIST cells (`#ERROR!` in ours) are
T-2433's. statistics.md "Thin distribution functions" (T-1049, T-2035) already has the truncation and
negative-fraction rules; the threshold for "huge" lies between 202769 (computes, b6593d7dd13f2dac A377) and
753749999999999.8 (`#NUM!`): probe it (e.g. 1E9, 1E12, 2^53) before choosing one.
Code: crates/rustsheet-engine/src/functions/statistical/hypegeom.rs.

Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 15 known-failure lines in eval/baseline/gen-m9.txt, 15 formulas (29 before the T-2699 and T-2700 splits).
Examples:
- gen-m9/05d44f6dd6cfc567 Calc!A54 `=HYPGEOMDIST(SUMIF(Data!B4:B9,"a*",Data!C2:D8),2,8,9)`: Excel 0.0, ours #NUM!
- gen-m9/06b4e8f5de409d5e Calc!A125 `=HYPGEOMDIST((SUM(Data!B$4,Data!D4:D7)+EDATE(61,4)),4,8,20)`: Excel #NUM!, ours 0.0
Cells:
- gen-m9/06b4e8f5de409d5e.case.json (2): Calc!A125, A228
- gen-m9/58e3253bbd3a7a82.case.json (1): Calc!A343
- gen-m9/68d8ffb69ff739d8.case.json (1): Calc!A145
- gen-m9/8d3588d8df0812df.case.json (2): Calc!A16, A199
- gen-m9/a013a45a8fbcab4f.case.json (2): Calc!A314, A352
- gen-m9/a612e89321a6df9c.case.json (1): Calc!A395
- gen-m9/abc2939ebd1e2365.case.json (1): Calc!A347
- gen-m9/cde551ea787f713a.case.json (1): Calc!A254
- gen-m9/eced4bb6a0234c9e.case.json (2): Calc!A336, A390
- gen-m9/ee2497a5e683c5d9.case.json (1): Calc!A264
- gen-m9/f34f66fb7f81e673.case.json (1): Calc!A386

T-2256 sweep (gen-m9 part 2, statistical M to Z; gap triage 2026-10-23): HYPGEOM.DIST and HYPGEOMDIST with huge or negative counts nested in the statistical calls.

Owned gen-m9 cells (T-2256 triage, excel-graph, 2026-10-23): 11 known-failure lines in eval/baseline/gen-m9.txt, 11 visibly off.
Examples:
- gen-m9/139684e68fb2fc3b Calc!A131 `=SLOPE(Data!B1:D11,(HYPGEOM.DIST(Data!A9,1,1,1E+15,Data!D2)*Data!B9))`: Excel #NUM!, ours #N/A
- gen-m9/1db18890916ee643 Calc!A23 `=RSQ(HYPGEOM.DIST((Data!E2+(-1)),5,2,2^31,("ü">Data!C$6)),Data!A5:B9)`: Excel #NUM!, ours #N/A
- gen-m9/38e4fc309fe199ea Calc!A5 `=QUARTILE.EXC(Data!D7,HYPGEOMDIST(2,3,7,2^32))`: Excel #NUM!, ours #VALUE!
- gen-m9/38e4fc309fe199ea Calc!A376 `=SLOPE(Data!A7:D10,HYPGEOMDIST(1,1E+15,5,LAMBDA(acc,w,acc)(Data!D7,Data!$C$10)))`: Excel #NUM!, ours #VALUE!
- gen-m9/44a8841352dc48e9 Calc!A146 `=PROB((STDEV(Data!A8:A11)^HYPGEOM.DIST(4,5,10,15,0.5)),Data!B11:B12,0.2)`: Excel #N/A, ours -0.0
Cells:
- gen-m9/11eb3ab4f4f8104c.case.json (1): Calc!A10
- gen-m9/139684e68fb2fc3b.case.json (1): Calc!A131
- gen-m9/1db18890916ee643.case.json (1): Calc!A23
- gen-m9/38e4fc309fe199ea.case.json (2): Calc!A5, A376
- gen-m9/44a8841352dc48e9.case.json (1): Calc!A146
- gen-m9/5395b8a427764a3f.case.json (1): Calc!A302
- gen-m9/73f053591a54bc43.case.json (1): Calc!A387
- gen-m9/941c409532603d32.case.json (1): Calc!A138
- gen-m9/a4090567758520df.case.json (1): Calc!A396
- gen-m9/afc00c8b70e78baf.case.json (1): Calc!A135

T-2436 probe (excel-graph, 2026-10-24; probes/t2436/<case>, each gen-m9 cell's sub-expressions on its Data sheet): the impossible sample is 0 for the cumulative form too: `HYPGEOMDIST(2,5,9,11)` and `HYPGEOM.DIST(1,3,6,7,TRUE)` are 0 (ours `#NUM!`), so the enclosing CHIDIST and KURT/YEAR give Excel's values.
Owned gen-m9 cells (T-2436 probe, excel-graph, 2026-10-24): none left; both moved to T-2699.
Examples:
- gen-m9/0a9226016e8f5f33 Calc!A26 `=CHIDIST(TIME(11,HYPGEOMDIST(2,5,9,11),1),9)`: Excel 0.9999790730352063, ours #NUM!; inner `HYPGEOMDIST(2,5,9,11)` Excel 0.0, ours #NUM!
- gen-m9/35e8168aae2b00f8 Calc!A271 `=KURT(Data!D10:D11,YEAR(HYPGEOM.DIST(1,3,6,7,TRUE)))`: Excel #DIV/0!, ours #NUM!; inner `HYPGEOM.DIST(1,3,6,7,TRUE)` Excel 0.0, ours #NUM!
(Both cells moved to T-2699.)

## Acceptance criteria
- [x] Visible parity (gen-m9, T-2255 and T-2256; 24 of 26: 21 gone, 3 already passing on main, 2 moved, see Outcome): these 26 known-failure lines (15 from T-2255, 11 from T-2256) disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] Unit tests: `HYPGEOMDIST(1E15,2,2,17)` is `#NUM!`. The impossible sample and the negative fractions are tested in T-2699 and T-2700.
- [x] statistics.md's entry gets an "Ours:" line and the probed threshold.
- [x] `make check` green; reviewers approve.

## Out of scope
- HYPGEOM.DIST's last digits (T-0977) and large-population cdf (T-1896).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2255): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.

## Outcome
- Probed the threshold (probes/t2429-hypgeom-huge-count, probes/t2429-hypgeom-threshold, excel-graph 2026-10-25): a count
  of 2^31 - 1 or more in any of the four arguments is `#NUM!` (2147483646.9 computes), the integer-count limit
  `limits::COUNT_LIMIT_EXCLUSIVE` that PERMUT, COMBIN and BINOM.DIST already use. Every argument is read first (a text
  cumulative is `#VALUE!` beside a huge count). probes/t2429/<case> traces each owned gen-m9 cell's HYPGEOM arguments.
- hypegeom.rs `hyp_geom_dist_of` now checks every count for a negative fraction (before truncation) and the limit. That
  also lands T-2700's HYPGEOM part (its 3 HYPGEOM cells pass).
- gen-m9/38e4fc309fe199ea A5 needed QUARTILE.EXC of a single cell holding TRUE to be no number (`#NUM!`), not `#VALUE!`
  (probes/t2429-quartile-exc-no-number): `percentile_numbers` now reads a boolean cell as it reads a text cell. That fixed
  24 of T-2344's cells too.
- Two owned cells are not this kernel's: eced4bb6a0234c9e A390 (the inner `NPER(-1,..)` is Excel `#NUM!`, ours 0) moved to
  T-2698; 11eb3ab4f4f8104c A10 (`#SPILL!` vs ours `#ERROR!` over a GROUPBY lift) is noted in T-2433. The task's three
  T-2256 cells 38e4fc309fe199ea A376, 44a8841352dc48e9 A146 and 941c409532603d32 A138 already passed on main.
- The capped-cdf DEVIATION's example moved below the limit (`HYPGEOM.DIST(537370912,1073741823,1073741823,2147483646,TRUE)`);
  the subnormal-window bench needed a population past the limit and is gone from benches/lift.rs.
- Eval delta: gen-m9 known failures 4324 -> 4271 (-53: 21 owned, 3 of T-2700, 24 of T-2344, 1 of T-2495 (67a4c03afd63f988
  A352), 1 each of T-2436, T-2472, T-2492 and T-2496's lists); probes 2615 -> 2639 (+39 in the new probe cases: the
  impossible sample of T-2699, large-population last digits of T-0977/T-1896, the NPER and GROUPBY traces; -15 fixed
  in t2436, t2492 and tbill-error-order probes); no regressions.
