---
id: T-1427
title: "Probe IMAGE with a scalar error option beside an array option, and a non-URL source beside an option error element"
milestone: M1
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's result is recorded and the engine matches it for (a) `IMAGE(url,"alt",#DIV/0!,{5,5})` (a scalar error option beside an array option: a per-element grid or one scalar error) and (b) a non-URL source element (a number) beside an option error element.

## Context
T-1417 reviewer follow-up. `Model::image_grid` (functions/information/offline_errors.rs) returns the option error per element in both cases; neither is probed. Extend probes/rich-errors-image-array-error-elements (delete the oracle sidecar, then `rustsheet-oracle fill`).

## Acceptance criteria
- [x] Probe cells for (a) and (b) with oracle filled; information.md records them VERIFIED and the engine passes.
- [x] `make check` green; reviewers approve.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe cells added to `rich-errors-image-array-error-elements` (now 59 cells). Excel lifts a scalar error option over an array option
(a grid, not one scalar error) and an option error element beats a non-URL source element. `offline_args` now keeps a scalar
option error for `image_grid` when IMAGE has an array option (STOCKHISTORY unchanged); `information.md` is VERIFIED.
