---
id: T-0493
title: "SPEC-0020 amendment: UNIQUE reads a tailed argument (corpus-bench 45896 under 1 s)"
milestone: M11
lane: perf
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
UNIQUE reads an element-wise array argument as a head and a tail (SPEC-0020 Amendment 1), so corpus-bench 45896
runs under 1 s with identical results.

## Context
corpus-bench 45896 (17.6 s): 27 `TEXTJOIN(",",TRUE,UNIQUE(IF(A2=ZORD!A:A,ZORD!C:C,"")))`. Contract in docs/specs/SPEC-0020-whole-column-array-operands.md
Amendment 1 section 3 "UNIQUE": `unique_of` over `head ++ [t, t]` equals it over `head ++ [t] * tail_len`;
tailed only when `by_col` and `exactly_once` are absent or literals; the lift cost stays the logical size.
Code: `fn_unique` (functions/spill_functions.rs). Bench: `aggregate -- tailed_consumers`, the TEXTJOIN/UNIQUE line.

## Acceptance criteria
- [x] Equivalence tests (tailed vs `TAILED_OFF`) with `exactly_once` TRUE and FALSE, a tail equal to a head value,
      an all-tail result (#CALC! with exactly_once) and `by_col` TRUE (refused).
- [x] corpus-bench 45896 under 1 s through the harness; the bench line's before/after in the Outcome.
- [x] `rustsheet-eval check` 0 regressions.
- [x] `make check` green; reviewers approve.

## Out of scope
- TEXTJOIN as a direct consumer (contract in the amendment; not needed by 45896).

## Outcome
- `fn_unique` reads its array through `read_tailed` when `by_col` and `exactly_once` are each absent, empty or a
  literal and `by_col` is not TRUE (`unique_flags_are_literals`), and runs `unique_of` unchanged on the head
  followed by two copies of the tail row; the lift cost is the logical size (`unique_array`).
- `Tailed::elements` is now `Tailed::elements_by` over any reading (the RefArg form delegates to it), keeping
  T-0491's `ROW()` sentinel refusal for every consumer.
- Tests (`test_tailed_array.rs`, tailed vs `TAILED_OFF`): `exactly_once` TRUE and FALSE, tails equal to a head
  value (empty, 0) and not, an all-tail `#CALC!`, the literal flag forms, a bounded span and a `ROW()` branch the
  tail does not pick (tailed); `by_col` TRUE or 1, reference, array and string flags, a bare range, a short
  range and a tail that picks `ROW()` (refused).
- corpus-bench 45896 through the harness: 17,860 ms → 6 ms (84/84 cells match their excel-graph values both before and after).
- `aggregate -- tailed_consumers`, TEXTJOIN/UNIQUE line: min 11.07 s → 38.1 ms (median 11.32 s → 39.8 ms).
- Eval delta: 0 regressions, 0 fixes (45896 already passed under the 60 s timeout; it is only faster).
