---
id: T-3197
title: "Nested (ABS( levels cost less parser depth, so Excel's 64 and 65 levels of (ABS( parse"
milestone: M2
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
RustSheet loads and evaluates probes/t3097-nesting-paren-abs-64 and -65 (stored and typed), 64
and 65 levels of `(ABS(`, which Excel accepts, with untrusted formulas still bounded (directive 5).

## Context
T-3191 (decision 0173) parses a run of directly nested parentheses in a loop, so Excel's 256 bare
parentheses fit, but a `(` followed by a call still recurses through the precedence chain and
charges an operand's two levels for the group and two for the call: 64 levels of `(ABS(` cost
256 levels plus the innermost operand, past `MAX_FORMULA_DEPTH` (depth.rs). Measured for T-3165
(release, 1 MiB thread): the parser overflows at about 112 nested `(ABS(`, so 65 fit the stack;
the bound, not the stack, refuses them. A group whose content is exactly one call adds one tree
node, not two, so it could cost one level (or share the call's), as long as the parser stack per
level stays covered (re-measure on a 1 MiB release stack and a 2 MiB debug stack, and
test_nested_formula_stack.rs). Both cases are pinned whole in eval/suites/probes/deviations
(decision 0168); their lines leave when they match.

## Acceptance criteria
- [x] The four t3097-nesting-paren-abs cases load and match Excel; their whole-case lines leave
      probes/deviations and the baseline records the passes.
- [x] At-limit tests on a 1 MiB release stack (test_nesting_depth.rs) and a 2 MiB debug stack cover
      the new bound; syntax.md updated.
- [x] `make check` green; reviewers approve.

## Out of scope
- Bare parenthesis runs (T-3191, done); unary sign runs (T-3193).

## Outcome
`Parser::parse_primary` bounds its recursion in half levels; the first operand of a group's
content, unless a group, reserves three (not four) and a plain call there adds one tree level
(decision 0184). A level of `(ABS(` costs 3.5, so 72 levels parse and 73 are too deep. Measured:
72 levels of `SUM((` need ~700 KB (release) and ~1.9 MiB (debug) to parse, evaluate, print, insert a
row and rename the sheet, below 127 levels of `ABS(` (746 KB / 1.97 MiB). Eval: the four
t3097-nesting-paren-abs cases (8 cells) leave probes/deviations and match Excel bit-exact; known
failures unchanged.
