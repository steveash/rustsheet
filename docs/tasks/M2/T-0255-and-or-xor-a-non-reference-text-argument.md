---
id: T-0255
title: "AND/OR/XOR: a non-reference text argument that is not TRUE/FALSE is #VALUE! (1 poi known failure + T-0254 probe)"
milestone: M2
lane: functions-logic
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-0254]
---

## Goal
AND, OR and XOR treat a text argument that does not come from a reference exactly as the T-0254 probe
shows (expected: `#VALUE!` unless it reads as TRUE/FALSE), so `AND("1",1)` is `#VALUE!` like Excel.

## Context
Evidence: poi/FormulaEvalTestData_Copy.xlsx EverythingTests!J136 `AND("1",1)`, desktop Excel cached
`#VALUE!`, engine TRUE (known failure). The full rule per function and per argument kind (literal vs
function-returned text, "TRUE" text, empty text) comes from the T-0254 probe and its
docs/excel-semantics/coercion.md entry; implement only what that entry marks VERIFIED.

Code: `logical_nary` in `crates/rustsheet-engine/src/functions/logical/and_or_xor_not.rs`, the
`CalcResult::String(..)` arm. Today a non-reference string is folded when `get_boolean` accepts it and
silently ignored otherwise; references and array/range text stay ignored (unchanged). One rule in that
arm for all three functions unless the probe shows they differ.

## Acceptance criteria
- [ ] `F	poi/FormulaEvalTestData_Copy.xlsx	EverythingTests!J136` disappears from eval/baseline/poi.txt.
- [ ] Every known failure T-0254 baselined for probes/logical-text-args disappears from
      eval/baseline/probes.txt; no new F lines in any suite.
- [ ] Unit tests in crates/rustsheet-engine/src/test/test_fn_or_xor.rs (and the AND tests) pin the
      probed values.
- [ ] The coercion.md entry gets an `Implemented in:` line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Error precedence among several errors (T-0229).
- Whole-column walks (T-0230).
