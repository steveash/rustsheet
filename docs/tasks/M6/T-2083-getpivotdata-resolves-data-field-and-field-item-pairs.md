---
id: T-2083
title: "GETPIVOTDATA resolves data_field and field/item pairs against a loaded pivot table (SPEC-0015)"
milestone: M6
lane: functions-lookup
status: in-progress
priority: P3
size: M
tier: opus
campaign: visible
depends_on: [T-2081]
spec: docs/specs/SPEC-0015-pivot-tables-and-getpivotdata.md
---

## Goal
GETPIVOTDATA answers from a loaded pivot table as Excel does on every cell of probes/getpivotdata-real-pivot.

## Context
SPEC-0015 "GETPIVOTDATA": pick the pivot by the reference, the data field by caption or source name (case-insensitive, untrimmed, blank = the only data field), the items on the row/column axes, then aggregate the cache records (Sum over none is 0; every unknown listed in SPEC-0015 "Excel behavior to match" is `#REF!` or keeps T-1703's behavior). Arrays lift (SPEC-0011). Unknowns stay `#REF!` until T-2085 has evidence. Replaces the T-1703 stub in functions/lookup_and_reference/getpivotdata.rs. Depends on T-2081 (tests can build the model directly); scores in eval once T-2084 lands.

Visibly-off cells (gap triage 2026-10-21, decision 0083): 19 cells, 0 numeric past half a unit in the 15th digit and 19 with a wrong type, error or text, blamed on GETPIVOTDATA 19.

| case / cell | formula | Excel | ours |
|---|---|---|---|
| probes/getpivotdata-real-pivot P!H1 | `=GETPIVOTDATA("Sales",$A$3)` | 49 | #REF! |
| probes/getpivotdata-real-pivot P!H10 | `=GETPIVOTDATA("sales",$A$3,"region","east")` | 22 | #REF! |
| probes/getpivotdata-real-pivot P!H11 | `=GETPIVOTDATA("Sales",$C$6)` | 49 | #REF! |
| probes/getpivotdata-real-pivot P!H13 | `=GETPIVOTDATA("Sales",$A$3,"Region","West","Product","C")` | 0 | #REF! |
| probes/getpivotdata-real-pivot P!H14 | `=GETPIVOTDATA("Sales",$A$3,"Region","West","Product","B")` | 7 | #REF! |
| probes/getpivotdata-real-pivot P!H16 | `=GETPIVOTDATA("Sales",$A$3:$E$7)` | 49 | #REF! |
| probes/getpivotdata-real-pivot P!H2 | `=GETPIVOTDATA("Sum of Sales",$A$3)` | 49 | #REF! |
| probes/getpivotdata-real-pivot P!H3 | `=GETPIVOTDATA("Sales",$A$3,"Region","East")` | 22 | #REF! |
| probes/getpivotdata-real-pivot P!H4 | `=GETPIVOTDATA("Sales",$A$3,"Product","A")` | 34 | #REF! |
| probes/getpivotdata-real-pivot P!H5 | `=GETPIVOTDATA("Sales",$A$3,"Region","East","Product","A")` | 14 | #REF! |
| probes/getpivotdata-real-pivot P!H6 | `=GETPIVOTDATA("Sales",$A$3,"Product","A","Region","East")` | 14 | #REF! |
| probes/getpivotdata-real-pivot P!J1 | `=GETPIVOTDATA("Sales",$A$20,"Region","East")` | 5.5 | #REF! |
| probes/getpivotdata-real-pivot P!J2 | `=GETPIVOTDATA("Average of Sales",$A$20)` | 8.166666666666666 | #REF! |
| probes/getpivotdata-real-pivot P!J5 | `=GETPIVOTDATA("Sales",$A$20)` | 8.166666666666666 | #REF! |
| probes/getpivotdata-real-pivot P!J6 | `=GETPIVOTDATA("Sales",$A$3,"Region",A5)` | 22 | #REF! |
| probes/getpivotdata-real-pivot P!J7 | `=GETPIVOTDATA(C3,$A$3)` | 49 | #REF! |
| probes/getpivotdata-real-pivot P!J8 | `=GETPIVOTDATA("Sales",$A$3,"Product","a","Region","EAST")` | 14 | #REF! |
| probes/getpivotdata-real-pivot P!L1 | `=GETPIVOTDATA("Sales",$A$3,"Region",{"East";"West"})` | 22 | #REF! |
| probes/getpivotdata-real-pivot P!L5 | `=GETPIVOTDATA("Sales",$A$3,"Product",{"A","B","C"})` | 34 | #REF! |

## Acceptance criteria
- [ ] Visible parity: the 19 visibly-off cells listed under Context (2026-10-21) have Excel's value to 15 significant digits, or its type, error and text: the 19 lines in eval/baseline/probes.txt disappear (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Unit tests against a model built in code cover every rule row of SPEC-0015's table.
- [ ] The index lives in the Model's evaluation state keyed by pivot (not on `PivotTable`/`PivotCache`, so `Workbook`'s `PartialEq`/`Clone` are unaffected), invalidated by changes to that pivot or cache; the memo is cleared at each `Model::evaluate` (bounded), and the bench counts it.
- [ ] If T-2085 shows Excel sums in record or display order on non-representable decimals, the index keeps per-tuple record lists (or exact sums) instead of folded accumulators; re-run the bench then.
- [ ] Names resolved once per call, never case-folded per record.
- [ ] Any change to pivot_tables/pivot_caches dirties every GETPIVOTDATA cell (holds under full recalc; a test pins it).
- [ ] The lazy per-pivot index and memo of SPEC-0015 "Untrusted-input limits" bound the work; a bench or test shows N calls over a large cache cost O(records + N).
- [ ] No NaN or infinite result (non-finite is `#NUM!`, undefined aggregates `#REF!` until T-2085).
- [ ] probes/getpivotdata-evidence-floor still passes; re-check the rules when T-2085 merges.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Pivot refresh, creation, layout in the engine, export (SPEC-0015 non-goals).

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (`#REF!` where Excel answers from a real pivot table), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it. The cells score only once T-2081 (model) and T-2084 (adapter loads the baked pivot) are merged too; T-2084 carries the key for the same cells.
