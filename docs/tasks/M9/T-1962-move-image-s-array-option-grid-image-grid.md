---
id: T-1962
title: "Move IMAGE's array-option grid (image_grid) onto the SPEC-0011 driver"
milestone: M9
lane: quality
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-1775]
---

## Goal
IMAGE with an array `alt_text`, `sizing`, `height` or `width` broadcasts through `Model::lift_values` (SPEC-0011), and `image_grid` with its own `bcast_get` loop is deleted, with no result change.

## Context
T-1775's final SPEC-0011 grep (`rg -n "bcast_get|bcast_idx" crates/rustsheet-engine/src`) still finds `information/offline_errors.rs` (`image_grid`, T-1370/T-1417), which the spec's "Today's lifting code" table missed, and CHOOSE's `choose_array` in lookup_and_reference/mod.rs, which T-1810 moves. Evidence to keep: probes/rich-errors-image-array-options, probes/rich-errors-image-array-error-elements (an error element propagates as itself: the source's error first, then the first option error in argument order; a position past a shorter array is `#N/A`). Check the precedence of a past-the-end position against another argument's error element, which `lift_values` reads as an `#N/A` element.

## Acceptance criteria
- [ ] `make eval`: 0 fixes, 0 regressions; probes/rich-errors-image-* pass.
- [ ] The SPEC-0011 grep finds no `bcast_get` in functions/information/.
- [ ] `make check` green; reviewers approve.

## Out of scope
- STOCKHISTORY and FIELDVALUE (already on `lift_values`, T-1774).
