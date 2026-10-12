---
id: T-1427
title: "Probe IMAGE with a scalar error option beside an array option, and a non-URL source beside an option error element"
milestone: M1
lane: research
status: in-progress
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
- [ ] Probe cells for (a) and (b) with oracle filled; information.md records them VERIFIED and the engine passes.
- [ ] `make check` green; reviewers approve.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
