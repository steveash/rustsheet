---
id: T-4943
title: "Render a pivot's date items under a date number format, as Excel's refresh does"
milestone: M16
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A pivot's date items render under a date number format, so the engine's displayed text of a date row label is Excel's (`1/1/1900`, `2/29/1900`; in a 1904 workbook serial 0 shows `12:00:00 AM`).

## Context
T-4177's pivot-export suite (excel-graph, 2026-11-07): after a refresh, Excel shows a date item of a row field as a date, while the engine's render writes the serial with no number format, so the value matches and the displayed text does not (TF lines, decision 0070):
- pivot-export/t4177-cache-dates-1900-refresh.export.json P!A4:A10 (`1/1/1900`, `1/2/1900`, `1/31/1900`, `2/28/1900`, `2/29/1900` for the phantom 60, `3/1/1900`, `3/2/1900`; the engine `1` ... `62`);
- t4177-cache-dates-1904-refresh.export.json P!A4:A10 (serial 0 is `12:00:00 AM`, then `1/2/1904` ...);
- t4177-api-items-refresh.export.json P!A8 (`1/1/2020`).
Which format Excel applies (the source cell's `yyyy-mm-dd` was not kept: it shows `m/d/yyyy`) and how a pivot without refreshOnLoad keeps it are open. Code: the grid writer (crates/rustsheet-engine/src/pivot/grid.rs).

## Acceptance criteria
- [ ] The text of those cells matches Excel's, and pivots.md records the format rule with this evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
