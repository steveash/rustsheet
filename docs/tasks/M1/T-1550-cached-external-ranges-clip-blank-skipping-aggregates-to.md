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
