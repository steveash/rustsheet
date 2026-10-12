---
id: T-1344
title: "One printer: to_string_moved reuses stringify precedence (SPEC-0007 step 5)"
milestone: M1
lane: engine
status: done
priority: P3
size: M
tier: opus
depends_on: [T-1340, T-1342]
spec: docs/specs/SPEC-0007-written-parentheses.md
---

## Goal
Cut/paste and move print the same text as FORMULATEXT: `to_string_moved` reuses `stringify`'s precedence rules through a reference-rewriting hook, and the remaining duplicate precedence tables (`is_atom`, the per-operator wrap lists) give way to one "wrap only if the re-parse differs" rule.

## Context
SPEC-0007 Rollout step 5. move_formula.rs has its own `moved_operator_text` (its Power, Sum, Concat and Compare arms never wrap) and unary table, and prints `SpillRangeOperator` and `LambdaCallKind` callees differently from stringify.

## Acceptance criteria
- [x] move_formula.rs has no precedence table of its own.
- [x] A test over the probe corpus: moving a formula by (0,0) prints the same text as `to_english_string`.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`to_string_moved` is now `stringify` with a `Rewrite::Move` hook that rewrites only cell and range
references (`moved_reference_text` in move_formula.rs); move_formula.rs keeps no printer or precedence
table of its own. Its old printer also mangled array constants (`{1,"p";2,"q"}` printed
`{{1;"p"},{2;"q"}}`), used English booleans and `,` in every locale, and dropped `[x]` LAMBDA parameters;
all gone. In stringify, `is_atom`, the unary-minus list and the per-operator wrap lists are replaced by one
`Binding` order (the parser's chain levels, then `%`, `-`, operands) and one rule: a left operand wraps when
looser, a right operand when no tighter. New probe probes/operand-wrap-text (excel-graph, 30 cells) shows Excel
adds no parentheses (`=SUM(A1)^2`, `=2^-A1`, `=2^A2%`, ...) where the old `^` table printed `=(SUM(A1))^2` and
`=2^(-A1)`; all 30 pass. Corpus test: moving each of the ~16.9k probe formulas by (0,0) prints `to_english_string`.
Eval delta: probes +1 case (30 cells, all passing), 0 fixed, 0 regressions in every suite.
