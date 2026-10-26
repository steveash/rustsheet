---
id: T-2551
title: "DB with a negative salvage returns NaN as a number (ISNUMBER is TRUE) instead of #NUM! (gen-m9, 1 cell)"
milestone: M9
lane: functions-financial
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
DB with a negative salvage is `#NUM!` inside a formula too, not a NaN number that ISNUMBER sees as a number.

## Context
Gap triage of the T-2258 gen-m9 sweep. In the engine `=DB(59341,-1,28,6,12)` shows `#NUM!`, but nested it is a number: `ISNUMBER(DB(59341,-1,28,6,12))` is TRUE, `ISERROR(..)` FALSE, `TYPE(..)` 1, `ERROR.TYPE(..)` `#N/A`, so `DCOUNTA(Db!A1:D9,DB(59341,(Data!D$10-Data!D5),28,6,12),Db!F1:F2)` is `#VALUE!` (a bad field) where Excel gives DB's `#NUM!` (gen-m9/f066968376660329 A244). The kernel returns a non-finite number ((salvage/cost)^(1/life) with a negative salvage) and only the cell writer turns it into `#NUM!`. Return `#NUM!` from `fn_db` for a non-finite result (check DDB and VDB for the same leak), and say in the Outcome whether other functions can leak a non-finite Number (an engine follow-up if so).
Code: `fn_db` in crates/rustsheet-engine/src/functions/financial/financial_depreciation.rs.

Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 1 known-failure lines in eval/baseline/gen-m9.txt, 1 formulas, 1 visibly off.
Examples:
- gen-m9/f066968376660329 Calc!A244 `=DCOUNTA(Db!A1:D9,DB(59341,(Data!D$10-Data!D5),28,6,12),Db!F1:F2)`: Excel #NUM!, ours #VALUE!
Cells:
- gen-m9/f066968376660329.case.json (1): Calc!A244

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2258): these 1 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Unit test: `ISNUMBER(DB(59341,-1,28,6,12))` is FALSE and `ISERROR(..)` TRUE.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other T-2258 triage families: each has its own task (T-2258 Outcome).
## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2258): every cell this task owns has the wrong type, error, text or value (none is a last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria states it.

## Outcome
Already fixed on main before this task was picked: `fn_db` returns `#NUM!` for a negative salvage
(the `salvage < 0.0` domain check), and gen-m9/f066968376660329 Calc!A244 is no longer a
known-failure line in eval/baseline/gen-m9.txt. This PR adds the missing unit test
(`fn_db_negative_salvage_is_an_error_value`: ISNUMBER FALSE, ISERROR TRUE, `#NUM!`). Eval delta: none.
No non-finite Number leak was reproduced for DB. DDB and VDB (financial_depreciation.rs) carry the same `salvage < 0.0` domain check before any arithmetic, so neither leaks NaN for a negative salvage (read, not probed). Whether other functions can return a non-finite Number was not audited.
