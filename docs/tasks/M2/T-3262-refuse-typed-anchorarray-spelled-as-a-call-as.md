---
id: T-3262
title: "Refuse typed ANCHORARRAY spelled as a call, as Excel for the web's Graph entry does"
milestone: M2
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A formula typed through the API that spells `ANCHORARRAY(` as a call is refused (`EditError::InvalidInput`), as
Excel for the web refuses it typed, while the `#` operator and a stored `_xlfn.ANCHORARRAY(` keep working.

## Context
- T-3143 (docs/excel-semantics/syntax.md "Formula nesting depth cap", paragraph "`@` and `#`") found Graph PATCH
  refuses `=ANCHORARRAY(C1)` and `=SUM(ANCHORARRAY(C1))` typed (400 `InvalidArgument`) with C1 spilling, at any
  depth, while typed `=SUM(C1#)` is accepted and a stored `ANCHORARRAY(D1)` loads (probes/at-operand-formulatext J30).
- The parser turns a spelled `ANCHORARRAY(x)` into `Node::SpillRangeOperator` (expressions/parser/operands.rs);
  typed refusals go through `Parser::typed_refusal` (decision 0103 pattern, like `=SUM()`).
- Typed `SINGLE(` spelled as a call is accepted by Excel (probes/t3143-single-spelled-64-typed), so leave it.
- Desktop typed entry is unprobed (M14).

## Acceptance criteria
- [x] `Input::Formula("=ANCHORARRAY(C1)")` and `Input::Parse` of it are `EditError::InvalidInput`; the cell keeps
      its content. `Input::StoredFormula` and xlsx import still accept it.
- [x] Unit test cites probes/t3143-anchor-typed-* (dropped refusals) and t3143-anchor-inner-64-typed (accepted `#`).
- [x] `make check` green; reviewers approve.

## Out of scope
- Desktop Excel's typed entry.

## Outcome
Typed `ANCHORARRAY(` spelled as a call is refused via `Parser::typed_refusal` (new `spelled_anchorarray` flag); stored text and the `#` operator unchanged. Unit test in test_typed_parse_errors.rs. New oracle-filled probe `t3262-anchor-typed-refused` (excel-graph): typed `ANCHORARRAY(C1)`, `SUM(ANCHORARRAY(C1))` and `_xlfn.ANCHORARRAY(C1)` are refused, `SUM(C1#)` is accepted (12 cells pass).
