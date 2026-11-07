---
id: T-4942
title: "Export a pivot collapsed to #SPILL! so Excel for the web opens it collapsed"
milestone: M16
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A pivot the engine collapsed to `#SPILL!` (a foreign cell in its layout) is exported so that Excel for the web opens it as a collapsed pivot, `#SPILL!` at its anchor, as Excel saves its own collapsed pivots.

## Context
The export writes a collapsed pivot's definition with a one-cell location and no `rowItems`/`colItems` (crates/rustsheet-xlsx/src/export/pivots/table.rs; the layout lines are dropped with the collapse). Excel for the web saves its own collapsed pivot with its lines kept (probes/t4128-pivot-spike S: location A3, rowItems 3, colItems 4). T-4177's pivot-export suite shows what Excel makes of the export (excel-graph, 2026-11-07):
- pivot-export/t4177-api-spill.export.json (API pivot at P!A3, the constant P!A5 in its way, no refreshOnLoad): Excel did not keep `#SPILL!`. It saved location A3:B3 holding only `Row Labels` and `Sum of Sales`, with no lines, and both GETPIVOTDATA cells gave `#REF!` (the engine: `#SPILL!` at A3 and `#SPILL!`).
- t4177-api-spill-refresh.export.json (refreshOnLoad patched on): Excel's refresh collapsed it again, `#SPILL!` at A3, and kept rowItems 3 and colItems 1; the engine saves none (report "Pivot layout": LinesDiffer). t4177-spike.export.json S is the same.
Writing the lines needs the layout the collapse dropped, or Excel's own reading of a definition without them. Probe a collapsed definition with Excel's lines and no refreshOnLoad first.

Owner of these visibly-off cells (decision 0245; T-4177, Excel opened the exported collapsed pivot as a header-only pivot):
- pivot-export/t4177-api-spill.export.json P!A3:B3, Calc!B1:B2

## Acceptance criteria
- [ ] pivots.md "Excel for the web opens engine-exported pivot parts" records what Excel opens for a collapsed definition with lines.
- [ ] pivot-export/t4177-api-spill.export.json passes, and the collapsed pivots of t4177-api-spill-refresh and t4177-spike have Excel's rowItems/colItems.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
