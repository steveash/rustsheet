---
id: T-0157
title: "Formula stringify drops the parentheses around a & operand of + - * /"
milestone: M1
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A formula with a `&` expression as an operand of `+`, `-`, `*` or `/` keeps its meaning after the
engine prints it back as text: `=("a"&1)+1` must not come back as `="a"&1+1`.

## Context
Found by the T-0151 fidelity review. `&` binds looser than `+ - * /`, but the `OpSumKind` and
`OpProductKind` arms of `stringify` in `crates/rustsheet-engine/src/expressions/parser/stringify.rs`
wrap only `OpSumKind` and `CompareKind` operands, never `OpConcatenateKind`. The printed text then
re-parses to a different tree (`"a"&(1+1)`). Use the existing `compare_in_parens`-style closure
rather than a new helper. Excel's operator precedence decides the cases; check each against
oracle evidence or a spec case.

## Acceptance criteria
- [ ] Unit tests: parse -> print -> parse gives the same tree for `("a"&1)+1`, `1+("a"&1)`,
      `("a"&1)*2` and `2/("a"&1)`.
- [ ] No new F lines in any eval suite.
- [ ] `make check` green; reviewers approve.

## Out of scope
- A general precedence-based parenthesiser; other operators (T-0128, T-0134, T-0148).

## Outcome
`stringify` now parenthesises a `&` operand of `+ - * /` (left and right, and the right of `-`),
via a `loose_in_parens` closure that extends `compare_in_parens` with `OpConcatenateKind`. Unit test
`concatenation_operands_of_arithmetic_keep_their_parentheses` checks print/re-parse round trips.
Eval delta: none (no suite changed).
