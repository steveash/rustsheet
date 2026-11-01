---
id: T-2552
title: "gen-m9 engineering cells waiting on T-2093's F.TEST of a scalar: BESSELI and COMPLEX over it (2 cells)"
milestone: M9
lane: functions-stat
status: done
priority: P3
size: S
tier: sonnet
depends_on: [T-2093, T-2532]
campaign: semantics
---

## Goal
The two gen-m9 engineering cells whose value is F.TEST of a scalar pass once T-2093 and T-2532 have landed.

## Context
Gap triage of the T-2258 gen-m9 engineering sweep. `BESSELI(FTEST(0.1,IF(..)<0.5>),EOMONTH(55462,1))` is `#DIV/0!` in Excel (gen-m9/4e180f54ff140abc A186; ours `#VALUE!`): F.TEST of two scalars is `#DIV/0!` (T-2093, done on origin/agent/functions-stat/T-2093-ftest-scalar, not merged). `COMPLEX(INT(..)<#VALUE!>,HLOOKUP(..)<#N/A>,SWITCH(F.TEST(<1E+15>,<0.2>),..))` is `#DIV/0!` (gen-m9/df083d85b436b49a A120): the suffix's error first (T-2532) and that error is SWITCH's F.TEST `#DIV/0!`. Verify after both merge, and fix whatever is left.

Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 2 known-failure lines in eval/baseline/gen-m9.txt, 2 formulas, 2 visibly off.
Examples:
- gen-m9/4e180f54ff140abc Calc!A186 `=BESSELI(FTEST(0.1,IF(Data!$D11,Data!$A10,Data!$B5)),EOMONTH(55462,1))`: Excel #DIV/0!, ours #VALUE!
- gen-m9/df083d85b436b49a Calc!A120 `=COMPLEX(INT((Data!C3+Data!B2)),(HLOOKUP(Data!A12,Data!D5:D5,4,FALSE)/Data!$E4),SWITCH(F.TEST(Data!E6,Data!C11),(Data!B11/2),SUMIF(Data!C7:D10,"a*",Data!B3:D3),WRAPCOLS(Data!C1,1,(-1)),RIGHT(Data!B12),(1.005*(-1)),"a*"...`: Excel #DIV/0!, ours #VALUE!
Cells:
- gen-m9/4e180f54ff140abc.case.json (1): Calc!A186
- gen-m9/df083d85b436b49a.case.json (1): Calc!A120

## Acceptance criteria
- [x] Visible parity (gen-m9, T-2258): these 2 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] `make check` green; reviewers approve.

## Out of scope
- The other T-2258 triage families: each has its own task (T-2258 Outcome).
## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2258): every cell this task owns has the wrong type, error, text or value (none is a last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria states it.

Note (T-2532): gen-m9/df083d85b436b49a Calc!A120 already passes on main with T-2532, so it left the baseline there; skip it here.

## Outcome
- Verified on origin/main: gen-m9/4e180f54ff140abc Calc!A186 (`#DIV/0!`) and gen-m9/df083d85b436b49a Calc!A120 (`#DIV/0!`)
  both pass, and eval/baseline/gen-m9.txt has no known-failure line for either (T-2093 and T-2532 landed). No code change
  was needed. Eval delta: none.
