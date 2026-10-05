---
id: T-0217
title: "MIN/MAX: a non-numeric text value argument is #VALUE! (5 gen-m2 known failures)"
milestone: M2
lane: functions
status: todo
priority: P2
size: S
tier: sonnet
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
- [ ] The 5 cells above disappear from eval/baseline/gen-m2.txt with no new F lines.
- [ ] Unit tests: `MIN("a")`, `MAX(CONCATENATE("x"))` give `#VALUE!`. `MIN("2")` gives 2. A range with text is ignored.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Reference-vs-value results of IF/INDEX (T-0216).
