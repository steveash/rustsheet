---
id: T-2429
title: "HYPGEOM.DIST and HYPGEOMDIST: an impossible sample is 0, a negative or huge count is #NUM! (gen-m9, 30 cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
HYPGEOM.DIST and HYPGEOMDIST return 0 for a sample below the smallest possible count, and `#NUM!` for any negative count
and for a count near 1E15, as Excel does.

## Context
Gap triage of the T-2255 gen-m9 sweep. statistics.md "HYPGEOMDIST: a sample below the possible range is 0; a negative or
huge count is #NUM! (T-2255)" records it: 30 cells, 15 where Excel is 0 and ours `#NUM!` (sample_s below
number_sample - (number_pop - population_s)), 12 where a negative (down to -5E-20) or huge (about 1E15) count is `#NUM!`
and ours 0 or 1, 3 where a PIVOTBY or MAP argument lifts (Excel `#SPILL!`, ours `#ERROR!` from the lift budget: those
need T-2433 too). statistics.md "Thin distribution functions" (T-1049, T-2035) already has the truncation and
negative-fraction rules; the threshold for "huge" lies between 202769 (computes, b6593d7dd13f2dac A377) and
753749999999999.8 (`#NUM!`): probe it (e.g. 1E9, 1E12, 2^53) before choosing one.
Code: crates/rustsheet-engine/src/functions/statistical/hypegeom.rs.

Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 30 known-failure lines in eval/baseline/gen-m9.txt, 30 formulas.
Examples:
- gen-m9/05d44f6dd6cfc567 Calc!A54 `=HYPGEOMDIST(SUMIF(Data!B4:B9,"a*",Data!C2:D8),2,8,9)`: Excel 0.0, ours #NUM!
- gen-m9/06b4e8f5de409d5e Calc!A125 `=HYPGEOMDIST((SUM(Data!B$4,Data!D4:D7)+EDATE(61,4)),4,8,20)`: Excel #NUM!, ours 0.0
- gen-m9/530c636749a85209 Calc!A338 `=HYPGEOM.DIST(IRR(Data!C2:D3,BITXOR(235,Data!E4)),3,2,19,NOT((1000000000000000.0<=Data!A7)))`: Excel #DIV/0!, ours 1.0
Cells:
- gen-m9/05d44f6dd6cfc567.case.json (1): Calc!A54
- gen-m9/06b4e8f5de409d5e.case.json (2): Calc!A125, A228
- gen-m9/4250f880ab640c49.case.json (1): Calc!A339
- gen-m9/4bad303d32cc3f46.case.json (1): Calc!A267
- gen-m9/4e55500bb8ad3cd8.case.json (1): Calc!A318
- gen-m9/530c636749a85209.case.json (1): Calc!A338
- gen-m9/589f42e67f4b4b8d.case.json (1): Calc!A2
- gen-m9/58e3253bbd3a7a82.case.json (1): Calc!A343
- gen-m9/68d8ffb69ff739d8.case.json (1): Calc!A145
- gen-m9/7674c89c824aca43.case.json (1): Calc!A386
- gen-m9/8d3588d8df0812df.case.json (2): Calc!A16, A199
- gen-m9/8f2ac25945af87da.case.json (1): Calc!A218
- gen-m9/9cca17438a7e6f8a.case.json (2): Calc!A96, A340
- gen-m9/a013a45a8fbcab4f.case.json (2): Calc!A314, A352
- gen-m9/a612e89321a6df9c.case.json (1): Calc!A395
- gen-m9/a7bf0d8d81d33ba6.case.json (1): Calc!A315
- gen-m9/abc2939ebd1e2365.case.json (1): Calc!A347
- gen-m9/c2974c061384fdd9.case.json (1): Calc!A81
- gen-m9/c468f5046aadfbbc.case.json (1): Calc!A85
- gen-m9/c6cfda6bd10cae93.case.json (1): Calc!A31
- gen-m9/cde551ea787f713a.case.json (2): Calc!A72, A254
- gen-m9/eced4bb6a0234c9e.case.json (2): Calc!A336, A390
- gen-m9/ee2497a5e683c5d9.case.json (1): Calc!A264
- gen-m9/f34f66fb7f81e673.case.json (1): Calc!A386

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2255): these 30 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Unit tests: `HYPGEOMDIST(3,5,5,6)` is 0, `HYPGEOMDIST(3,-0.025,5,9)` and `HYPGEOMDIST(1E15,2,2,17)` are `#NUM!`.
- [ ] statistics.md's entry gets an "Ours:" line and the probed threshold.
- [ ] `make check` green; reviewers approve.

## Out of scope
- HYPGEOM.DIST's last digits (T-0977) and large-population cdf (T-1896).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2255): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.
