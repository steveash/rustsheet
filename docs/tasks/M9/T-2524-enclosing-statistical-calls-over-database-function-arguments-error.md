---
id: T-2524
title: "Enclosing statistical calls over database-function arguments: error choice and last-ulp values (T-1390 follow-up)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
The gen-m9 cells below, which T-1390's triage owned because a database function is the innermost call,
match Excel. Their database calls already do: what still differs is the enclosing call (which error it
returns when several arguments are errors or out of range, or its last digit).

## Context
T-1390 resume (2026-10-23): a scratch oracle fill (excel-graph) evaluated each cell's whole formula, each
top-level argument of its outermost call, and each D-call alone, on the cell's own workbook. Every D-call
and every top-level argument agreed with Excel, except where noted; only the outermost call differs:
- gen-m9/0a614cfe974b59c3 Calc!A246 `BETADIST(...)`: Excel #DIV/0!, ours #VALUE!
- gen-m9/2138b909d8f810ea Calc!A108 `PROB(...)`: Excel #VALUE!, ours #DIV/0!
- gen-m9/afc00c8b70e78baf Calc!A11 `PROB(...)`: Excel #DIV/0!, ours #VALUE!
- gen-m9/2a18b2f5a4dc43ac Calc!A334 `LINEST(<number>,<error>)`: Excel 0, ours #VALUE!
- gen-m9/8d3588d8df0812df Calc!A64 `LINEST(Data!E$10,DMAX(...))`: Excel 0, ours #VALUE!
- gen-m9/530c636749a85209 Calc!A191 `LOGEST(...)`: Excel #VALUE!, ours #NUM!
- gen-m9/f2bd5f380ec002bf Calc!A321 `LOGEST(...)`: Excel #NUM!, ours #VALUE!
- gen-m9/3b7990bd4e3307ba Calc!A286 `T.TEST(...)`: Excel #DIV/0!, ours #NUM!
- gen-m9/607e85cfee83a689 Calc!A256 `T.TEST(...)`: Excel #VALUE!, ours #NUM!
- gen-m9/9e2225ead8a6a07b Calc!A10 `T.TEST(...)`: Excel #VALUE!, ours #NUM!
- gen-m9/91e167cd8d7eb652 Calc!A148 `TTEST(...)`: Excel #N/A, ours #VALUE!
- gen-m9/530c636749a85209 Calc!A174 `FREQUENCY(...)`: Excel #DIV/0!, ours #VALUE!
- gen-m9/5be29fa316931f05 Calc!A299 `PERCENTILE(...)`: Excel #VALUE!, ours #NUM!
- gen-m9/8d3588d8df0812df Calc!A235 `BETAINV(...)`: Excel #NUM!, ours #VALUE!
- gen-m9/c18dbf21f11093f3 Calc!A178 `F.TEST(DVARP(...),Data!A5:C5)`: Excel #DIV/0!, ours #VALUE!
- gen-m9/c18dbf21f11093f3 Calc!A24 `COVARIANCE.S(DATE(DSUM(..),XIRR(..),14),..)`: the inner `DATE(...)` already
  differs (Excel #NUM!, ours #VALUE!); its DSUM agrees.
- gen-m9/d208bbafc3bc1997 Calc!A399 `ZTEST(...)`: Excel #VALUE!, ours #NUM!
- gen-m9/d8266bc15533702a Calc!A343 `GAMMA.DIST(..., IF(GETPIVOTDATA(...),...))`: the GETPIVOTDATA argument
  differs (Excel "3.995" for the IF, ours #REF!); its DMIN agrees.
- gen-m9/984cba67bbe6972b Calc!A217 `CHISQ.DIST(...)`: Excel 1.1210083920801943e-35, ours 1.1210083920802015e-35
- gen-m9/d910f000b60e2eac Calc!A358 `TDIST(3,6,2)`: Excel 0.09242631153167513, ours 0.09242631153167512 (1 ulp)
Use `make eval-show CASE=gen-m9/<id>.case.json` for the full formulas. Probe each function's error
precedence (excel-probe, one question per cell) before changing it; split by function family if it is
bigger than M. Cells already owned elsewhere when you start move to that task.

## Acceptance criteria
- [ ] Visible parity: the 18 error or type cells above, and CHISQ.DIST A217 (off in the 14th significant digit), leave eval/baseline/gen-m9.txt (`make eval-baseline`), or
      move to a task that owns their enclosing function, with no new known-failure lines.
- [ ] TDIST A358 (1 ulp) matches Excel or moves to the task that owns that distribution's numerics.
- [ ] Each error-precedence rule found is in docs/excel-semantics/.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the cells above are visibly off (a wrong error or type), so the task is in the campaign; A217
is visibly off too (14th digit); only A358 (1 ulp) is bit-level and may move to a follow-up without the key.

## Out of scope
- Database-function semantics (T-1390, T-2516).
