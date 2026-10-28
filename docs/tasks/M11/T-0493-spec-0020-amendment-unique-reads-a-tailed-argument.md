---
id: T-0493
title: "SPEC-0020 amendment: UNIQUE reads a tailed argument (corpus-bench 45896 under 1 s)"
milestone: M11
lane: perf
status: in-progress
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
- [ ] Equivalence tests (tailed vs `TAILED_OFF`) with `exactly_once` TRUE and FALSE, a tail equal to a head value,
      an all-tail result (#CALC! with exactly_once) and `by_col` TRUE (refused).
- [ ] corpus-bench 45896 under 1 s through the harness; the bench line's before/after in the Outcome.
- [ ] `rustsheet-eval check` 0 regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
- TEXTJOIN as a direct consumer (contract in the amendment; not needed by 45896).
