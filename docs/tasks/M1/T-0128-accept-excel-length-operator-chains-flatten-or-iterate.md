---
id: T-0128
title: "Accept Excel-length operator chains: flatten or iterate left-assoc chains past the depth cap"
milestone: M1
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: [T-0931]
---

## Goal
Formulas with long same-precedence operator chains (`=A1+A2+...+A2000`, legal in Excel up to its 8,192-character limit) parse and evaluate without a stack overflow, removing the chain part of the T-0931 DEVIATION.

## Context
T-0931 caps formula tree depth at `MAX_FORMULA_DEPTH` (256; `expressions/parser/depth.rs`) because the parser builds operator chains as left-nested binary nodes, so a chain of N terms is a tree N deep, and the evaluator, stringify, static analysis, move_formula and `Drop` all recurse over it. Measured on a 2 MiB thread before T-0931: release evaluation survived ~4,000-term chains, debug ~500. Options (performance reviewer, T-0931): an n-ary chain node (keeping left-to-right evaluation order, error precedence and float rounding identical to the binary fold), or making every consumer walk the left spine iteratively, including a custom `Drop`. See the "Formula nesting depth cap" entry in docs/excel-semantics/syntax.md.

## Acceptance criteria
- [x] `=1+1+...` with 4,096 terms (8,191 characters) evaluates to 4096 on a 1 MiB stack in an optimized build; the same for `&`, `*` and comparison chains.
- [x] Nesting depth (parentheses, calls) is still bounded as in T-0931; the semantics entry is updated.
- [x] Evaluation order and results are unchanged (eval 0 regressions).
- [x] `make check` green; reviewers approve.

## Out of scope
- Raising the parenthesis/function nesting cap.

## Outcome
Operator chains keep their binary tree shape, and every consumer walks a chain in a loop down its
left operands (`expressions/parser/chain.rs`: `left_chain`, `for_each_chain_operand_mut`): the
evaluator (`Model::evaluate_operator_chain`), both printers, static analysis, units, sheet and
name renames, LET/LAMBDA id assignment and `_xlpm.` export. `Node::clone` copies chains in a loop
too (the derived one overflowed a 1 MiB optimized stack at ~4,600 links). The parser builds all
five precedence levels with one loop (`Parser::parse_chain`, replacing five copies), and a chain
link adds no depth; instead a formula holds at most `MAX_VALUE_OPERATORS` (4,096) value operators,
which no 8,192-character Excel formula exceeds (tree drop still recurses: ~13,000 links fit on
1 MiB). Over the cap is a parse error the API refuses like a too-deep formula. 4,096-term `+ - *
^ & =` chains, mixed-precedence chains and chains inside 60 nested calls evaluate, print and move
on a 1 MiB stack in an optimized build (test_nesting_depth.rs). The syntax.md depth-cap entry is
updated. Eval delta: 0 fixed, 0 regressions.
