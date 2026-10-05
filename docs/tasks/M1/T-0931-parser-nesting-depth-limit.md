---
id: T-0931
title: "Parser: bound formula nesting depth (stack overflow on deeply nested parentheses)"
milestone: M1
lane: engine
status: done
priority: P0
size: S
tier: opus
depends_on: []
spec: docs/specs/SPEC-0001-public-api-v0.md
---

## Goal
A formula such as `=((((...1...))))` nested a few hundred levels deep must return a parse error, never crash the process.

## Context
Found while testing T-0104 through Python: `wb.set("A1", rs.Formula("=" + "(" * 500 + "1" + ")" * 500))` segfaults (stack overflow in the recursive-descent parser); 200 levels works. Violates CLAUDE.md prime directive 5 (untrusted input: no unbounded recursion) and SPEC-0001's test plan ("deeply nested formulas return `EditError`, never panic").

## Acceptance criteria
- [x] Parser enforces a maximum nesting depth (pick a limit that Excel's own 64-level function nesting cap sits well under; cite the evidence) and returns a typed parse error beyond it.
- [x] Also covers nested function calls, arrays `{}` and unary-operator chains; the evaluator's recursion over the resulting AST is bounded by the same limit.
- [x] Engine unit tests plus a facade test (`Workbook::set` with `Input::Formula` at depth 10_000 returns `EditError`).
- [x] `make check` green; reviewers approve.

## Out of scope
Matching Excel's exact error message for over-nested formulas.

## Outcome
- The parser bounds formula depth at `MAX_FORMULA_DEPTH` = 256 (`expressions/parser/depth.rs`):
  each operand costs 2 levels and each operator 1, checked top-down before every operand (bounds
  the parser's own recursion) and bottom-up as each node is built (bounds the tree every consumer
  recurses over: evaluator, stringify, static analysis, drop). Over the limit is a parse error
  ("Formula is nested too deeply"); `Model::set_user_input` / `update_cell_with_formula` reject it,
  so the facade returns `EditError::InvalidInput`; workbook loads get a parse-error cell.
- Accepts 127 nested parentheses/calls (Excel's cap is 64 nested functions) and 255-term operator
  chains. Longer chains, which Excel accepts, are a DEVIATION (decision 0014), recorded in
  `docs/excel-semantics/syntax.md`.
- Measured before the fix on a 2 MiB thread: release builds survived ~800 nested calls and
  ~4,000-term chains, debug builds only ~120 and ~500 (about 23 KB of parser stack per nesting
  level in debug vs ~2 KB in release), so the limit is sized for optimized builds on a 1 MiB
  stack and the at-limit tests run on 1 MiB (release) / 16 MiB (debug) threads.
- Moved array-row and function-argument parsing to `parser/lists.rs` (deduplicating the two
  array-item matches) and the bound to `parser/depth.rs`; one `Model::parse_cell_formula` now
  replaces two copies of the parse-and-retry code.
- Eval: 0 regressions, 0 fixed (no eval formula is near the limit). Duplicated lines 8360 -> 8288.
