---
id: T-1370
title: "IMAGE array-valued sizing/height/width lift with an array source"
milestone: M1
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`IMAGE({url,url},"alt",{0,9})` gives `#BLOCKED!`, `#VALUE!` like Excel: an array-valued `sizing`, `height` or `width` lifts with an array `source`.

## Context
probes/rich-errors-image-sizing B121 (T-1345): Excel gives A121 `#BLOCKED!`, B121 `#VALUE!`; the engine gives `#BLOCKED!` for both because `image_options_valid` (functions/information/offline_errors.rs) skips array-valued options. Probe first how shapes broadcast (a 1x2 source against a 2x1 `sizing`, and an array `height`).

## Acceptance criteria
- [ ] probes/rich-errors-image-sizing B121 leaves the baseline; new probe cells cover the other broadcast shapes.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- New probe `probes/rich-errors-image-array-options` (61 cells, excel-graph): the source and an array `sizing`, `height` or `width` broadcast against each other (length-1 axis repeats, a shorter array is `#N/A` beyond its end) and each element is validated alone. `Model::image_grid` implements it; `image_options_valid` no longer special-cases arrays.
- probes/rich-errors-image-sizing B121 leaves the baseline. Rule in information.md.
