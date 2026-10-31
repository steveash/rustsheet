---
id: T-3454
title: "load_merge_cells checks each merged range against every earlier one, quadratic in the ranges"
milestone: M12
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Importing a sheet with many merged ranges costs time close to linear in their number. A crafted file with 10^5 small
`<mergeCell>`s must not cost 10^10 intersection checks.

## Context
Found in T-3442. `load_merge_cells` (crates/rustsheet-xlsx/src/import/worksheets.rs) keeps the first of any
intersecting ranges. It does this by checking each new range against every range kept so far
(`merged_cells.iter().any(|m| m.intersects(..))`), which is O(n^2) in the number of ranges. A few MB of `<mergeCell>`
elements (about 30 bytes each) then mean about 10^10 checks (directive 5). Keep the same first-wins result, which
T-3442's sweep relies on, since kept ranges never overlap. Use a structure that answers "does this rectangle hit a
kept one" in log time, for example a sweep over rows with an interval tree of open column spans, or sorted kept
ranges bucketed by row block.

## Acceptance criteria
- [ ] A test imports a sheet with 10^5 merged ranges, overlapping ones among them, in well under a second, and keeps
      exactly the ranges the current rule keeps.
- [ ] `make eval-check` shows no regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
