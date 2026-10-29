---
id: T-0494
title: "SPEC-0020 amendment: LOOKUP vector form searches a tailed lookup_vector (corpus-bench 33935 under 1 s)"
milestone: M11
lane: perf
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
LOOKUP's vector form searches an element-wise `lookup_vector` as a head and a tail whose element the search
skips (SPEC-0020 Amendment 1), so corpus-bench 33935 runs under 1 s with identical results.

## Context
corpus-bench 33935 (8.6 s): 29 `IF(L3>1,LOOKUP(2,1/(J:J<>""),J:J))`. Contract in docs/specs/SPEC-0020-whole-column-array-operands.md Amendment 1 section 3
"LOOKUP, vector form": with a result_vector and a tail of no `value_kind` (error or empty), the search vector is
the head with the virtual empty tail `SearchData` already has (`len` past `values`); a searched-kind tail and the
array form keep today's path; vector checks use the logical size. Code: `lookup_search` (functions/
lookup_and_reference/lookup.rs), `SearchVector` (lookup_table.rs). Bench: `aggregate -- tailed_consumers`, the
LOOKUP line.

## Acceptance criteria
- [x] Equivalence tests (tailed vs `TAILED_OFF`) with an error tail, an empty tail, a numeric tail (refused), the
      array form (refused), a lifted lookup_value, and a target whose kind differs from the head's last
      element (the search's midpoints).
- [x] corpus-bench 33935 under 1 s through the harness; the bench line's before/after in the Outcome.
- [x] `rustsheet-eval check` 0 regressions.
- [x] `make check` green; reviewers approve.

## Out of scope
- A searched-kind tail (a run of equal keys at the end).

## Outcome
- LOOKUP's vector form (with a result_vector) reads its `lookup_vector` through `read_tailed` first
  (`Model::tailed_lookup_search`, functions/lookup_and_reference/lookup.rs). When the tail element is of no searched kind
  (an error such as `1/FALSE`, or an empty element), the search vector is the head followed by `SearchVector`'s virtual
  empty tail, with the materialized array's logical length. The lifted search's cost is that logical length too. A tail
  of a searched kind (a number, FALSE, text), an untailed shape and the array form keep today's path.
- `lookup_table::is_searched_kind` exposes the approximate search's kind test without exporting `ValueKind`.
- Tests (`test_tailed_array.rs`, tailed vs `TAILED_OFF`):
  - Tailed: error and empty tails; number, text and boolean targets, some of a kind the head's last element is not; a
    target below every key; a lifted lookup_value; a bounded span.
  - Not tailed: numeric, FALSE and text tails (identical results) and the array form (0 tailed reads).
- corpus-bench 33935 through the harness (`rustsheet-eval run --suite corpus-bench --filter 33935`, release):
  7.8 s → 0.41 s wall, 58/58 cells pass before and after.
- `aggregate -- tailed_consumers`, LOOKUP line (same machine): min 4.370 s → 8.7 ms (median 4.398 s → 9.6 ms).
- Eval delta: 0 regressions, 0 fixes (33935 already passed; it is only faster).
