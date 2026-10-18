---
id: T-1236
title: "WORKDAY, WORKDAY.INTL, NETWORKDAYS and NETWORKDAYS.INTL lift over arrays and refuse ranges (SPEC-0011 driver)"
milestone: M6
lane: functions-date
status: todo
priority: P1
size: S
tier: opus
depends_on: [T-1770, T-1295, T-1808]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
WORKDAY, WORKDAY.INTL, NETWORKDAYS and NETWORKDAYS.INTL lift over an array argument and are `#VALUE!` for a multi-cell range in a date or day argument, as Excel does. EDATE, EOMONTH, WEEKNUM and YEARFRAC moved to T-1770.

## Context
SPEC-0011 rollout step 2 (docs/specs/SPEC-0011-scalar-argument-lifting.md). Rescoped by T-1767: EDATE, EOMONTH, WEEKNUM and YEARFRAC, and the gen-m9 EOMONTH cells below, are T-1770's first consumers. This task does WORKDAY, WORKDAY.INTL, NETWORKDAYS and NETWORKDAYS.INTL. Their date and day arguments are `LiftArrays`, `weekend` is `Lift` (it lifts over a range too), and `holidays` is `Whole`. After T-1295 (WORKDAY's whole-week stepping) and T-1808 (NETWORKDAYS in closed form) a kernel call costs O(holidays), so the entries are `Constant` or `Prepared(holiday cells)` and `Weekend::lift`'s per-mask memo is no longer needed. Its probe cells are F/G 36, 38, 40, 58, 60, 62.

probes/date-argument-modes (excel-graph, 2026-10-09, T-1090): these Analysis ToolPak-era functions return `#VALUE!`
when a date, month count, day count, return type or basis argument is a multi-cell range (the engine matches that),
but lift over an array literal in the same place (`INDEX(EDATE({45000.25;45031.5;45100.75},1),2)` = 45061), where the
engine gives `#N/IMPL!` (F20, F22, F24, F26, F36, F38, F40, F54, F56, F58, F60, F62, F66, F68 and their G cells).
Holidays stay read whole (dates.md "WORKDAY holidays take the whole range"); the weekend lift over a range is T-1235's (done), kept as `Lift`.
Semantics: docs/excel-semantics/dates.md "Date function argument modes".

More cells (T-1289's gen-m9 suite, excel-graph, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/4df1b22d3ed63f58 Calc!A24 `=EOMONTH((IF(Data!E11,Data!$E1)+WRAPROWS(Data!D3:D7,1,Data!$E9)),-14)`: Excel `#SPILL!`, engine `#N/IMPL!` (an array start date lifts).
- gen-m9/b58b6d7cd39d195f Calc!A9 `=EOMONTH((1*HYPGEOMDIST(0,3,1,13)),TAKE(Data!B9:E12,-2,-1))`: Excel `#VALUE!`, engine `#N/IMPL!`. TAKE of a range is a reference (references.md, ISREF of it is TRUE), so this is the range form (`#VALUE!`), not an array lift: keep TAKE/DROP results on the range path when lifting.

## Acceptance criteria
- [ ] probes/date-argument-modes F/G 36, 38, 40, 58, 60 and 62 pass, the range forms stay `#VALUE!` and the weekend lifts keep passing; baseline updated.
- [ ] No new failing gen-m9 cell and no new known failures in eval/baseline/ (the two gen-m9 EOMONTH cells are T-1770's).
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Handoff (resumed, then split)
Branch `agent/functions-date/T-1236-date-fn-array-lift` (PR #991, draft) holds a complete, reviewed implementation of the *old* T-1236 scope (EDATE, EOMONTH, WEEKNUM, WORKDAY(.INTL), NETWORKDAYS(.INTL), YEARFRAC): `lift_array_args` in `date_and_time/lift.rs`, a thin wrapper over `lift_with_cost`, with the `*_scalar` bodies behind lifting entry points. All four reviewers approved it, `make check` was green and probes known failures went 3026 -> 2998 (28 date-argument-modes cells). Before it could merge, `main` adopted SPEC-0011 (T-1767), which rescopes this task to WORKDAY/NETWORKDAYS on the table-driven driver (depends on T-1770, not yet done) and moves EDATE/EOMONTH/WEEKNUM/YEARFRAC to T-1770. My wrapper would be a parallel lifting mechanism next to the spec's driver, so I did not merge it. Next: decide whether to land it as an interim step (then T-1770/T-1236 migrate it) or drop it; if dropped, reuse its pieces: the cost estimates (`day_span`/`step_days`, capped at the calendar span, holidays counted per element), the tests in `test_fn_date_array_lift.rs`, and the ACCRINTM finding (financial callers must call the scalar YEARFRAC, not a lifted one). Unprobed items are tracked by T-1807.
Resumed by opus 2026-10-17: the branch now merges origin/main and takes main's SPEC-0011 code. The interim `lift_array_args` wrapper is dropped, because it would be a second lifting mechanism next to the driver. Its tests and cost estimates are still in commit 55b64dca. The probe follow-up was renumbered T-1807, because T-1789 is taken on main.

On the driver, the day loops are the blocker. `KernelWork::Weighted` charges each element a fixed weight, and a weight that covers the worst-case day loop (about 2.96M days) caps a lift at about 45 elements under `MAX_LIFTED_CELL_VISITS`. That would regress the weekend lifts main already does for any range size. T-1295 (in flight on its own branch) makes WORKDAY(.INTL) step whole weeks and T-1808 makes NETWORKDAYS(.INTL) closed-form first; after it this task enters them as `Constant`, or as `Prepared(holiday cells)`, with holidays `Whole`, `weekend` `Lift` and the dates `LiftArrays`.
