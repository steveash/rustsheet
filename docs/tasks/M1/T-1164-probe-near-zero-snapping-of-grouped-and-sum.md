---
id: T-1164
title: "Probe near-zero snapping of grouped and SUM defined-name bodies (follow-up of T-1160)"
milestone: M1
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's near-zero snapping of a defined-name body written `=(0.1+0.2-0.3)`, `=((0.1+0.2-0.3))` and `=SUM(0.1,0.2,-0.3)` is known with oracle evidence, and the engine matches it.

## Context
T-1160 snaps a name body at its root via `evaluate_formula_root`. A name body is parsed with `Parser::parse`, which drops parentheses, so a grouped body `=(X-b)` is now snapped, while the cell form is not (numbers.md "Near-zero snapping"). Both the grouped and the SUM name body are unprobed.

## Acceptance criteria
- [ ] A probe case (excel-graph) with names `nzg` = `=(0.1+0.2-0.3)`, `nzgg` = `=((0.1+0.2-0.3))`, `nzs` = `=SUM(0.1,0.2,-0.3)` used as `=nzg`, `=nzgg`, `=nzs`; the finding is in numbers.md.
- [ ] The engine matches the oracle (keep the grouped root for name bodies if Excel does not snap them); eval baseline stays consistent.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Added probes/near-zero-snap-name-bodies (excel-graph sidecar, 7 cells). Excel does not snap a grouped name body (`(0.1+0.2-0.3)`, `((0.1+0.2-0.3))`) and does snap `SUM(0.1,0.2,-0.3)`. The engine snapped the grouped ones because name bodies were parsed without a grouped root; they now keep it (model.rs `parse_internal_cell_formula`, `user_formula_to_internal`; xlsx import re-parse). Eval delta: 7/7 cells pass, no baseline change, 0 regressions. numbers.md updated.
