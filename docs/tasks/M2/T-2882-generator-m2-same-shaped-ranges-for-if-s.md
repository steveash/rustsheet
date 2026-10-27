---
id: T-2882
title: "Generator M2: same-shaped ranges for *IF(S) so criteria are scored"
milestone: M2
lane: harness
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Generated SUMIF/SUMIFS/COUNTIFS/AVERAGEIFS formulas pair ranges of the same shape, so the criteria are scored rather than masked by `#VALUE!`.

## Context
Found by T-2717 reviews: about half the cells of gen-m2 cases 7d61ca5cd4679d78 and a073d3573617bf8e are `#VALUE!` because the criteria range and sum range differ in shape, so the new text-number/date criteria never reach scoring. Code: `argument_text` in `tools/src/rustsheet_tools/gen/generator.py`.

## Acceptance criteria
- [ ] Paired range parameters of the *IF(S) family are drawn with equal height and width (unit test); a small focused batch is filled with excel-graph and baselined with 0 regressions; any new mismatch family gets a functions task.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
