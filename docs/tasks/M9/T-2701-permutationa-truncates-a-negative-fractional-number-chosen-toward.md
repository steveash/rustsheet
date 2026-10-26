---
id: T-2701
title: "PERMUTATIONA truncates a negative fractional number_chosen toward 0, so it is 1, not #NUM! (gen-m9, 5 cells)"
milestone: M9
lane: functions-stat
status: done
priority: P3
size: S
tier: sonnet
campaign: semantics
depends_on: []
---

## Goal
PERMUTATIONA(number, number_chosen) with number_chosen in (-1, 0) truncates it toward 0 before the sign check, so the
result is number^0 = 1, as Excel does. Today ours floors it to -1 and returns `#NUM!`.

## Context
Cut from T-2474 (gap triage 2026-10-25, sonnet refill). T-2474 keeps PERMUTATIONA's `#NUM!` for a number of about 1E+15
or more and its unprobed threshold.

Rule and evidence: docs/excel-semantics/statistics.md "Scalar distributions at extreme arguments, part 2 (T-2256)":
"PERMUTATIONA truncates number_chosen toward 0 before its sign check (`PERMUTATIONA(10,-0.75)` and `(7,-0.1)` are 1,
0620505280d2ea6e A280, 5395b8a427764a3f A385)". This is the opposite of the discrete distributions, which check the
sign first (T-2700).

Code: `permutationa_of` in crates/rustsheet-engine/src/functions/statistical/permut.rs. `number_chosen` is
`.floor() as i64`. Truncate it instead with `.trunc()`. For a positive value floor and trunc agree, so only (-1, 0)
changes. A number_chosen of -1 or less stays `#NUM!` (no cell probes it). Leave `number`'s floor alone, since no cell here has a negative
fractional `number`.

Excel cells (gen-m9, excel-graph, 2026-10-23; known failures in eval/baseline/gen-m9.txt). Each is Excel 1 where ours is
`#NUM!`:
- gen-m9/0620505280d2ea6e Calc!A280 `=PERMUTATIONA(10,(Data!E12*(-0.5)))` (number_chosen -0.75)
- gen-m9/5395b8a427764a3f Calc!A385 `=PERMUTATIONA(7,(0.1*(Data!C10+Data!E9)))` (number_chosen -0.1)
- gen-m9/31f09e6c48ca245c Calc!A227 `=PERMUTATIONA(8,SUBTOTAL(5,Data!A1:A7,Data!A7:B11,Data!A3:A10))`
- gen-m9/5cebac1f02bf30f2 Calc!A148 `=PERMUTATIONA(3,((Data!E2-Data!$B$1)/WEEKDAY(60,1)))`
- gen-m9/a5342a7e24fd803e Calc!A370 `=PERMUTATIONA(8,ISPMT(1E-10,12,251,196427))`

The same rule probably fixes T-2621's probe cell probes/t2475-stat-subexpressions P!A28 `PERMUTATIONA(D02!D$12,D02!A12)`
(Excel 1, ours `#NUM!`; gen-m9/097af1ca33638670 Calc!A358). Whichever task lands first takes it. Note it in T-2621's
Outcome if this one does.

Cells:
- gen-m9/0620505280d2ea6e.case.json (1): Calc!A280
- gen-m9/31f09e6c48ca245c.case.json (1): Calc!A227
- gen-m9/5395b8a427764a3f.case.json (1): Calc!A385
- gen-m9/5cebac1f02bf30f2.case.json (1): Calc!A148
- gen-m9/a5342a7e24fd803e.case.json (1): Calc!A370

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2256): these 5 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Unit tests: `PERMUTATIONA(10,-0.75)` and `PERMUTATIONA(7,-0.1)` are 1. A number_chosen of -1 or less keeps
      today's `#NUM!`. That is unprobed: Excel documents it, and the truncate-then-check rule implies it.
- [ ] statistics.md's T-2256 entry gets an "Ours:" note for PERMUTATIONA naming this task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- PERMUTATIONA of a number of about 1E+15 or more (T-2474) and a negative fractional `number` (unprobed).
- PERMUT, COMBIN, COMBINA (T-2621 has COMBINA's negative fraction).

## Visible-parity stage
Decision 0083: every cell this task owns has the wrong type (`#NUM!` for Excel's 1), so the task is in the campaign at the
`semantics` stage. Its target is that every cell it owns has Excel's type and value; the visible-parity criterion under
Acceptance criteria states it.

## Outcome
Closed by T-2621, which made the same change (`permutationa_of` truncates number_chosen) for its probe cell
probes/t2475-stat-subexpressions P!A28; the five gen-m9 cells above left eval/baseline/gen-m9.txt in that PR.
