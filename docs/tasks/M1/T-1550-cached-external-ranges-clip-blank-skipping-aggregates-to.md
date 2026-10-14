---
id: T-1550
title: "Cached external ranges: clip blank-skipping aggregates to the cache, probe COUNTA over a cached empty string and GCD/LCM over cached text"
milestone: M1
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`SUM([1]Uses!$A$2:$B$1048576)` over a cached sheet gives Excel's value instead of the array-bound
`#ERROR!`, and two unprobed cached-range readers have oracle evidence.

## Context
Left over from T-1526 (its Context listed them). `external_aggregate_value` (external_links.rs)
clips only whole columns and rows to the cache extent; any other cached area keeps its shape and
blanks, so a tall area trips the array bound (arrays.md "Array results above 1,048,576 cells",
DEVIATION). Aggregates that skip blanks (SUM, COUNT, MAX, AVERAGE, ...) could clip any area to the
cache extent; only blank-sensitive readers (TEXTJOIN, CONCAT) need the full shape. Narrow the
DEVIATION entry accordingly. Also probe (excel-probe; an xlsx fixture with an externalLink part,
as fixtures/external-link-cached-ranges): COUNTA over a cached `t="str"` empty string, and GCD/LCM
over a cached range holding text.

## Acceptance criteria
- [ ] `SUM`/`COUNT`/`MAX` over a tall cached area read the cached cells (unit test in
      `test_external_links.rs`); the arrays.md DEVIATION names only the readers that keep blanks.
- [ ] The COUNTA and GCD/LCM questions have oracle values recorded in references.md and the
      engine matches them (or a follow-up is filed).
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Handoff
Opus run of 2026-10-14 claimed this, then released it before any code because the session budget ran
out. It could not delete the claim branch (the remote refuses branch deletes), so this note hands it on.
- Clipping cannot happen in `external_aggregate_value` unconditionally. `evaluate_reference_arg`
  (functions/util.rs, the external branch) feeds it to every reader, including shape-sensitive ones:
  INDEX (index.rs, index_arg.rs), LET, BYCOL/BYROW, FREQUENCY, IF/CHOOSE (logical/mod.rs), and
  CONCAT/TEXTJOIN (text/concatenate.rs). TEXTJOIN must keep the blanks
  (`test_external_links.rs` "10|5|||").
- Suggested shape: give `external_aggregate_value` a "skip blanks" flag (or a sibling) and set it
  only on the blank-skipping aggregate paths: sum.rs, count.rs, count_and_average.rs, variance.rs,
  product.rs, mode_functions.rs, and SUBTOTAL/AGGREGATE/RANK via `external_reference_cells`.
  Then narrow the arrays.md DEVIATION.
- Probes need a new xlsx fixture with an externalLink part. tools/ has no builder for one, so
  copy fixtures/cases/external-link-cached-ranges.xlsx, edit its sheet XML and
  `xl/externalLinks/externalLink1.xml` cache, re-zip it as a new fixture, and fill it with excel-graph.
