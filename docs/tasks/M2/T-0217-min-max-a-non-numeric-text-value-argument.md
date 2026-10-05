---
id: T-0217
title: "MIN/MAX: a non-numeric text value argument is #VALUE! (5 gen-m2 known failures)"
milestone: M2
lane: functions-math
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
MIN and MAX return `#VALUE!` for a non-numeric text **value** argument (a literal, or a function
result such as CONCATENATE/IFERROR), as SUM already does. Text inside a referenced range stays ignored.
Today MIN/MAX ignore such text (`MIN("a")` gives 0).

## Context
gen-m2 (T-0203): 5 failing cells. Evidence (excel-graph):
- `gen-m2/dd99be1f2454159f` Calc!A73 `=MIN(IFERROR(CONCATENATE(Data!$A1,Data!A8),COUNTIF(Data!B5:C7,">=2")))`: expected `#VALUE!`, actual 0.
- `gen-m2/dd99be1f2454159f` Calc!A264 `=SUM(Data!B9:B9,MIN(IFERROR(Data!$B$5,Data!D9)))` (B5 = "a"): expected `#VALUE!`, actual 0.
- `gen-m2/f7e6d351c8538ef5` Calc!A14 `=MAX(IFERROR(IF(Data!D4,"ü"),Data!$E$7),Data!D7:D9)`: expected `#VALUE!`, actual 1.005.
- `gen-m2/2cfe279fe77a000d` Calc!A138 `=MIN(IF(VLOOKUP(...),CONCAT("ABC"),Data!C9))`: expected `#VALUE!`, actual 0.
- `gen-m2/04cfd82f54dbc2b2` Calc!A182 `=AVERAGE(Data!B10:B11,MIN(IFERROR(Data!C4,123456789.123),...),ABS(42))` (C4 = "a"): expected `#VALUE!`, actual 10.625.

Code: `crates/rustsheet-engine/src/functions/math_and_trigonometry/mathematical.rs` (`fn_min`, `fn_max`).
Route them through the T-0200 aggregate iterator the way SUM is, rather than adding a special case.
Check the other T-0200 users (AVERAGE, COUNT?) with the same probe and file follow-ups if any differ.

## Acceptance criteria
- [x] The 5 cells above disappear from eval/baseline/gen-m2.txt with no new F lines.
- [x] Unit tests: `MIN("a")`, `MAX(CONCATENATE("x"))` give `#VALUE!`. `MIN("2")` gives 2. A range with text is ignored.
- [x] `make check` green; reviewers approve.

## Out of scope
- Reference-vs-value results of IF/INDEX (T-0216).

## Handoff (resumed)
Superseded by the Outcome below (the helper and the regressions it describes are gone).

Tried: `fn_min`/`fn_max` fallback arm now coerces value args via `cast_to_number` (new helper
`min_max_value_arg`; single-cell `Node::ReferenceKind` args stay ignored). Unit test
`test_fn_min_max_text_value_arguments` passes and the 5 target cells are fixed, but `eval-check`
reports 3 REGRESSIONS in gen-m2: `207a5753da48c7e0` Calc!A278, `f0d1bacec5de8146` Calc!A298,
`fa437879253827be` Calc!A312. All three pass `IF(cond, ref)` or `INDEX(...)` into MIN/MAX where
Excel treats the result as a reference (text skipped), but our IF/INDEX return values, so the text
is now coerced to #VALUE!. This is exactly T-0216 (IF/INDEX return references). Suggested next
step: land T-0216 first (or do both together), then re-apply this change; cannot be fixed without
reference-preserving IF/INDEX, and the baseline may not take regressions.

## Outcome
Resumed from the handoff on top of T-0216 (IF/INDEX hand aggregates references via
`evaluate_aggregate_arg`), which removes the 3 regressions the handoff hit. MIN and MAX now share
one `fold_min_max`: ranges go through `for_each_number_in_range`, arrays keep only numbers, and a
value argument is coerced with `cast_to_number` like SUM's. New probe `probes/min-max-value-args`
(16 cells, excel-graph) confirmed text/boolean/empty-arg coercion and array skipping; recorded in
docs/excel-semantics/coercion.md. Eval: gen-m2 known failures 873 → 868 (the 5 target cells),
poi 933 → 931, probes +16 new cells passing and dates-and-blanks A15 (`MAX("3",1)`) fixed; 0 regressions.
Follow-up: T-0236 (AVERAGE family array booleans / empty args).
