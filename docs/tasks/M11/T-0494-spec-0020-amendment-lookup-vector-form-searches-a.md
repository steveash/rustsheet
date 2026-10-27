---
id: T-0494
title: "SPEC-0020 amendment: LOOKUP vector form searches a tailed lookup_vector (corpus-bench 33935 under 1 s)"
milestone: M11
lane: perf
status: todo
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
- [ ] Equivalence tests (tailed vs `TAILED_OFF`) with an error tail, an empty tail, a numeric tail (refused), the
      array form (refused) and a lifted lookup_value.
- [ ] corpus-bench 33935 under 1 s through the harness; the bench line's before/after in the Outcome.
- [ ] `rustsheet-eval check` 0 regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
- A searched-kind tail (a run of equal keys at the end).
