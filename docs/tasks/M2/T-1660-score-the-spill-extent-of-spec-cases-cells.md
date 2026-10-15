---
id: T-1660
title: "Score the spill extent of spec cases: cells absent from the oracle sidecar must be empty in the engine"
milestone: M2
lane: harness
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A spec case fails when the engine spills into a cell that Excel left empty, so a spill-extent divergence cannot hide.

## Context
The harness scores only the cells present in a case's oracle sidecar. In probes/ceiling-floor-mround-array-lift the engine spilled `{#VALUE!, 3}` for `MROUND(K1:L1,1)` where Excel gives one `#VALUE!` with B21 empty, and the case still scored 65/65 (found by the fidelity reviewer of T-1589). Decide how absence is recorded (an explicit empty entry in the oracle fill for the neighbours of a formula cell, or a per-case check of the engine's used area) and what it does to the existing baselines (expect new known failures to be recorded, never hand-edited).

## Acceptance criteria
- [ ] A cell outside the oracle's results that the engine fills is reported as a mismatch for every spec suite; the baselines are regenerated with `make eval-baseline`, with the new known failures listed in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
