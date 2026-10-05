---
id: T-0215
title: "AND/OR: no short-circuit, an error in any argument wins (20 gen-m2 known failures)"
milestone: M2
lane: functions-logic
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
AND and OR evaluate every argument. If any argument (or any cell of a range argument) is an error,
the result is that error, even after a FALSE (AND) or TRUE (OR) has been seen. Today `logical_nary`
short-circuits.

## Context
gen-m2 (T-0203): 20 failing cells need this, 17 of them together with a lookup fix. Evidence (excel-graph):
- `gen-m2/78bc88199fc96650` Calc!A217 `=OR(TRUE,(2<COUNTIFS(Data!A4:B9,"a*",Data!B3:E7,">=2")))`
  (COUNTIFS is `#VALUE!`, T-0209): expected `#VALUE!`, actual TRUE.
- `gen-m2/1b6ceed7489e0800` Calc!A143 `=OR(Data!B4:B8,((Data!E6-Data!C5)<>Data!B$6),...)` (C5 = "a*",
  so the second argument is `#VALUE!`): expected `#VALUE!`, actual TRUE.
- `gen-m2/96a0952412cefa6c` Calc!A126 `=IF(AND(Data!B3,AND(Data!A10:A11,FALSE),AND(Data!B4:B4)),...)`:
  expected `#VALUE!` (`AND` of a text-only range), actual FALSE.
- `gen-m2/75840b9c71a50596` Calc!A56 `=IF(TRUE,AND(Data!C2:C6,VLOOKUP(2.5,Data!B2:B8,2)),...)`: expected `#REF!`, actual FALSE.

Code: `crates/rustsheet-engine/src/functions/logical/and_or_xor_not.rs::logical_nary`
(`short_circuit_value`). XOR already passes `None`. Check whether its error order matches.

Cells whose only gen-m2 root cause is this one:
- `gen-m2/1b6ceed7489e0800.case.json`: A143
- `gen-m2/893302d6c5b99e2e.case.json`: A170
- `gen-m2/96a0952412cefa6c.case.json`: A126

These cells also need a lookup or criteria task:
- `gen-m2/2cfe279fe77a000d.case.json` A185 (also:  lookup)
- `gen-m2/61cacc22c4244179.case.json` A323 (also:  lookup)
- `gen-m2/75840b9c71a50596.case.json` A56 (also:  lookup)
- `gen-m2/78bc88199fc96650.case.json` A217 (also: ifs_shape)
- `gen-m2/893302d6c5b99e2e.case.json` A235 (also:  lookup)
- `gen-m2/893302d6c5b99e2e.case.json` A57 (also:  lookup)
- `gen-m2/942bdade381b7537.case.json` A194 (also:  lookup)
- `gen-m2/96a0952412cefa6c.case.json` A150 (also:  lookup)
- `gen-m2/96a0952412cefa6c.case.json` A206 (also:  lookup)
- `gen-m2/a6d4377c7b1b92cd.case.json` A270 (also:  lookup)
- `gen-m2/d04d501154afc771.case.json` A353 (also: index2d)
- `gen-m2/d0c29dd5c530e28c.case.json` A138 (also:  lookup)
- `gen-m2/d3e2df0bef2f173a.case.json` A135 (also:  lookup)
- `gen-m2/dd99be1f2454159f.case.json` A3 (also:  lookup)
- `gen-m2/dd99be1f2454159f.case.json` A6 (also:  lookup)
- `gen-m2/ef5c55a81783b7d5.case.json` A207 (also:  lookup)
- `gen-m2/f7e6d351c8538ef5.case.json` A312 (also:  lookup)

## Acceptance criteria
- [ ] The 3 cells above disappear from eval/baseline/gen-m2.txt with no new F lines. The 17 shared cells leave once their
      other task lands (check them after T-0209/T-0211/T-0212).
- [ ] Unit tests: error after a deciding value propagates, for scalars and for ranges.
- [ ] docs/excel-semantics/ entry (errors.md or a new logical.md) with the evidence above.
- [ ] `make check` green; reviewers approve.

## Out of scope
- -
