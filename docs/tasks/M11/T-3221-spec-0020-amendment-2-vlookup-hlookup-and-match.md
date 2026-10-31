---
id: T-3221
title: "SPEC-0020 Amendment 2: VLOOKUP, HLOOKUP and MATCH over a tailed key as tailed interior nodes, with the continued lift"
milestone: M11
lane: perf
status: done
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
A VLOOKUP, HLOOKUP or MATCH call whose `lookup_value` is a tailed operand (a whole column, a bounded span past the used
area, or an expression of them) is read as a head and a tail by every existing SPEC-0020 consumer, with bit-identical
results and the same lift charges (`SUM(--(VLOOKUP(A:A,..)=x))` does work proportional to the used rows).

## Context
Design: docs/specs/SPEC-0020-whole-column-array-operands.md "Amendment 2", sections 1, 2 and 4 (Rollout step 1). Found by T-3217 from probes/t3096-lookup-value-blank-elements
(4.8 s; its 32 whole-column-key formulas take about 160 ms each).
Code: `tailed_array.rs` (`push_operands`, `tailed_leaves`, `TailView`), `functions/lift/driver.rs` (`lift_grids`,
`count_walks`), `functions/lookup_and_reference/lookup_each.rs`, `index_arg.rs` (`lookup_head`,
`argument_error_each`), `mod.rs` (`fn_match`).

## Acceptance criteria
- [x] `tailed_leaves` takes keyed VLOOKUP, HLOOKUP and MATCH calls with the fixed-argument rules of section 1, and
      refuses every shape that section names.
- [x] The continued lift record (section 2): kernel runs, per-call cap outcome and work-budget charges equal the
      `TAILED_OFF` path's, shown by counting tests for the merged and unmerged tail and at the cap boundary.
- [x] The equivalence tests of the amendment's test plan for this step, in `test/test_tailed_array.rs`.
- [x] A bench line for `SUM(--(VLOOKUP(A:A,..)=x))` in `aggregate -- tailed_consumers`, before and after numbers in the Outcome.
- [x] The refused-shapes tests and the `ROW()`-leaf-in-key tests (picked and not picked) of the test plan, and a fixed
      `ReferenceKind` argument pointing at a cell in a cycle or in a spill evaluated during the head.
- [x] A `DEVIATION` entry in `docs/excel-semantics/` (decision 0014) for the fallback's possible work-budget
      `#VALUE!` in place of the per-call cap's `#ERROR!` (Amendment 2 section 2), or a reference to an existing one.
- [x] `rustsheet-eval check` 0 regressions.
- [x] `make check` green; reviewers approve.

## Out of scope
- INDEX as a consumer (T-3222); XLOOKUP and XMATCH (T-3223), which keep today's path until T-3223 lands.

## Outcome
- `tailed_leaves` takes VLOOKUP and HLOOKUP (3-4 arguments) and MATCH (2-3) as interior nodes whose one operand is
  the key. The key must reach a leaf, and the fixed arguments must be a range, a one-cell reference or an array
  literal for the table, and a literal or a one-cell reference for the others (`fixed_arguments`). Everything else
  is refused.
- The continued lift is `Model::lift_values_keyed` (functions/lift/keyed.rs). The head lift records a `LiftRecord`
  in the `TailView`: its up-front visits, its last element's slots and that element's result. A merged tail reuses
  the result and charges nothing. An unmerged tail checks the per-call cap on head + tail visits and charges only
  its own (`lift_grids_after` with `prior`).
- Tests in `test/test_tailed_array.rs` cover equivalence (`TAILED_OFF` against tailed) and refused shapes. They
  include `ROW()` in a key, picked and not, and fixed references in a cycle or spill. Kernel runs and up-front
  visits (`LIFT_TALLY`) equal the one lift's, merged and unmerged, across both kernels. The per-call cap is tested at
  its boundary (4,096 walks × 2^15 scan), merged and unmerged.
- Bench `aggregate -- tailed_consumers`, `SUM(--(VLOOKUP($A:$A,Data!$A$1:$B$51,2,FALSE)=x))`, 20 calls over 2,000
  rows: median 3.23 s on origin/main and 244 ms after (min 3.09 s and 221 ms). The other three lines are unchanged.
- DEVIATION entry: docs/excel-semantics/lookup.md, "A lookup over a whole-column key read as a head and a tail: the
  limits it meets".
- Eval: `rustsheet-eval check` shows 0 regressions and 0 fixes in every suite. probes/t3096 still passes, and its
  INDEX-wrapped keys wait for T-3222.
