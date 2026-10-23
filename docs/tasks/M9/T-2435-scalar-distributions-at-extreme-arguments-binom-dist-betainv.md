---
id: T-2435
title: "Scalar distributions at extreme arguments: BINOM.DIST, BETAINV, GAMMA.DIST, CRITBINOM, FDIST and LOGNORMDIST range checks (gen-m9, 20 cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
BINOM.DIST, BINOMDIST, BINOM.INV, CRITBINOM, BETAINV, GAMMA.DIST, GAMMADIST, FDIST, LOGNORMDIST and EXPON.DIST give Excel's
result at extreme arguments: `#NUM!` for p = 1 with fewer successes than trials, a slightly negative number_s, 2^32 trials,
a tiny beta alpha, a huge gamma shape, df 1E10, 1E15 trials, and 0 for a pdf that underflows or is subnormal.

## Context
Gap triage of the T-2255 gen-m9 sweep. statistics.md "Scalar distributions at extreme arguments (T-2255)" lists each cell;
20 cells, one per shape, so each rule rests on one cell. Before implementing a threshold (where 2^32 trials or a 1E15
CRITBINOM starts to be `#NUM!`, how small a beta alpha), probe a few values around it (excel-probe skill). EXPON.DIST's
0 is numbers.md "Subnormal results become 0" applied to a result. Overlaps: T-1891 (huge gamma shapes), T-1871 (BETAINV
tails), T-1538 (huge df); take the cell from them if one lands first.
Code: functions/statistical/binom.rs, beta.rs, gamma.rs, fisher.rs, lognormal and exponential readers.

Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 20 known-failure lines in eval/baseline/gen-m9.txt, 20 formulas.
Examples:
- gen-m9/05d44f6dd6cfc567 Calc!A344 `=BINOM.DIST(0.1,2,1,0)`: Excel #NUM!, ours 0.0
- gen-m9/165db58d1540f014 Calc!A65 `=EXPON.DIST(ERROR.TYPE(MATCH("TRUE",Data!C5:E6,-1)),(100+MIN(Data!C8,Data!D7:D7,2)),0)`: Excel 0.0, ours 8.36257629135218e-309
- gen-m9/2801c1c11c21e134 Calc!A99 `=GAMMA.DIST(1E+307,100,0.5,FALSE)`: Excel 0.0, ours #NUM!
Cells:
- gen-m9/05d44f6dd6cfc567.case.json (1): Calc!A344
- gen-m9/165db58d1540f014.case.json (1): Calc!A65
- gen-m9/2801c1c11c21e134.case.json (2): Calc!A99, A241
- gen-m9/3e98b26cf6713e77.case.json (1): Calc!A151
- gen-m9/40425341c5034eed.case.json (1): Calc!A265
- gen-m9/4ab2ba98d26c8a5e.case.json (1): Calc!A104
- gen-m9/7bd219ec0c0c5e22.case.json (1): Calc!A320
- gen-m9/85f48d5cdfb24555.case.json (1): Calc!A35
- gen-m9/8fec7b207f4dd753.case.json (1): Calc!A125
- gen-m9/9cca17438a7e6f8a.case.json (1): Calc!A240
- gen-m9/a013a45a8fbcab4f.case.json (2): Calc!A97, A206
- gen-m9/a5828d3925575713.case.json (1): Calc!A315
- gen-m9/a61e547a1cab994c.case.json (1): Calc!A127
- gen-m9/b6593d7dd13f2dac.case.json (1): Calc!A231
- gen-m9/bc40d37cb4d8301d.case.json (1): Calc!A56
- gen-m9/c6cfda6bd10cae93.case.json (1): Calc!A363
- gen-m9/caa1eed48b1883fe.case.json (1): Calc!A384
- gen-m9/ee2497a5e683c5d9.case.json (1): Calc!A345

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2255): these 20 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Each rule's probe cells are in eval/, and statistics.md's entry gets an "Ours:" line naming this task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Last digits of these functions (their accuracy tasks).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2255): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.
