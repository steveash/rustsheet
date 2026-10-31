---
id: T-3466
title: "Move lower_structured_reference out of expressions/parser/operands.rs (997 of its 1,000 lines) into its own parser module"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-3260]
---

## Goal
`crates/rustsheet-engine/src/expressions/parser/operands.rs` has room under the 1,000-line file limit again
(`scripts/check_file_size.py`, `LIMIT = 1000`), so the next parser fix that touches an operand does not first have
to split the file. The structured-reference lowering moves, unchanged, into a module of its own.

## Context
- Planner run 2026-10-31 (`make eval` on origin/main): operands.rs is at 997 lines (`wc -l`), 3 under the limit,
  and it is not in quality/oversized-files.json, so the gate fails any PR that adds 4 lines to it. Open work that
  edits it: T-3262 (typed `ANCHORARRAY(` refusal: "The parser turns a spelled `ANCHORARRAY(x)` into
  `Node::SpillRangeOperator` (expressions/parser/operands.rs)") and the in-flight branch of T-3260
  (`agent/harness/T-3260-headerless-tables`, which rewrites the `#Totals`/`#Headers` arms of
  `lower_structured_reference`). This task depends on T-3260 so the two never conflict: move the code as it is
  after T-3260 merges.
- Same pattern as the pending splits of parser/mod.rs and reference_operators.rs filed by the planner on
  2026-10-31 (T-3440, T-3439) and as T-3377 and T-3313.
- What moves: `Parser::lower_structured_reference` (the `fn` at about line 751 to the end of `impl Parser`, about
  240 lines), as an `impl Parser` block in a new `expressions/parser/structured_lowering.rs` declared in
  parser/mod.rs. Its doc comment and line comments move with it. `structured_reference_operand` (which calls it)
  stays in operands.rs; the moved method becomes `pub(super)`. `result_of_even_let` (the free `fn` at the end of
  operands.rs) stays: only `parse_named_call` uses it.
- Imports the moved code needs are imported in the new file, never copied (directive 3); operands.rs drops the
  imports only the moved code used (clippy's unused-import lint will say which).

## Acceptance criteria
- [ ] `lower_structured_reference` lives in `expressions/parser/structured_lowering.rs` with its body unchanged
      (a reviewer can diff it line by line); operands.rs no longer defines it.
- [ ] operands.rs is at most 780 lines; the new file is under 400.
- [ ] `git diff --stat` shows only operands.rs, the new file and the one `mod` line in parser/mod.rs.
- [ ] `make eval-check` shows no cell changed (0 fixed, 0 regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any behavior change in structured-reference lowering.
- Splitting parser/mod.rs or reference_operators.rs (T-3440, T-3439).
