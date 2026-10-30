---
id: T-3181
title: "Decide Excel's collation prefix rule: a sort-safe pairwise implementation or a DEVIATION"
milestone: M2
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: [T-3154]
---

## Goal
Excel's prefix rule (a text sorts before any text it is a proper prefix of, even when the wrapped
accent weight says otherwise) is either matched where it cannot reach a sort, or recorded as a
DEVIATION (decision 0014) with accepted-deviation entries, so the prefix cells of the t3154
collation probes stop being unexplained known failures.

## Context
- Rescoped by the planner on 2026-10-30: the long-run rule moved to T-3201 (probe) and T-3202
  (engine), the UNVERIFIED T-3154 engine choices to T-3203. Lane moved from research to engine,
  so this task and T-3202 (both in collation.rs) are not in flight together.
- T-3154 found (coercion.md "Accent weights add up and wrap", "Not a total order"):
  `"a"&UNICHAR(834)` < `"a"&UNICHAR(834)&UNICHAR(834)` although the latter equals
  `"a"&UNICHAR(807)`, which is below `"a"&UNICHAR(834)`; likewise `"a"&UNICHAR(803)` < three
  U+0323 < `"á"` < `"a"&UNICHAR(803)`.
- T-3201 generalized the prefix rule (coercion.md "Shared leading units are skipped before the
  sum"): per letter, the leading units both texts share with equal weights are dropped before the
  wrapped sums are compared, and a side with nothing left is less. T-3202's long-run cells are the
  same rule, so the pairwise-or-DEVIATION choice here decides them too.
- Cells owned (decision 0096; gap triage 2026-10-30, report run at 83cf65c2):
  - probes/t3154-collation-accent-pairs Calc!S19, T18, W23, X22, AB28, AC27
  - probes/t3154-collation-accent-weights Calc!A94, A95, A104
- `cmp_text` is shared by the `<`/`=`/`>` operators, criteria, lookups and the sorts (GROUPBY's
  `grouping.rs`, TEXTSPLIT, the sort helpers in `functions/util.rs`); `slice::sort_by` may panic
  on a non-total order since Rust 1.81. XMATCH and XLOOKUP hash `CollationKey` (T-0448), so
  equality must not change.
- Options: apply the prefix rule only in pairwise comparisons (operators, criteria; one entry
  point beside `cmp_text`, not a copy of it) while sorts keep the total order; or declare a
  DEVIATION for the prefix rule (directive 5, no panics) with `eval/suites/probes/deviations`
  entries via a decision record. If pairwise, check what Excel's SORT, SORTBY and MATCH(...,1)
  do with such pairs before choosing which callers get it (probe if no case covers it).

## Acceptance criteria
- [x] The prefix rule is implemented where it cannot reach a sort, or recorded as a DEVIATION with
      a decision record and accepted-deviation entries; either way the cells above pass or are
      accepted deviations (`make eval-check`: 0 regressions; `make eval-baseline` run).
- [x] No sort can panic: a unit test over the Data strings of the two t3154 weight/pair cases checks
      that `cmp_text` (what sorts use) stays a total order and that `CollationKey` equality matches
      `Ordering::Equal`.
- [x] coercion.md's "Not a total order" text says what the engine does, citing the cells.
- [x] `make check` green; reviewers approve.

## Out of scope
- Long runs of one mark (T-3201, T-3202).
- The UNVERIFIED T-3154 engine choices (T-3203).
- Scripts other than Latin, Greek and Cyrillic.

## Outcome
- Probed the open question first (probes/t3181-collation-prefix-callers, M2, 35 cells;
  probes/t3181-collation-prefix-arrays, M7, 13 cells; excel-graph): Excel applies the pairwise
  rule in every caller, operators, criteria, approximate and exact lookups, SWITCH, SORT, SORTBY
  and UNIQUE, and its `=` is the rule's too (`"a"` is not equal to `"a"` plus 16 U+0302, whose
  wrapped sum is the same). Recorded in coercion.md "Not a total order".
- Neither option of the task as written: no DEVIATION is needed for the operators and criteria,
  which never sort. `cmp_text_pairwise` (collation.rs) applies the full T-3201 rule (shared
  leading units skipped, a side with none left is less, else wrapped sums) through one accent-unit
  stream that also gives the sort's wrapped sums (`accent_units`), so there is still one accent
  computation. The comparison operators (arithmetic.rs) and every text criterion (criteria.rs)
  use it; the criteria index keeps hashing `CollationKey` and tests each candidate, since texts
  the rule finds equal always share a key. Sorts, lookups and `CollationKey` keep the total order.
- This also delivers T-3202: every cell of the four t3154/t3201 accent cases passes, and the
  total-order unit test covers all four cases' Data strings.
- Eval delta: probes known failures 2676 -> 2531 (fixed 170: the 9 cells above, T-3202's 161 and
  the operator/criteria cells of the new cases), 25 new-coverage failures in the two new cases,
  owned by T-3210 (lookups, sonnet) and T-3211 (SORT/SORTBY/UNIQUE, opus); 0 regressions.
