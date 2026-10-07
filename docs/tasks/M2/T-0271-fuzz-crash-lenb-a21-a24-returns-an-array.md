---
id: T-0271
title: "Fuzz crash: LENB(A21:A24) returns an array into a scalar cell (static analysis says not_implemented)"
milestone: M2
lane: functions-text
status: done
priority: P0
size: S
tier: sonnet
depends_on: []
---

## Goal
No formula can make a scalar-context cell receive a larger-than-1x1 array: `=LENB(A21:A24)` evaluates
like Excel (spills in a dynamic-array formula, like `LEN`), and the fuzz target `formula_eval` no
longer crashes on it.

## Context
Found by the first working nightly fuzz run (T-1067; Actions run 37586797587, job `fuzz
(formula_eval)`, artifact `fuzz-crash-formula_eval-37586797587`). Reproducer (13 bytes, the formula
without `=`): `LENb(A21:A24)` — base64 `TEVOYihBMjE6QTI0KQ==`. Reproduce:
`cd fuzz && cargo fuzz run --target "$(rustc -vV | sed -n 's/^host: //p')" formula_eval <file>`.

Panic: the `debug_assert!` "Larger-than-1x1 array reached scalar-context cell" in
`crates/rustsheet-engine/src/model.rs` (`set_cells_with_result`, ~line 967). Release builds return
`#VALUE!` there instead, so it is also a wrong result, not only a debug crash.

Cause: `static_analysis.rs` has `Function::Len => scalar_arguments(args)` but
`Function::Lenb => not_implemented(args)`, so no implicit intersection is inserted / no spill is
planned, while `fn_lenb` lifts over the range at runtime. Any other function whose static analysis
is `not_implemented` but whose implementation lifts over arrays has the same bug.

## Acceptance criteria
- [x] `Function::Lenb` gets the same static analysis as `Len`; `=LENB(A21:A24)` matches Excel
      (cite or probe: LENB is LEN for en-US single-byte text, docs/excel-semantics/text.md).
- [x] Audit every `not_implemented(args)` entry in `static_analysis.rs` against functions that lift
      at runtime; fix each one found (or file one P0 per distinct function if out of this lane).
- [x] The reproducer is a unit test (decision 0035); `make check` green; reviewers approve.

## Out of scope
- The `debug_assert!` itself stays: it caught a real static-analysis gap.

## Outcome
FINDB, LEFTB, LENB, MIDB, REPLACEB, RIGHTB and SEARCHB delegate to their non-B twins at runtime, so
`static_analysis.rs` now gives them `scalar_arguments` like those twins (they were `not_implemented`,
so no spill was planned and the scalar-context `debug_assert!` fired). Unit test
`byte_functions_spill_like_the_non_byte_functions` includes the reproducer `=LENB(A21:A24)`.
Audit: a throwaway debug-build test ran 60 other functions that are `not_implemented` in
`static_analysis.rs` (T, TEXTAFTER/BEFORE, UNICODE, CHAR, CODE, VALUE, DATEVALUE, HOUR, DAYS, the
financial and conversion functions, ...) with range arguments in 1-4 argument positions; none hit the
assert, while the same test caught `LENB` with the fix reverted. Functions not in that list (the D*
database functions, lookups, IFS/SWITCH, aggregates) were not probed; no further instance found.
Excel evidence: probes/text-b-functions-broadcast (oracle excel-graph, 29/29 match); recorded in
docs/excel-semantics/text.md "The *B text functions equal their non-B twins and spill". The unaudited
remainder is T-0272.
