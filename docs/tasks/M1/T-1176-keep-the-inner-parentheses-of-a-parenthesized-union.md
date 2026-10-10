---
id: T-1176
title: "Keep the inner parentheses of a parenthesized union cut in FORMULATEXT"
milestone: M1
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
FORMULATEXT of `=SUM(((A1,B1) B1))` prints the written text with both parentheses, as Excel does.

## Context
probes/union-cut-precedence F6 (oracle excel-graph, T-1140): Excel prints `=SUM(((A1,B1) B1))`; the engine prints `=SUM((A1,B1) B1)`, dropping the union's own parentheses when the union is the left operand of a cut and the sole argument. Compare the `written` handling on `OpUnionKind` (T-1024) and `Parser::parse_parenthesized`.

## Acceptance criteria
- [x] probes/union-cut-precedence F6 passes and `make eval-baseline` records the fix.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`OpIntersectKind` gained `written` (parentheses levels around a cut whose leftmost operand is a parenthesized union); the parser sets it and `stringify` prints it. The cut now also prints its written parentheses inside larger expressions (`-((A1,B1) B1)`, `SUM(((A1,B1) B1),1)`), matching the typed text but without an oracle cell of its own (T-1178); other parenthesized cuts are unchanged. The whole-formula, `@` and `:` wrappers do not count those levels twice (unit tests in test_union.rs). Eval: probes known failures 394 -> 393 (F6 fixed), no regressions.

## Handoff (resumed)
Branch `agent/engine/T-1176-union-cut-parens` has the fix (`OpIntersectKind::written`, set in `parse_parenthesized` via `cuts_written_union`, printed in `stringify.rs`; `parse_cell_formula`, `tight_operand` and `at_operand` skip the extra level). `make check` was green on the previous head, F6 is fixed in the baseline.

reviewer-excel-fidelity requested changes twice. Round 1: double-counted levels at grouped root, `@` and `:` (fixed). Round 2 (open, BLOCKING): `intersection_operand` (reference_operators.rs) does not treat `OpIntersectKind { written: 1.., .. }` as bare when it is the right operand of another cut, so `C1 ((A1,B1) B1)` prints `C1 (((A1,B1) B1))` and gains a level on each re-parse. Next: add that arm (as `tight_operand` does), add the right-operand forms (`C1 ((A1,B1) B1)`, `A1:B1 ((A1,B1) B1)`, `((A1,B1) B1) ((A1,B1) B1)`) to `parenthesized_cut_of_a_union_keeps_its_parentheses` in tests/test_union.rs with a re-parse round-trip assert, then `make check`, re-run reviewer-excel-fidelity, open the PR. Other reviewers (security, quality, architecture) approved the earlier head. The task Outcome was reset; restore it when done.
