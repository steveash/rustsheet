---
id: T-0216
title: "IF and INDEX return references, IFERROR returns values, in aggregate arguments (8 gen-m2 known failures)"
milestone: M2
lane: functions
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
When IF (and INDEX) select a cell reference, aggregates (SUM, AVERAGE, COUNT, MIN, MAX, ...) treat the
result as a **reference**: text and blank cells are skipped, as for a direct range argument. IFERROR
returns a **value** (a blank cell becomes 0, text stays text).

## Context
gen-m2 (T-0203): 8 failing cells. Evidence (excel-graph):
- `gen-m2/96a0952412cefa6c` Calc!A88 `=ABS(SUM(IF(FALSE,(-2.5),Data!E10),Data!D4:D7))` (E10 = "abc"): expected 123456799.623, actual `#VALUE!`.
- `gen-m2/1b6ceed7489e0800` Calc!A206 `=(COUNT(Data!B1:B4,IF(Data!C6,Data!C$8,Data!$B12))/...)` (C8 = "1" text, not counted): expected -30000, actual -40000.
- `gen-m2/942bdade381b7537` Calc!A212 `=SUM(Data!E7:E7,INDEX(Data!B11:C11,1))` (B11 = " x "): expected 0.1, actual `#VALUE!`.
- IFERROR is a value: `gen-m2/caa981b0753f455c` Calc!A300 `...AVERAGE(IFERROR(Data!E7,Data!D7),IF(Data!A3,Data!E10),Data!C8:C9)`
  (E7 blank counts as 0, IF to "x" is skipped): expected -1.4999999999998498e-15, actual `#VALUE!`.
  `gen-m2/7469c3e920e1ade4` Calc!A351 `=IF(Data!E12,OR(IFERROR(Data!D9,Data!A9)))` (D9 blank gives 0, so FALSE): expected FALSE, actual `#VALUE!`.
- Other cells: `gen-m2/60a543744fc4a8f7` Calc!A200, `gen-m2/f0d1bacec5de8146` Calc!A183,
  `gen-m2/fa437879253827be` Calc!A372 (the last also needs T-0218).

Code: `crates/rustsheet-engine/src/functions/logical/mod.rs` (`fn_if`, `fn_iferror`), INDEX in
`lookup_and_reference/mod.rs`, and the shared aggregate-argument iterator from T-0200 (reuse it, do not
add a second classification).

## Acceptance criteria
- [ ] gen-m2 cells 96a0952412cefa6c A88, 1b6ceed7489e0800 A206, 942bdade381b7537 A212,
      caa981b0753f455c A300, 7469c3e920e1ade4 A351, 60a543744fc4a8f7 A200, f0d1bacec5de8146 A183
      disappear from eval/baseline/gen-m2.txt with no new F lines (fa437879253827be A372 after T-0218).
- [ ] docs/excel-semantics/ entry with the evidence above (IF/INDEX reference, IFERROR value).
- [ ] `make check` green; reviewers approve.

## Out of scope
- MIN/MAX on text values (T-0217).
