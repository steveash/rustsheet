---
id: T-3181
title: "Decide Excel's collation prefix rule and long mark runs: a sort-safe implementation or a DEVIATION"
milestone: M2
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-3154]
---

## Goal
The text comparisons Excel makes that are not a total order are either matched where it is safe
or recorded as a DEVIATION (decision 0014) with accepted-deviation entries, so the remaining
t3154 collation cells stop being unexplained known failures.

## Context
- T-3154 found (coercion.md "Accent weights add up and wrap"): Excel's accent is the Windows
  diacritic weight summed over combining marks in a wrapping byte, but a text sorts before any text
  it is a proper prefix of even when the wrapped weight says otherwise
  (probes/t3154-collation-accent-pairs Calc!S19, T18, W23, X22, AB28, AC27), and runs of 20 or more
  of one mark do not follow the sum (probes/t3154-collation-accent-weights Calc!A105:A112; their
  presence shifts the ranks of A2:A68 by one).
- `cmp_text` is shared by the `<`/`=`/`>` operators, criteria, lookups and the sorts (GROUPBY's
  `grouping.rs` sorts with it; `slice::sort_by` may panic on a non-total order since Rust 1.81).
- Options: apply the prefix rule only in pairwise comparisons (operators, criteria) while sorts
  keep the total order; or declare a DEVIATION for the prefix rule (directive 5, no panics) with
  `eval/suites/probes/deviations` entries via a decision record. The long-run rule needs a probe of
  its own first (e.g. n acutes for n = 9..30 against fixed references, pairwise).

## Acceptance criteria
- [ ] A probe pins how runs of one combining mark beyond 8 compare (rule stated in coercion.md).
- [ ] The prefix rule is implemented where it cannot reach a sort, or recorded as a DEVIATION with
      a decision record and accepted-deviation entries; no sort can panic (a test over the t3154
      spellings checks `cmp_text` stays a total order wherever sorts use it).
- [ ] The UNVERIFIED engine choices T-3154 recorded in coercion.md (U+0363-U+036F as letters, a
      mark after an expansion, the variant pass before the skipped pass, the table-only mark
      weights) are probed or stay listed as unprobed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Scripts other than Latin, Greek and Cyrillic.
