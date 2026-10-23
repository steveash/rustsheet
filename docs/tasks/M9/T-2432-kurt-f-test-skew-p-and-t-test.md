---
id: T-2432
title: "KURT, F.TEST, SKEW.P and T.TEST over data whose squares overflow (1E+200): Excel's #DIV/0! and #NUM! (gen-m9, 11 cells)"
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
KURT and F.TEST over data whose sums of squares overflow (`{1E+200,1E+200,-1E+200}`) return Excel's `#DIV/0!` (KURT) and
`#NUM!` (F.TEST and FTEST) instead of a finite number.

## Context
Gap triage of the T-2255 gen-m9 sweep (the data pool's cancelling large values). statistics.md "F.TEST: an error argument is
that error; overflowing squares are #NUM!, KURT's #DIV/0! (T-2255)" records it: 6 KURT cells (ours -4.9, -13.5, ..), 5
F.TEST/FTEST cells (ours 0). Excel's KURT likely divides by a variance that overflowed to infinity and its F.TEST checks for
an overflowing variance; the exact rule (which intermediate overflows, and whether DEVSQ, SKEW, VAR and STDEV behave
alike on the same data) is to be pinned by a probe of `KURT({1E+200,1E+200,-1E+200})`, `VAR.S(..)`, `DEVSQ(..)`,
`SKEW(..)` and `F.TEST({1,2,3},{1E+200,1E+200,-1E+200})` (excel-probe skill) before the change.
Code: `fn_kurt` (functions/statistical/count_and_average.rs), `fn_f_test` (fisher.rs).

Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 11 known-failure lines in eval/baseline/gen-m9.txt, 11 formulas.
Examples:
- gen-m9/27601819397dd56e Calc!A246 `=KURT(Data!A1:A12,{1E+200,1E+200,-1E+200},(-0.5))`: Excel #DIV/0!, ours -4.166666666666667
- gen-m9/2a18b2f5a4dc43ac Calc!A130 `=F.TEST(Data!A3:A10,{1E+200,1E+200,-1E+200})`: Excel #NUM!, ours 0.0
Cells:
- gen-m9/27601819397dd56e.case.json (1): Calc!A246
- gen-m9/2a18b2f5a4dc43ac.case.json (1): Calc!A130
- gen-m9/3a25a03eb94fcfbd.case.json (1): Calc!A183
- gen-m9/3e98b26cf6713e77.case.json (1): Calc!A339
- gen-m9/7c20351562a2d71d.case.json (1): Calc!A120
- gen-m9/a61e547a1cab994c.case.json (1): Calc!A14
- gen-m9/b8ee55934c5b824c.case.json (1): Calc!A64
- gen-m9/bc40d37cb4d8301d.case.json (1): Calc!A186
- gen-m9/c2974c061384fdd9.case.json (1): Calc!A398
- gen-m9/ee2497a5e683c5d9.case.json (2): Calc!A166, A396

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2255): these 11 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The probe cells are in eval/ and statistics.md's entry has the rule and an "Ours:" line naming this task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The cancellation of finite data (T-1749).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2255): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.
