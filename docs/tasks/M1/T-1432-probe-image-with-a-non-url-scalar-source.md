---
id: T-1432
title: "Probe IMAGE with a non-URL scalar source beside an array alt_text or a scalar error option beside an array option"
milestone: M1
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's result is recorded and the engine matches it for `IMAGE("x",{"a","b"})` and `IMAGE(1,"alt",#DIV/0!,{5,5})` (a non-URL scalar source).

## Context
T-1427 reviewer follow-up. Both shapes now go through `Model::image_grid` (functions/information/offline_errors.rs) by inference from URL-source probes. Extend probes/rich-errors-image-array-error-elements (delete the oracle sidecar, then `rustsheet-oracle fill`).

## Acceptance criteria
- [ ] Probe cells for both with oracle filled; information.md records them VERIFIED and the engine passes.
- [ ] `make check` green; reviewers approve.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
