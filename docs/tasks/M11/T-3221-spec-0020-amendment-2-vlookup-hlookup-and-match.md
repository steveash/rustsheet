---
id: T-3221
title: "SPEC-0020 Amendment 2: VLOOKUP, HLOOKUP and MATCH over a tailed key as tailed interior nodes, with the continued lift"
milestone: M11
lane: perf
status: in-progress
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
- [ ] `tailed_leaves` takes keyed VLOOKUP, HLOOKUP and MATCH calls with the fixed-argument rules of section 1, and
      refuses every shape that section names.
- [ ] The continued lift record (section 2): kernel runs, per-call cap outcome and work-budget charges equal the
      `TAILED_OFF` path's, shown by counting tests for the merged and unmerged tail and at the cap boundary.
- [ ] The equivalence tests of the amendment's test plan for this step, in `test/test_tailed_array.rs`.
- [ ] A bench line for `SUM(--(VLOOKUP(A:A,..)=x))` in `aggregate -- tailed_consumers`, before and after numbers in the Outcome.
- [ ] The refused-shapes tests and the `ROW()`-leaf-in-key tests (picked and not picked) of the test plan, and a fixed
      `ReferenceKind` argument pointing at a cell in a cycle or in a spill evaluated during the head.
- [ ] A `DEVIATION` entry in `docs/excel-semantics/` (decision 0014) for the fallback's possible work-budget
      `#VALUE!` in place of the per-call cap's `#ERROR!` (Amendment 2 section 2), or a reference to an existing one.
- [ ] `rustsheet-eval check` 0 regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
- INDEX as a consumer (T-3222); XLOOKUP and XMATCH (T-3223), which keep today's path until T-3223 lands.
