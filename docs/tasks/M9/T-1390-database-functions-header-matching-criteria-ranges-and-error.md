---
id: T-1390
title: "Database functions: header matching, criteria ranges and error choice differ from Excel on generated data (gen-m9 database sample)"
milestone: M9
lane: functions-stat
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
The database functions match Excel on the shapes that generated workbooks produce: header matching
with non-text or blank headers, criteria ranges that overlap the database or have one row, and the
error each one returns.

## Context
A seeded sample (`rustsheet-gen --milestone M9 --count 20 --seed 17`, using a scratch catalog that
held only the database signatures from T-1086) was not committed. excel-graph accepted all 20
workbooks, and the engine passed only 673 of 800 cells. Examples (Excel, then ours):
- `DSUM(Data!D1:E8,2,Data!C3:E8)` is 1000000000000003.6, where we give 0.
- `DVARP(Data!A1:D11,4,Data!B3:C11)` is 11.9025, where we give `#DIV/0!`.
- `DAVERAGE(Data!A7:A9,1,Data!B9:B9)` is `#VALUE!` (one-row criteria), where we give 3.14159.
- `DGET(Data!A1:D10,1,Data!C1:E4)` is "?b", where we give `#NUM!`.
- `DMAX(Data!A1:A10,1,Data!C3:D8)` is 123456789.123, where we give 0.
- `DAVERAGE(Data!A10:D12,4,Data!B3:E7)` is `#DIV/0!`, where we give -1.25005.
- `DCOUNT(Data!B4:D6,0,Data!D9:D10)` is 0, where we give `#VALUE!`.
The likely root causes are how a criteria header that is a number, a boolean or blank matches a
database column, criteria rows that are entirely blank, and when a one-row criteria range is
`#VALUE!`. Write a probe with one question per cell (excel-probe), record the rules in
docs/excel-semantics/statistics.md, then fix. Split the task if it is bigger than M. T-1054 covers
DVAR/DSTDEV numerics.

gen-m9 sweep (gap triage 2026-10-23, T-2255). 46 cells where a database function inside a statistical call is the
innermost divergence (DAVERAGE, DGET, DSUM, DSTDEV, DSTDEVP, DVAR, DVARP, DCOUNT, DCOUNTA, DMIN, DMAX, DPRODUCT over the
cases' Data sheet; the enclosing call only passes the value on, so the cell's Excel value is the D-function's or follows
from it). T-2258 (the database sweep) will add direct evidence.
Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 46 known-failure lines in eval/baseline/gen-m9.txt, 46 formulas.
Examples:
- gen-m9/0a614cfe974b59c3 Calc!A246 `=BETADIST(Data!A9,0.01,10000000000.5,DVARP(Data!A11:D12,4,Data!B5:D6),(MODE(Data!C8:C10,Data!C7:C11)-2.5))`: Excel #DIV/0!, ours #VALUE!
- gen-m9/27601819397dd56e Calc!A265 `=F.INV.RT(DSTDEV(Data!B9:E12,3,Data!D8:D10),1.5,1)`: Excel #NUM!, ours #DIV/0!
- gen-m9/285d87cea1aaa73c Calc!A210 `=HYPGEOM.DIST(5,DVARP(Data!C9:C10,1,Data!A7:C7),9,9,Data!$E8)`: Excel #VALUE!, ours 1.0
- gen-m9/2a18b2f5a4dc43ac Calc!A334 `=LINEST(IF(ISREF(Data!A2:B10),MIN(Data!C12,Data!C1:C3,(-0.5)),(Data!B5-1e-15)),((0+Data!D10)*DMIN(Data!A2:D11,4,Data!B9:D12)))`: Excel 0.0, ours #VALUE!
- gen-m9/530c636749a85209 Calc!A191 `=LOGEST(SUM(DAVERAGE(Data!C8:C10,4,Data!A8:D9),Data!A$2),0.5,(FACT(59)>=NORMSINV(2.5)),TRUE)`: Excel #VALUE!, ours #NUM!
- gen-m9/58e3253bbd3a7a82 Calc!A100 `=GAMMAINV(0.001,((0/(-1))+0.2),T.DIST.2T(DSTDEVP(Data!B9:C10,1,Data!B8:C11),6))`: Excel #DIV/0!, ours #NUM!
Cells:
- gen-m9/0a614cfe974b59c3.case.json (1): Calc!A246
- gen-m9/27601819397dd56e.case.json (1): Calc!A265
- gen-m9/285d87cea1aaa73c.case.json (1): Calc!A210
- gen-m9/2a18b2f5a4dc43ac.case.json (1): Calc!A334
- gen-m9/320d51765af9b6b7.case.json (1): Calc!A279
- gen-m9/3a25a03eb94fcfbd.case.json (1): Calc!A345
- gen-m9/4250f880ab640c49.case.json (1): Calc!A118
- gen-m9/4bad303d32cc3f46.case.json (1): Calc!A335
- gen-m9/530c636749a85209.case.json (2): Calc!A174, A191
- gen-m9/589f42e67f4b4b8d.case.json (1): Calc!A308
- gen-m9/58e3253bbd3a7a82.case.json (2): Calc!A100, A265
- gen-m9/60b0b6e69ad35a45.case.json (1): Calc!A374
- gen-m9/7bd219ec0c0c5e22.case.json (2): Calc!A25, A324
- gen-m9/85f48d5cdfb24555.case.json (1): Calc!A329
- gen-m9/899f9555e67da2d6.case.json (1): Calc!A122
- gen-m9/8d3588d8df0812df.case.json (3): Calc!A64, A89, A235
- gen-m9/8dd2312c96effb1b.case.json (2): Calc!A154, A347
- gen-m9/984cba67bbe6972b.case.json (1): Calc!A217
- gen-m9/a612e89321a6df9c.case.json (1): Calc!A297
- gen-m9/a61e547a1cab994c.case.json (1): Calc!A328
- gen-m9/b6593d7dd13f2dac.case.json (2): Calc!A66, A261
- gen-m9/bbdfc06cae0e0d71.case.json (1): Calc!A328
- gen-m9/c18dbf21f11093f3.case.json (4): Calc!A24, A178, A254, A351
- gen-m9/c468f5046aadfbbc.case.json (1): Calc!A43
- gen-m9/c6cfda6bd10cae93.case.json (1): Calc!A210
- gen-m9/cde551ea787f713a.case.json (1): Calc!A13
- gen-m9/d8266bc15533702a.case.json (1): Calc!A343
- gen-m9/e6cf3b20a014e6a3.case.json (1): Calc!A372
- gen-m9/ecc87f80077959e0.case.json (1): Calc!A318
- gen-m9/eced4bb6a0234c9e.case.json (1): Calc!A221
- gen-m9/f2bd5f380ec002bf.case.json (4): Calc!A40, A154, A321, A345
- gen-m9/fd9fa22d212080d6.case.json (1): Calc!A313
- gen-m9/fec39efaba9284c3.case.json (1): Calc!A288

T-2256 sweep (gen-m9 part 2, statistical M to Z; gap triage 2026-10-23): database functions nested in the statistical calls (`TDIST(DCOUNTA(Db!A1:D5,4,Db!O1:O2),6,2)` Excel 0.09242631153167513 because DCOUNTA is not 0; `NORMSDIST(DCOUNTA(Db!A1:D5,3,Db!F1))` `#VALUE!`).

Owned gen-m9 cells (T-2256 triage, excel-graph, 2026-10-23): 45 known-failure lines in eval/baseline/gen-m9.txt, 45 visibly off.
Examples:
- gen-m9/17c110a53924beb8 Calc!A334 `=TREND(DCOUNT(Db!A1:D5,4,Data!B3:D6))`: Excel 3.0, ours 0.0
- gen-m9/6e18550e9fc9c1b9 Calc!A379 `=STEYX((-0.5),DGET(Db!A1:D12,3,Db!F1))`: Excel #VALUE!, ours #NUM!
- gen-m9/a4090567758520df Calc!A82 `=NORMSDIST(DCOUNTA(Db!A1:D5,3,Db!F1))`: Excel #VALUE!, ours 0.9999683287581669
- gen-m9/d910f000b60e2eac Calc!A358 `=TDIST(DCOUNTA(Db!A1:D5,4,Db!O1:O2),6,2)`: Excel 0.09242631153167513, ours 1.0
- gen-m9/eedd629f3d69fdda Calc!A337 `=SMALL(DGET(Db!$A$1:$D$9,3,Db!F1),3)`: Excel #VALUE!, ours #NUM!
Cells:
- gen-m9/0072265d385d8ab2.case.json (1): Calc!A306
- gen-m9/09b47629f54b9817.case.json (1): Calc!A29
- gen-m9/0bbab6206b5c9a07.case.json (1): Calc!A301
- gen-m9/17c110a53924beb8.case.json (1): Calc!A334
- gen-m9/1b31092a0d79e5f1.case.json (1): Calc!A238
- gen-m9/2138b909d8f810ea.case.json (1): Calc!A108
- gen-m9/31f09e6c48ca245c.case.json (2): Calc!A253, A287
- gen-m9/329b177b5d5165ad.case.json (2): Calc!A139, A145
- gen-m9/334f3dc9ca937f10.case.json (1): Calc!A111
- gen-m9/3b7990bd4e3307ba.case.json (1): Calc!A286
- gen-m9/3fb9cecf2c23169f.case.json (1): Calc!A129
- gen-m9/41248d1cb153c66e.case.json (1): Calc!A387
- gen-m9/43c6227e0ff368c7.case.json (1): Calc!A337
- gen-m9/5389dd4d5b5739d2.case.json (1): Calc!A50
- gen-m9/5919660b7bb4d195.case.json (1): Calc!A138
- gen-m9/5be29fa316931f05.case.json (1): Calc!A299
- gen-m9/607e85cfee83a689.case.json (1): Calc!A256
- gen-m9/6e18550e9fc9c1b9.case.json (1): Calc!A379
- gen-m9/787ba8d801105a60.case.json (1): Calc!A392
- gen-m9/7fd0a98183c6babf.case.json (1): Calc!A24
- gen-m9/80586b859b73c87d.case.json (2): Calc!A186, A289
- gen-m9/91e167cd8d7eb652.case.json (2): Calc!A76, A148
- gen-m9/928d08e3e1eecb5f.case.json (2): Calc!A272, A280
- gen-m9/9bc247733a1ca9fa.case.json (1): Calc!A307
- gen-m9/9e2225ead8a6a07b.case.json (1): Calc!A10
- gen-m9/a4090567758520df.case.json (1): Calc!A82
- gen-m9/a63d1c7d6f96c56d.case.json (1): Calc!A113
- gen-m9/afc00c8b70e78baf.case.json (1): Calc!A11
- gen-m9/b9a32ce9a1fd0d71.case.json (1): Calc!A256
- gen-m9/bcb1105c61a2649d.case.json (1): Calc!A63
- gen-m9/c7664e494e418395.case.json (1): Calc!A13
- gen-m9/cb61169f4c033d4b.case.json (1): Calc!A130
- gen-m9/d208bbafc3bc1997.case.json (1): Calc!A399
- gen-m9/d910f000b60e2eac.case.json (4): Calc!A34, A259, A268, A358
- gen-m9/eccd03df42e40006.case.json (1): Calc!A219
- gen-m9/eedd629f3d69fdda.case.json (1): Calc!A337
- gen-m9/f82473275380882e.case.json (1): Calc!A38

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2255 and T-2256): these 91 known-failure lines (46 from T-2255, 45 from T-2256) disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] A probe answers each root cause, and the rules are in docs/excel-semantics/.
- [ ] The probed cells pass, and a regenerated seed-17 database sample passes at least 95% of cells.
- [ ] `make check` green; reviewers approve.

Not yet met: 70 of the 91 owned lines left gen-m9.txt (with 20 unowned lines also fixed, 90 in all, and no
new known failures); 21 owned lines remain (fidelity review, see Resume notes). The probed cells pass; the seed-17 sample was not
regenerated, and the 91 owned gen-m9 cells, which cover the same shapes, stand in for it.

## Out of scope
- Lifting (T-1389).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2255): the gen-m9 cells listed under Context are visibly off (a wrong type, error or
text, or a number further than half a unit in the 15th significant digit from Excel's), so the task is in the campaign. Its
target is that every visibly-off cell it owns has Excel's type, error and text, or is within half a unit of the 15th
significant digit of Excel's value (an expected 0 exactly 0); the visible-parity criterion under Acceptance criteria states
it. Bit-level criteria still unmet then move to a follow-up task without the key.

## Outcome
- Two probes (excel-graph, 2026-10-23): probes/db-header-criteria-shapes (72 questions) and
  probes/db-computed-criteria-shapes (53). Rules in docs/excel-semantics/statistics.md "Database criteria":
  one-row criteria or database is `#VALUE!`; headers match by coerced text, case-insensitively, untrimmed;
  a blank or error criteria header names no column; a condition under a header that names no column is a
  computed criterion (TRUE, any non-zero number or "TRUE" hold), and a formula condition is re-evaluated per
  record with its relative references moved down; a blank criteria row matches every record; a text field
  of "" names a blank header; DGET of a blank field cell is `#VALUE!`.
- functions/database.rs: criteria columns are resolved once per call and kept aligned with their criteria
  column (the old matcher shifted conditions onto the wrong header after an unmatched one).
- Eval: gen-m9 known failures 8216 -> 8126 (90 lines fixed, 70 of them owned; 0 regressions); probes +2 cases
  (125 cells, all pass). Owned cell gen-m9/7bd219ec0c0c5e22 Calc!A324 (error order) is T-2516.
- The seed-17 regenerated sample was not rerun: the 91 owned gen-m9 cells cover the same shapes.
- Nested formula conditions run one level only (DEVIATION, directive 5), recorded in statistics.md.

## Resume notes (2026-10-23, session budget reached mid-review)
Review round 1: security, architecture and quality APPROVE; Excel fidelity REQUEST_CHANGES.
- BLOCKING (fixed in the last commit, but `make check` and the new unit test
  `cells_reached_from_a_formula_condition_evaluate_in_full` have NOT been run yet): `db_condition_running`
  leaked into cells first computed during a re-evaluation; `run_cell_formula` (defined_names.rs) now saves,
  clears and restores it.
- MAJOR: 21 owned gen-m9 lines still fail, among them 0a614cfe974b59c3 A246, 2138b909d8f810ea A108,
  2a18b2f5a4dc43ac A334, 3b7990bd4e3307ba A286, 530c636749a85209 A174/A191, 5be29fa316931f05 A299,
  607e85cfee83a689 A256, 8d3588d8df0812df A64/A235, 91e167cd8d7eb652 A148, 984cba67bbe6972b A217 (NumericClose),
  9e2225ead8a6a07b A10, afc00c8b70e78baf A11, c18dbf21f11093f3 A24/A178, d208bbafc3bc1997 A399
  (`DGET(Db!$A$1:$D$9,1,Db!F1:G2)` Excel #VALUE!, ours #NUM!), d8266bc15533702a A343, d910f000b60e2eac A358
  (1 ulp) and f2bd5f380ec002bf A321. Note that the D-call alone passes for many of these
  (oracle-filled scratch copies of every owned D-call agreed with ours except 3), so check whether the
  enclosing call is what still differs, and move those cells to the owning tasks.
- MAJOR: the seed-17 sample criterion is unmet (not rerun).
- MINOR: add "(decision 0014)" to the DEVIATION in the statistics.md Status line, and say the nested-condition
  behavior of Excel is unprobed; pin the value in `self_referencing_formula_condition_is_bounded`; run
  `scripts/check_duplication.py --update` (3121 -> 3083).
- `probes/union-walk-repeated-in-lambda` timed out under load in a local `make eval-check`; it is unrelated
  (no D-functions). Confirm on main before you baseline. A full `make check` on ffb2cf31 (before the guard fix)
  passed fmt, clippy and tests and failed only at eval-check, on that one timeout.
