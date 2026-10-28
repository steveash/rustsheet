---
id: T-0493
title: "SPEC-0020 amendment: UNIQUE reads a tailed argument (corpus-bench 45896 under 1 s)"
milestone: M11
lane: perf
status: todo
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

## Handoff
Nothing is implemented yet. The previous run claimed this task near the end of its session budget and could not
delete the claim branch (the push of a deletion was refused). Start from `origin/main` once T-0491 (PR #2110) has
merged: T-0491 changes `Tailed::elements` in `tailed_array.rs` (the `positions` sentinel check), which this task
generalizes.

Suggested approach (SPEC-0020 Amendment 1, section 3, "UNIQUE"):
- Generalize `Tailed::elements` to any reading (for example `elements_by(self, as_array)`), keeping the
  RefArg form for the aggregate walk and SUMPRODUCT.
- In `fn_unique`, when `by_col` and `exactly_once` are absent or literals (and `by_col` is not TRUE), read
  `args[0]` with `read_tailed(.., |m| m.eval_to_array_keep_blank(&args[0], cell))`.
- Build `head ++ [t, t]`, or just the head when there is no tail, and charge `LiftCost::Cells` at the logical
  size.
