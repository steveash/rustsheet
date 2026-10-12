---
id: T-1417
title: "Probe IMAGE with an error element inside a sizing/height/width array"
milestone: M1
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's result for IMAGE with an error element inside an array `sizing`, `height` or `width` is recorded, and the engine matches it.

## Context
T-1370 lifts array options element by element (`Model::image_grid`, functions/information/offline_errors.rs). An error element (`IMAGE(url,"alt",{0,#DIV/0!})`) is treated as an invalid option, `#VALUE!`; Excel may propagate the error instead (UNVERIFIED in information.md). Probe with ERROR.TYPE in probes/rich-errors-image-array-options style.

## Acceptance criteria
- [ ] A probe case covers an error element in `sizing`, `height` and `width` arrays, with a scalar and an array source; oracle filled.
- [ ] information.md is VERIFIED for it and the engine passes the probe.
- [ ] `make check` green; reviewers approve.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
