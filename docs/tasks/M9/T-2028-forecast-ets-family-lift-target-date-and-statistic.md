---
id: T-2028
title: "FORECAST.ETS family: lift target_date and statistic_type and coerce text target_date to #VALUE! before #N/IMPL!"
milestone: M9
lane: functions-stat
status: done
priority: P3
size: S
tier: sonnet
campaign: visible
depends_on: []
---

## Goal
The four FORECAST.ETS functions keep returning `#N/IMPL!` for a call they would compute, but take their
scalar arguments as Excel does first: `target_date` (FORECAST.ETS, FORECAST.ETS.CONFINT) and
`statistic_type` (FORECAST.ETS.STAT) lift over an array or a two-cell range into an array of results, and a
text `target_date` is `#VALUE!`.

## Context
Gap triage 2026-10-19. docs/excel-semantics/statistics.md "FORECAST.ETS family: excel-graph never computes
it": every valid call is `#N/A` on the web, so the web is a golden only for the type-coercion step and the
algorithm waits for desktop values (T-1553, M14). The coercion and lifting cells are web evidence that
does not depend on the algorithm (known failures in eval/baseline/probes.txt):
- probes/forecast-ets-longer-series Sheet1!K38 `FORECAST.ETS("x",B1:B24,A1:A24)`: #VALUE!, ours #N/IMPL!.
- probes/stat-argument-modes Sheet1!H268 `ROWS(FORECAST.ETS({9;9},A1:A8,E1:E8))`, H269 (`G1:G2`), H272,
  H273 (the same for CONFINT), H279 `ROWS(FORECAST.ETS.STAT(A1:A8,E1:E8,{4;4}))`, H280 (`G1:G2`): 2, ours
  #N/IMPL!. So the core lift kind (`Lift`, a two-cell range lifts), not `LiftArrays`.
Code: crates/rustsheet-engine/src/functions/statistical/forecast.rs (`fn_forecast_ets*` return #N/IMPL!
without reading their arguments). Route them through `lift_call` with a kernel that returns `#N/IMPL!`
after the number coercion; register the kinds in the SPEC-0011 table (lift/kinds/statistical.rs) and
regenerate eval/catalog/lift-kinds.json (`cd tools && uv run rustsheet-catalog-lift --write`).
The elements themselves stay `#N/IMPL!`, so H266/H267/H270/H271/H274-H278/H281 and the other K cells keep
failing (desktop evidence, T-1553).

Visibly-off cells (gap triage 2026-10-21, decision 0083): 7 cells, 0 numeric past half a unit in the 15th digit and 7 with a wrong type, error or text, blamed on ROWS 6, FORECAST.ETS 3, FORECAST.ETS.CONFINT 2, FORECAST.ETS.STAT 2. Every one is listed above.

## Acceptance criteria
- [ ] Visible parity: the 7 visibly-off cells listed under Context (2026-10-21) have Excel's value to 15 significant digits, or its type, error and text: the 7 lines in eval/baseline/probes.txt disappear (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The 7 lines above (K38, H268, H269, H272, H273, H279, H280) leave eval/baseline/probes.txt
      (`make eval-baseline`), with no new F lines in any baseline.
- [ ] Every other call still returns `#N/IMPL!` (unit test), and statistics.md's entry has an "Ours" line
      for the coercion and lifting.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any validation whose web answer is `#N/A` (seasonality, data_completion, aggregation, confidence_level,
  sizes): it cannot be told apart from a valid call on the web (T-1553).
- The gen-m9 cells whose FORECAST.ETS.CONFINT is `#VALUE!` on the web (gen-m9/1782efcecd5d4b1b A27,
  3462856259a37891 A28): their cause is unknown; list it in the Outcome if found, else leave them.

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (`#N/IMPL!` where Excel lifts or coerces first), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it. The other FORECAST.ETS cells stay `#N/IMPL!` by statistics.md "FORECAST.ETS family: excel-graph never computes it" (T-1553) and are not this stage.

## Outcome
FORECAST.ETS, .CONFINT and .STAT now go through `lift_call` (kinds `ETS_TARGET`, `ETS_CONFINT_TARGET`, `ETS_STAT_TYPE`, lift/kinds/statistical.rs): the lifted argument is read as a number first (`#VALUE!` for text), every element is otherwise `#N/IMPL!`. The 7 listed probes cells leave eval/baseline/probes.txt (7 lines removed, nothing added). Unit test `forecast_ets_family_reads_lifted_argument_then_is_not_implemented`. The gen-m9 CONFINT `#VALUE!` cells were not investigated.
