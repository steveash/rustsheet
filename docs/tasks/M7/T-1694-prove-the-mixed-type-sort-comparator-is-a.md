---
id: T-1694
title: "Prove the mixed-type sort comparator is a total order (sort_key_cmp, key_cmp) so sort_by cannot panic"
milestone: M7
lane: quality
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
Sorting cells of mixed types (numbers, text, booleans, errors, blanks) can never panic: the comparators used with `sort_by` are proven strict total orders, or are wrapped so they become one.

## Context
Since Rust 1.81, `slice::sort_by` may panic when the comparator is not a total order. SORT, SORTBY, UNIQUE (`functions/spill_functions.rs` `sort_key_cmp`, which goes through `functions/util.rs` `compare_values`), and GROUPBY/PIVOTBY (`functions/lookup_and_reference/groupby.rs` `key_cmp`, `Grouping::sort_by_keys`, `sort_by_aggregate`) sort untrusted mixed-type cell values with these comparators. Nobody has shown that they are consistent (antisymmetric, transitive) across every type pair, NaN-free numbers, case-folded text and error kinds. The T-1323 security review (PR #916) flagged this; it predates GROUPBY. Directive 5: untrusted input must never panic. Call sites: `functions/spill_functions.rs` (the SORT, SORTBY and UNIQUE `sort_by` calls, including the by-column variants) and `functions/lookup_and_reference/groupby.rs` (`sort_by_keys`, `sort_by_aggregate`); the test also covers `compare_values`, which `sort_key_cmp` delegates to. Coordinate with T-1599 (SORT comparison-key allocation) if both are in flight.

## Acceptance criteria
- [x] A property test (prefer a seeded exhaustive generator; adding a dependency such as proptest needs justification over a small value pool covering every `ArrayNode` variant, case variants of text, -0/+0 and each error kind) checks antisymmetry and transitivity of `sort_key_cmp` and `key_cmp` in both directions.
- [x] If any violation is found, the comparator is fixed so it matches Excel's documented order (docs/excel-semantics/arrays.md SORT and GROUPBY sections) or the sorts use a total-order wrapper; no evaluated SORT/UNIQUE/GROUPBY probe cell changes (`make eval-check` 0 regressions).
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing Excel-visible sort order (any change needs oracle evidence).

## Outcome
- Added `sort_comparators_are_total_orders_on_mixed_types` (grouping.rs tests): an exhaustive check
  of reflexivity, antisymmetry and transitivity over every pair and triple of a 104-value pool
  (every `ArrayNode` variant; -0/+0, subnormals, 15-digit neighbours, infinities, ±NaN; texts by
  case, accent, expansion, skipped character, ASCII vs non-ASCII path and lone surrogates; all 17
  error kinds) for `sort_key_cmp` and `key_cmp`, both directions. No dependency added.
- No violation found, so no comparator changed and no sort order moved; eval delta 0. The composite
  comparators (SORTBY's key chain, `sort_by_keys`' field chain, `sort_by_aggregate`'s parent tree)
  are lexicographic over these and so total too.
- The test catches a non-total comparator: run over raw `compare_values`, it fails on
  (-0, "", blank), since a blank equals both 0 and "". `sort_key_cmp` handles blanks before
  `compare_values`, and its doc comment now says that check must stay.
