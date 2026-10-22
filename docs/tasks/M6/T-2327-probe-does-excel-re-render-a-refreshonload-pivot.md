---
id: T-2327
title: "Probe: does Excel re-render a refreshOnLoad pivot or keep the builder's rendered cells"
milestone: M6
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
docs/excel-semantics/web.md records whether excel-graph re-renders a `refreshOnLoad` pivot or keeps the rendered cells the case builder wrote, so probes/pivot-rendered-cells is real evidence for the layout.

## Context
T-2084's probe reads cells the builder itself writes into the sheet, so it cannot tell Excel's own rendering from the builder's constants (Excel-fidelity review). Needs a builder option that writes stale or perturbed rendered cells and records under `refreshOnLoad` (tools/src/rustsheet_tools/pivot.py), probed through `=P!A3`/`ISBLANK`. Also confirm the `Count of`, `Max of` and `Min of` captions.

## Acceptance criteria
- [ ] A probe with an excel-graph oracle shows whether stale rendered cells are replaced; the web.md entry loses its UNVERIFIED note either way.
- [ ] Captions of Count, Max and Min pivots are probed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
