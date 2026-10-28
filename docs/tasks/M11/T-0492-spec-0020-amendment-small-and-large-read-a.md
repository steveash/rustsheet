---
id: T-0492
title: "SPEC-0020 amendment: SMALL and LARGE read a tailed argument (corpus-bench 59969 under 1 s)"
milestone: M11
lane: perf
status: in-progress
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
- [ ] Equivalence tests (tailed vs `TAILED_OFF`) for SMALL and LARGE with an error in the head and in the tail, and
      numeric, FALSE and empty tails; the RANK family keeps today's path.
- [ ] corpus-bench 59969 under 1 s through the harness; the bench line's before/after in the Outcome.
- [ ] `rustsheet-eval check` 0 regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other sorted-data consumers (PERCENTILE, MEDIAN, ...): filed when a corpus file needs them.
