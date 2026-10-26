---
id: T-2781
title: "SPEC-0023 step 1: a Spaced node keeps typed operand whitespace in formula text"
milestone: M2
lane: engine
status: todo
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
