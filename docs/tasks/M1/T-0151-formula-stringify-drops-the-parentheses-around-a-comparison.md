---
id: T-0151
title: "Formula stringify drops the parentheses around a comparison inside & (xlsx-imported formulas; 1 poi known failure)"
milestone: M1
lane: engine
status: in-progress
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
A formula with a comparison in parentheses as an operand of `&` keeps its meaning after the
engine prints it back as text. An imported `="its "&(2<3)&" blue"` must give "its TRUE blue", not
FALSE.

## Context
Evidence: `poi/FormulaEvalTestData_Copy.xlsx` EverythingTests!F27 `="its "&(2<3)&" blue"`. Desktop
Excel cached "its TRUE blue"; the engine gives FALSE (TypeMismatch). `make eval-show
CASE=poi/FormulaEvalTestData_Copy.xlsx` shows it.

What the gap triage found (2026-10-05):
- Typed into a spec case, the same formula is right ("its TRUE blue"). Only the xlsx path fails.
  A spec2xlsx workbook of these formulas, scored through the xlsx path, gives:
  `="a"&(2<3)` → FALSE, `=(2<3)&"b"` → TRUE, `=1&(2<3)&1` → TRUE, `="a"&(2=3)&"b"` → FALSE.
  `="a"&(1+1)&"b"` → "a2b" is right.
- So the xlsx path prints the parsed tree to text and parses it again, and the printer drops the
  parentheses. The importer stores formulas through `to_rc_format`
  (`crates/rustsheet-xlsx/src/import/worksheets.rs`). `"a"&(2<3)` comes back as `"a"&2<3`, which parses as `("a"&2)<3`.
- Suspected cause: in `crates/rustsheet-engine/src/expressions/parser/stringify.rs`, `stringify`
  prints `OpConcatenateKind` as `format!("{}&{}", sub(left), sub(right))` and never wraps a
  `CompareKind` operand. `&` binds tighter than the comparisons. `OpSumKind` and `OpProductKind`
  already wrap `CompareKind` operands (the match arms right after it); reuse that pattern. Do not
  write a second helper.
- Check the other low-precedence cases in the same function while there: a `CompareKind` on the
  right of a `CompareKind` (`=1<(2<3)`) and on either side of `&`. Every one needs a round-trip
  unit test.

## Acceptance criteria
- [ ] `F	poi/FormulaEvalTestData_Copy.xlsx	EverythingTests!F27` disappears from
      eval/baseline/poi.txt, with no new F lines in any suite.
- [ ] Unit tests in the stringify tests: parse → stringify → parse gives the same tree and value for
      `"a"&(2<3)`, `(2<3)&"b"`, `1&(2<3)&1`, `"a"&(2=3)&"b"` and `1<(2<3)`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other precedence or printing changes the evidence does not show (union, intersection,
  operator-chain depth: T-0128, T-0134, T-0148).
