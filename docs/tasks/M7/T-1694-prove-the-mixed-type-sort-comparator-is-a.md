---
id: T-1694
title: "Prove the mixed-type sort comparator is a total order (sort_key_cmp, key_cmp) so sort_by cannot panic"
milestone: M7
lane: quality
status: todo
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
Sorting cells of mixed types (numbers, text, booleans, errors, blanks) can never panic: the comparators used with `sort_by` are proven strict total orders, or are wrapped so they become one.

## Context
Since Rust 1.81, `slice::sort_by` may panic when the comparator is not a total order. SORT, SORTBY, UNIQUE (`functions/spill_functions.rs` `sort_key_cmp`, which goes through `functions/util.rs` `compare_values`), and GROUPBY/PIVOTBY (`functions/lookup_and_reference/groupby.rs` `key_cmp`, `Grouping::sort_by_keys`, `sort_by_aggregate`) sort untrusted mixed-type cell values with these comparators. Nobody has shown that they are consistent (antisymmetric, transitive) across every type pair, NaN-free numbers, case-folded text and error kinds. The T-1323 security review (PR #916) flagged this; it predates GROUPBY. Directive 5: untrusted input must never panic.

## Acceptance criteria
- [ ] A property test (proptest or a seeded exhaustive generator over a small value pool covering every `ArrayNode` variant, case variants of text, -0/+0 and each error kind) checks antisymmetry and transitivity of `sort_key_cmp` and `key_cmp` in both directions.
- [ ] If any violation is found, the comparator is fixed so it matches Excel's documented order (docs/excel-semantics/arrays.md SORT and GROUPBY sections) or the sorts use a total-order wrapper; no evaluated SORT/UNIQUE/GROUPBY probe cell changes (`make eval-check` 0 regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing Excel-visible sort order (any change needs oracle evidence).
