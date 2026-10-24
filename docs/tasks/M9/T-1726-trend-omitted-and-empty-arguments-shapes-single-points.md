---
id: T-1726
title: "TREND: omitted and empty arguments, shapes, single points, text and error inputs as Excel (T-1620 probes)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: M
tier: opus
campaign: visible
depends_on: []
---

## Goal
TREND matches Excel for omitted and empty arguments, shapes, single points, text, blanks and error inputs.

## Context
Found by T-1620 (probes/regression-trend, ROWS/COLUMNS/SUM/INDEX of each TREND call in columns L-O; `./target/release/rustsheet-eval show probes/regression-trend.case.json`, excel-graph 2026-10-16). Highlights (ours in parentheses):
- empty arguments: `TREND(A1:A6,,K1:K2)` has 2 rows (`#REF!`); `TREND(A1:A6,B1:B6,,FALSE)` and `,,TRUE` return 6 rows, the fit to the known x's (0 rows; `TREND(A1:A6,,,FALSE)` `#REF!`).
- array constants: `TREND({2,4,5},{1,2,3})` 1 row, 3 columns, first 2.1666666666666665 (`#REF!`); `TREND({2,4,5,4,5,7},{1,2,3,4,5,6},{7,8})` 1x2, first 7.2 (`#REF!`).
- a single point: `TREND(A1:A1,B1:B1)`, `TREND(5,5)` and `TREND(5)` are 2.0, 5.0, 5.0 (`#VALUE!`).
- zero-variance x: `TREND(A1:A6,G1:G6)` 6 rows with first 4.5 (`#NUM!`).
- text, blank or error in a range: `#VALUE!` (a blank known_y `D1:D6` or known_x gives numbers in ours; an error cell gives `#DIV/0!`), a text scalar known_x is `#REF!` (ours `#VALUE!`).
- 2-D known_y (`TREND(A1:B3,H1:I3)`): 3x2 result, first 3.6666666665000003 (`#REF!`).
Group the work by cause; record the rules in docs/excel-semantics/statistics.md.

Overlaps T-1402 (single-value known_y, `#REF!` before `#VALUE!`). Also: an error cell in new_x is `#VALUE!` for that element only (`TREND(A1:A6,B1:B6,F1:F2)` keeps 2 rows, first 3.3428571428571425), unequal sizes are `#REF!`, zero-variance x returns the mean.

Also: a blank new_x makes the whole result `#VALUE!` (ours 2 rows of numbers, trend L39) and a row-shaped new_x gives a row result (`TREND(A1:A6,B1:B6,{7,8})` 1x2, SUM 15.17142857142857; ours 1 column).

Visibly-off cells (gap triage 2026-10-21, decision 0083): 78 cells, 8 numeric past half a unit in the 15th digit and 70 with a wrong type, error or text, blamed on TREND 78, COLUMNS 20, ROWS 19.
- probes/regression-trend (78): Sheet1!L12, Sheet1!L13, Sheet1!L14, Sheet1!L15, Sheet1!L16, Sheet1!L20, Sheet1!L21, Sheet1!L25, Sheet1!L26, Sheet1!L28, Sheet1!L29, Sheet1!L30, Sheet1!L31, Sheet1!L33, Sheet1!L34, Sheet1!L35, Sheet1!L37, Sheet1!L39, Sheet1!L43, Sheet1!M6, Sheet1!M12, Sheet1!M13, Sheet1!M14, Sheet1!M15, Sheet1!M16, Sheet1!M20, Sheet1!M21, Sheet1!M25, Sheet1!M26, Sheet1!M28, Sheet1!M29, Sheet1!M30, Sheet1!M31, Sheet1!M33, Sheet1!M34, Sheet1!M35, Sheet1!M37, Sheet1!M39, Sheet1!M43, Sheet1!N6, Sheet1!N12, Sheet1!N13, Sheet1!N14, Sheet1!N15, Sheet1!N16, Sheet1!N20, Sheet1!N21, Sheet1!N25, Sheet1!N26, Sheet1!N28, Sheet1!N29, Sheet1!N30, Sheet1!N31, Sheet1!N33, Sheet1!N34, Sheet1!N35, Sheet1!N37, Sheet1!N39, Sheet1!N43, Sheet1!O12, Sheet1!O13, Sheet1!O14, Sheet1!O15, Sheet1!O16, Sheet1!O20, Sheet1!O21, Sheet1!O25, Sheet1!O26, Sheet1!O28, Sheet1!O29, Sheet1!O30, Sheet1!O31, Sheet1!O33, Sheet1!O34, Sheet1!O35, Sheet1!O37, Sheet1!O39, Sheet1!O43

gen-m9 sweep (gap triage 2026-10-23, T-2255). Two cells where a TREND inside another call diverges first (the TREND rules
above).
Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 2 known-failure lines in eval/baseline/gen-m9.txt, 2 formulas.
Examples:
- gen-m9/165db58d1540f014 Calc!A371 `=FTEST(IF((Data!C1<Data!D8),Data!$C$4,TREND(Data!C9,Data!$B$1)),Data!B3:C12)`: Excel #DIV/0!, ours #VALUE!
- gen-m9/165db58d1540f014 Calc!A91 `=KURT(TREND(Data!B10:B10,Data!C8),Data!A6:A10)`: Excel #VALUE!, ours #REF!
Cells:
- gen-m9/165db58d1540f014.case.json (2): Calc!A91, A371

T-2256 sweep (gen-m9 part 2, statistical M to Z; gap triage 2026-10-23): TREND errors and shapes: a range known_y alone is `#VALUE!` where ours spills (`TREND(Data!D1:E2)`), a text known_x `#REF!` (`TREND(Data!C4,Data!A3:D12)`), and the first error among the arguments.

Owned gen-m9 cells (T-2256 triage, excel-graph, 2026-10-23): 47 known-failure lines in eval/baseline/gen-m9.txt, 47 visibly off.
Examples:
- gen-m9/46ac3ef5e61583ca Calc!A341 `=TREND(Data!A1:A2)`: Excel #VALUE!, ours #SPILL!
Cells:
- gen-m9/0620505280d2ea6e.case.json (1): Calc!A357
- gen-m9/0bbab6206b5c9a07.case.json (1): Calc!A44
- gen-m9/11eb3ab4f4f8104c.case.json (1): Calc!A326
- gen-m9/329b177b5d5165ad.case.json (1): Calc!A350
- gen-m9/3b7990bd4e3307ba.case.json (2): Calc!A97, A345
- gen-m9/41248d1cb153c66e.case.json (2): Calc!A54, A116
- gen-m9/46ac3ef5e61583ca.case.json (1): Calc!A341
- gen-m9/5389dd4d5b5739d2.case.json (2): Calc!A61, A247
- gen-m9/5395b8a427764a3f.case.json (1): Calc!A348
- gen-m9/583ca2baea0bb30a.case.json (2): Calc!A25, A335
- gen-m9/5be29fa316931f05.case.json (1): Calc!A399
- gen-m9/6e18550e9fc9c1b9.case.json (1): Calc!A34
- gen-m9/73f053591a54bc43.case.json (2): Calc!A53, A155
- gen-m9/80586b859b73c87d.case.json (5): Calc!A61, A214, A265, A299, A384
- gen-m9/91e167cd8d7eb652.case.json (1): Calc!A220
- gen-m9/a2262ea466fcc142.case.json (1): Calc!A338
- gen-m9/a4090567758520df.case.json (1): Calc!A333
- gen-m9/acac71a65a5a1d1b.case.json (1): Calc!A257
- gen-m9/afc00c8b70e78baf.case.json (1): Calc!A277
- gen-m9/bbab6562e88b70e8.case.json (6): Calc!A13, A47, A200, A336, A353, A387
- gen-m9/bcb1105c61a2649d.case.json (3): Calc!A53, A239, A301
- gen-m9/c2684066d8df263e.case.json (2): Calc!A62, A124
- gen-m9/d5fd9e05e9e6f466.case.json (1): Calc!A223
- gen-m9/d910f000b60e2eac.case.json (1): Calc!A60
- gen-m9/eedd629f3d69fdda.case.json (5): Calc!A75, A92, A143, A177, A330
- gen-m9/f82473275380882e.case.json (1): Calc!A224

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2255 and T-2256): these 49 known-failure lines (2 from T-2255, 47 from T-2256) disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity: the 78 visibly-off cells listed under Context (2026-10-21) have Excel's value to 15 significant digits, or its type, error and text: the 78 lines in eval/baseline/probes.txt disappear (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The mismatching non-ULP cells of probes/regression-trend pass (or the remaining ones are listed in an Outcome note) and the baseline shrinks.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Probing further inputs (file a probe task instead).
- The ulp cells of probes/regression-trend: they come from the shared least-squares solve, which T-2006 fits (gap
  triage 2026-10-19).

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (wrong TREND shapes, errors and sums), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it.

## Outcome
TREND and GROWTH now share `trend_or_growth` (functions/statistical/trend.rs) on top of a new element-wise reader
(`read_data`, functions/statistical/linest.rs; LINEST and LOGEST read through it unchanged): empty known_x/new_x are the
defaults, Excel's three shape layouts (same shape element-wise, column y, row y) with new_x's shape deciding the result,
collinear x columns dropped on a singular fit (zero-variance x gives the mean), blank/text/boolean/error cells `#VALUE!`
(an error in new_x only for its own element), the argument-error, typed-text-const, shape, data, new_x order from the
gen-m9 cells, GROWTH's row-major first-offender sign rule, and lifting over an array const (`LiftOptions::
REUSE_ANY_EQUAL_TOP_LEFT`). Rules in statistics.md "TREND: defaults, shapes, single points and error inputs".
Eval delta: probes 1901 -> 1805 known failures (all 78 listed regression-trend lines, plus 26 growth-modes lines of
T-1743), gen-m9 7821 -> 7696 (48 of the 49 owned lines plus 77 other TREND/GROWTH cells), no regressions; visible parity
7562 off (lock 8632). Left: gen-m9 bcb1105c61a2649d A301 (`BESSELI(1E+15,5)` is 0 in ours, `#NUM!` in Excel;
engineering.md, T-2547/T-2548's sweep), the 23 last-digit regression-trend cells (T-2006), and
abc2939ebd1e2365 A187 (a 0 GROWTH y against a 2x2 x is `#NUM!`, unexplained; recorded in statistics.md).
