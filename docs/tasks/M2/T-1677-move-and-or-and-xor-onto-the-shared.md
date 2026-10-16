---
id: T-1677
title: "Move AND, OR and XOR onto the shared aggregate-argument walker"
milestone: M2
lane: quality
status: todo
priority: P2
size: S
tier: opus
spec: docs/specs/SPEC-0009-aggregate-argument-walker.md
depends_on: []
---

## Goal
AND, OR and XOR read their arguments through the SPEC-0009 walker (`functions/util/aggregate_args.rs`) instead of
a dispatch of their own, with no change in any result. After this, AND and OR (M2 scope functions) no longer bypass
the one aggregate-argument iteration that M2's exit criterion asks for.

## Context
T-1474 and T-1476 moved SUM, the AVERAGE family, MIN, MAX, PRODUCT, COUNT and COUNTA onto the walker, and T-1567
moved VARA/VARPA/STDEVA/STDEVPA. `Model::logical_nary` (functions/logical/and_or_xor_not.rs, used by `fn_and`,
`fn_or`, `fn_xor`) still sorts each argument by shape itself: `evaluate_node_in_context`, its own
`for_each_cell_or_error_in_areas` walk of a range (with the "Ranges are in different sheets" check), its own loop
over array elements, and the first error returning at once. That is the eighth argument dispatch SPEC-0009's Problem
table did not list.

Behavior to keep (all of it must stay as today; this is a refactor, not a fix):
- First error in argument order wins, ranges and arrays row by row, no short-circuit
  (`ErrorRule::First`; docs/excel-semantics/coercion.md "AND / OR / XOR evaluate every argument"; probes/and-or-xor-error-order).
- A direct text "TRUE"/"FALSE" (any case) counts unless the argument node is a `ReferenceKind`; other text,
  referenced text and array text are skipped (probes/and-or-xor-text-arguments, T-0258). Map this onto
  `ItemSource` (Direct vs ReferencedCell vs ArrayElement) in a boolean visitor, as COUNT/COUNTA use a visitor.
- An empty argument counts as FALSE; empty cells are skipped; no logical value at all is `#VALUE!`.
- Whole-column arguments stay clipped to the used area (`OR(F:LSE)`, T-0230).
- The reading: today `evaluate_node_in_context`, not `evaluate_aggregate_arg`. If `ArgReading::Aggregate` would read
  a shape differently (for example a cell picked by IF/INDEX/XLOOKUP holding text "TRUE", which is direct today and
  would become a referenced cell, or a linked-workbook reference), keep today's result: add an `ArgReading` variant
  or a policy knob, document it in SPEC-0009's "As built" and "Consumer mapping", and file a probe task for the
  unprobed difference (decision 0051 milestone rules). Do not change a result without oracle evidence.
NOT (`fn_not`) is a single-argument element-wise lift, not an aggregate; it stays as is.

## Acceptance criteria
- [ ] `fn_and`, `fn_or` and `fn_xor` call `walk_aggregate_args` (or `walk_aggregate_arg`) with a `const` policy;
      `logical_nary` no longer matches `CalcResult::Range` or `CalcResult::Array` itself
      (`rg "CalcResult::Range|CalcResult::Array" crates/rustsheet-engine/src/functions/logical/and_or_xor_not.rs`
      finds only `fn_not`).
- [ ] SPEC-0009 "Consumer mapping" has a row for AND, OR, XOR (reading, errors, skip empty, visitor).
- [ ] Unit tests in the engine for: direct "true" text vs a referenced "TRUE" cell, an empty argument, an error
      after a decisive FALSE, and a whole-column argument on an empty sheet.
- [ ] `make eval`: 0 regressions and 0 fixes in every suite; probes/and-or-xor-error-order and
      probes/and-or-xor-text-arguments pass bit-exact.
- [ ] `scripts/check_duplication.py` does not rise (record a drop with `--update`, decision 0010).
- [ ] `make check` green; reviewers approve.

## Out of scope
- NOT, IF, IFS, SWITCH and other logical functions.
- MODE, GCD/LCM, MULTINOMIAL, SUBTOTAL and AGGREGATE argument handling (later-milestone functions; SUBTOTAL and
  AGGREGATE read references only).
- Any behavior change, including the unprobed reading differences above (file probe tasks instead).
