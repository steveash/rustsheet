---
id: T-1370
title: "IMAGE array-valued sizing/height/width lift with an array source"
milestone: M1
lane: engine
status: in-progress
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
