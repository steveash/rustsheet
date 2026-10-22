---
id: T-2405
title: "A built-in function's result called directly: INDEX(A1:A3,1)(2), IF(TRUE,LAMBDA(x,x*2))(3), INDEX(s,1,3)(0)"
milestone: M7
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-2038]
---

## Goal
A built-in function call followed directly by `(...)` parses and evaluates as Excel does: `=INDEX(A1:A3,1)(2)` is `#REF!` and `=IF(TRUE,LAMBDA(x,x*2))(3)` is 6, while `=SUM(1)(2)` stays a syntax error.

## Context
T-2038 chained calls after a call and after a name (`Parser::chain_calls`, expressions/parser/lambda.rs) but not after a
built-in (`Node::FunctionKind`), because Excel's grammar depends on the function: Excel for the web refuses a workbook
holding `=SUM(1)(2)` (FileCorruptTryRepair) but evaluates `INDEX(A1:A3,1)(2)` (#REF!) and `IF(TRUE,LAMBDA(x,x*2))(3)` (6).
docs/excel-semantics/references.md "Calling a call's result". Probe which built-ins are callable first (one formula per
case file, since a refused formula loses its whole workbook): a hypothesis is the functions that can return a reference
or a LAMBDA (INDEX, IF, CHOOSE, OFFSET, INDIRECT, IFS, SWITCH, LET, XLOOKUP, IFERROR...), the set `is_reference_operand`
accepts for `(...)(...)`. Probe in the same pass the forms T-2038 widened without evidence: a chain after a
defined-name LAMBDA and after an unknown name (`=MYFN(1)(2)`, `=FOO(1)(2)`), and a call of a parenthesized value a function
returned (`=(IF(TRUE,1))(2)`, `=(INDEX({1,2},1))(2)`; ours `#NAME?`), plus a FORMULATEXT of a chained call. probes/lambda-escaping-closures H3 (`INDEX(s,1,3)(0)`, 123) also needs T-2039.

Failing cells, in eval/baseline/probes.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| probes/chained-calls-ref-4 Sheet1!B1 | `=INDEX(A1:A3,1)(2)` | #REF! | #ERROR! (parse) |
| probes/chained-calls-ref-6 Sheet1!B1 | `=IF(TRUE,LAMBDA(x,x*2))(3)` | 6 | #ERROR! (parse) |

## Acceptance criteria
- [ ] The two cells above pass and leave eval/baseline/probes.txt; the callable set is probed and recorded in references.md.
- [ ] `=SUM(1)(2)` stays a syntax error (unit test).
- [ ] `make check` green; reviewers approve.

## Out of scope
- LAMBDA values in arrays (T-2039).
