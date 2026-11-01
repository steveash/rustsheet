---
id: T-3222
title: "SPEC-0020 Amendment 2: INDEX with a scalar row as a consumer of a tailed argument (probes/t3096-lookup-value-blank-elements under 0.5 s)"
milestone: M11
lane: perf
status: done
priority: P3
size: S
tier: opus
depends_on: [T-3221]
---

## Goal
INDEX with scalar indices over a tailed argument picks its element from the head or the tail without building the
logical array, so probes/t3096-lookup-value-blank-elements runs in under 0.5 s (4.8 s today) with the same values.

## Context
Design: docs/specs/SPEC-0020-whole-column-array-operands.md "Amendment 2", section 3 "INDEX with scalar indices" (Rollout step 2). Needs T-3221's keyed lookups for the t3096
shapes (`INDEX(VLOOKUP(Keys!$A:$A,..),1000)`, `INDEX(MATCH(Keys!$A:$A,..,1),2)`).
Code: `functions/lookup_and_reference/index.rs` (`index_impl`, `index_source`, `index_from_array`), `tailed_array.rs`.

## Acceptance criteria
- [x] The INDEX consumer of section 3, in `index_source`'s ordinary reading, with the fallback for every other index.
- [x] The INDEX equivalence tests of the amendment's test plan.
- [x] probes/t3096-lookup-value-blank-elements under 0.5 s through the harness (time before and after in the Outcome),
      and an `INDEX(MATCH(A:A,..),n)` bench line.
- [x] `rustsheet-eval check` 0 regressions.
- [x] `make check` green; reviewers approve.

## Out of scope
- Lifted INDEX indices; IF and CHOOSE forms that `pick_reference` resolves.

## Outcome
- `Model::index_tailed` (index.rs) handles INDEX with `area_num` 1, a scalar row of 1 or more, and `column_num` 1 or
  the two-argument form. It reads INDEX's own first argument through `read_tailed`, the same reading as
  `index_source`, and picks the row's element from the head or the tail with `Tailed::element_at`. That element is
  converted by `index_from_array` exactly as an element of the whole array would be: a value with column 1, a 1x1
  array in the two-argument form.
- Every other index falls back to today's reading, and so does any argument that is not a one-column tailed
  argument. That covers row 0, row past the logical rows, column 0 or 2, area 2, lifted indices, an IF or CHOOSE
  that `pick_reference` resolves, and the `ROW()` sentinel tail. With no tail (the pass is being abandoned), a row
  past the head is `#REF!`.
- Tests in `test_tailed_array.rs`: `index_picks_from_a_tailed_argument` (head, tail and the logical last row, every
  form, six argument kinds) and `index_forms_that_fall_back` (every fallback index, an error key element in the
  picked row, `#REF!` in the indices before an error array). Each formula is checked with `TAILED_OFF` and without.
- probes/t3096-lookup-value-blank-elements through `rustsheet-eval show` (release): 5.34 s with T-3221 alone and
  0.03 s after, still 203/203 passing.
- Bench `aggregate -- tailed_consumers`, `INDEX(MATCH($A:$A,Data!$A$1:$A$50,0),n)`, 20 calls over 2,000 rows: median
  2.07 s with T-3221 alone and 15.2 ms after. The other lines are unchanged.
- Eval: `rustsheet-eval check` shows 0 regressions and 0 fixes in every suite. lookup.md's head-and-tail limits
  entry now names INDEX.
- Resumed by an opus run after the branch sat idle 2 h with a conflicted merge of `main` (T-3221 squashed): the one
  `impl Tailed<RefArg>` block now holds `element_at` (the review's MINOR), and the panel re-reviewed the new head.
