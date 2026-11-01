---
id: T-0492
title: "SPEC-0020 amendment: SMALL and LARGE read a tailed argument (corpus-bench 59969 under 1 s)"
milestone: M11
lane: perf
status: done
priority: P2
size: S
tier: opus
depends_on: [T-0491]
---

## Goal
SMALL and LARGE read an element-wise array argument as a head and a tail (SPEC-0020 Amendment 1), so corpus-bench
59969 runs under 1 s with identical results.

## Context
corpus-bench 59969 (5.7 s): `IFERROR(INDEX(Report!G:G,SMALL(IF((Report!$A$2:$A$70000=Form!$B$3)*(Report!$B$2:$B$70000=
Form!$B$5),ROW(Report!$A$2:$A$70000)),ROWS($A$1:A1))),"")`. Contract in docs/specs/SPEC-0020-whole-column-array-operands.md Amendment 1 section 3
"SMALL and LARGE": the head's first error, else the tail's error, else the head's numbers and `tail_len` copies
of a numeric tail. Code: `ranked_numbers` (functions/statistical/count_and_average.rs) through
`numbers_from_reference_arg` (chisq.rs). Bench: `cargo bench -p rustsheet --bench aggregate -- tailed_consumers`,
the SMALL line (T-0488 Outcome has today's timing).

## Acceptance criteria
- [x] Equivalence tests (tailed vs `TAILED_OFF`) for SMALL and LARGE with an error in the head and in the tail, and
      numeric, FALSE and empty tails; the RANK family keeps today's path.
- [x] corpus-bench 59969 under 1 s through the harness; the bench line's before/after in the Outcome.
- [x] `rustsheet-eval check` 0 regressions.
- [x] `make check` green; reviewers approve.

## Out of scope
- Other sorted-data consumers (PERCENTILE, MEDIAN, ...): filed when a corpus file needs them.

## Outcome
- `ranked_numbers` (LARGE, SMALL) reads its argument through `Model::tailed_numbers_from_reference_arg` first
  (functions/statistical/chisq.rs): `read_tailed` with the ordinary `evaluate_reference_arg` reading, then one column
  of the head's elements and the tail element once through the ordinary kernels (`nested_array_error`,
  `values_from_array`), so the first error is the head's, else the tail's; a numeric tail is repeated `tail_len`
  times, any other tail adds nothing. Untailed shapes and the RANK family keep today's path.
- Tests (`test_tailed_array.rs`, tailed vs `TAILED_OFF`): SMALL and LARGE with k = 1, 3, 40, 5000 over errors in the
  head, an error tail with and without head errors, numeric, FALSE, empty and text tails, whole columns and bounded
  spans with `ROW()` (tailed); RANK, RANK.EQ, PERCENTRANK and a SMALL whose tail row picks `ROW()` (not tailed).
- corpus-bench 59969 through the harness (`rustsheet-eval run --suite corpus-bench --filter 59969`, release):
  6.2 s → 0.38 s wall, 162/162 cells pass before and after.
- `aggregate -- tailed_consumers`, SMALL line (same machine): min 510.8 ms → 18.6 ms (median 550.6 ms → 19.1 ms).
- Eval delta: 0 regressions, 0 fixes (59969 already passed; it is only faster).
