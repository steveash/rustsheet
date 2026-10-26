---
id: T-2781
title: "SPEC-0023 step 1: a Spaced node keeps typed operand whitespace in formula text"
milestone: M2
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: [T-2776]
---

## Goal
A formula typed with whitespace prints back with it, as Excel's FORMULATEXT does: `=1 + 2`, `=SUM( 1 )`,
`=- 5`, `=+ 5`, `=@ B1:B2`, `=LET(x, 1, x + 1)`. Values do not change.

## Context
SPEC-0023 step 1. Read its Design section first: a transparent `Node::Spaced { space: Box<Spacing>, child }`
attached at the primary level, peeled with `Group` in `ungrouped`/`lowered`, printed by every printer.
Evidence: docs/excel-semantics/syntax.md "Formula text keeps typed whitespace…". Owned failing cells:
probes/t2769-formula-text-whitespace C1:C5, C7, C8, C10:C15, C17:C20, C22 (line-feed runs may move to
T-2782); probes/t2723-unary-plus-formula-text C24, C25; probes/t2776-formula-text-whitespace-shapes C1, C2,
C7:C13, C16:C18; probes/t2776-ws-after-at C1.

## Acceptance criteria
- [ ] The cells above pass (or, for a line-feed run, are handed to T-2782 in the Outcome); 0 regressions.
- [ ] `NODE_SIZE` unchanged; the T-1169 depth tests pass unchanged and gain a spaced `( ( … ) )` shape.
- [ ] Unit tests: print(parse(f)) == f for each probed typed shape; a `Spaced` operand in a chain round-trips
      through `clone()`; a row insert and a sheet rename keep the spaces.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Line-feed run normalisation (T-2782), intersection spaces (T-2783), stored-text leading runs (T-2784).

## Outcome
- `Node::Spaced { space: Spacing, child }` (`expressions/parser/spacing.rs`): runs of spaces kept before an operand
  (after `=`, an operator, a sign, `(`, a comma, `@`), after one before an operator, `%` or `)`, dropped at the end,
  before a comma, in an array constant and around `:`; the space count wraps at 256. Printed by every printer;
  structural edits, renames and moves keep it. `NODE_SIZE` unchanged.
- Deviation from the spec's design, recorded in SPEC-0023 "Implementation note": a cell stores the written tree and a
  stripped copy (`model::ParsedFormula`, `Node::strip_spaces`), and evaluation, units and static analysis read the
  copy, because ~300 argument-shape sites match nodes without peeling; defined names, conditional formats and
  outside-cell formulas are stripped where parsed. A `Spaced` costs no depth level; the
  nesting tests gain spaced shapes (`( `, `SUM( - `, `IF( TRUE, `…) printed, edited and evaluated at the limit. The xlsx export still writes the stripped tree (T-2786 decides).
- Stored text: a file's `<f>` drops its leading whitespace (`Parser::loaded`, spaces only), so t2776-shapes E7/E8 keep passing and
  E1, E2, E10 (T-2784's) pass now; T-2784 is left with nothing in E-cells beyond E6/E9 (line feed, intersection).
- Eval: probes known failures 2690 -> 2654 (36 fixed, 0 regressions): t2769 C1:C5, C7, C8, C10:C15, C18:C20, C22;
  t2723 C24, C25; t2776-shapes C1, C2, C7:C12, C16:C18, C21, E1, E2, E10, F21; t2776-ws-after-at C1. Every other
  suite unchanged.
- Handed to T-2782 (line feeds): t2769 C17, t2776-shapes C13 (and C19, C20, C22, E6 as planned).
- LAMBDA parameter names drop the whitespace around them (unprobed; noted in syntax.md).
