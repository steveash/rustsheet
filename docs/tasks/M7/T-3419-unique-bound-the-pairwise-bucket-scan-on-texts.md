---
id: T-3419
title: "UNIQUE: bound the pairwise bucket scan on texts that share a CollationKey but Excel finds unequal"
milestone: M7
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-3211]
---

## Goal
UNIQUE's work stays near-linear in its input on every workbook, including texts built so that many share one
`CollationKey` while Excel's `=` finds them unequal, with no change to any probed UNIQUE result.

## Context
- Found by the T-3211 security and Excel-fidelity reviewers (PR #2564). `unique_of` (functions/spill_functions.rs)
  buckets lines by `CollationKey` and walks the whole bucket with `cmp_text_pairwise` for each line. Texts such as
  `"a"` plus runs of U+0302 whose wrapped sums are equal share a key but are pairwise unequal (coercion.md "Not a
  total order"), so one bucket can hold k kept lines and the call costs O(n * k * length); its `LiftCost::Cells`
  charge does not count it (directive 5).
- Options: a finer hash that pairwise-equal texts still share (for example the key of the text after the
  shared-leading-unit rule), a work charge per comparison through the formula budget (SPEC-0024), or a cap with a
  DEVIATION entry (decision 0014).
- Evidence for the semantics to keep: probes/t3211-collation-sort-cycles G1:J15, t3181-collation-prefix-arrays B1, B2.

## Acceptance criteria
- [ ] A unit test builds UNIQUE over thousands of distinct same-key texts and stays within a stated budget, or
      the call ends with the formula budget's error.
- [ ] Every UNIQUE eval cell keeps its result (`make eval-check`: 0 regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- UNIQUE's sort-order grouping on cycles (T-3418).
